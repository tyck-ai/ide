use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store, StoreLimits, StoreLimitsBuilder};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

use super::limits::WasmLimits;
use super::sandbox::WasiCapabilities;
use crate::apps::error::TappError;

pub struct WasmHostState {
    ctx: WasiCtx,
    table: ResourceTable,
    limits: WasmLimits,
    pub store_limits: StoreLimits,
}

impl WasiView for WasmHostState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl WasmHostState {
    pub fn new(ctx: WasiCtx, table: ResourceTable, limits: WasmLimits) -> Self {
        let store_limits = StoreLimitsBuilder::new()
            .memory_size(limits.max_memory_bytes)
            .table_elements(limits.max_table_elements as usize)
            .build();
        Self { ctx, table, limits, store_limits }
    }

    pub fn limits(&self) -> &WasmLimits {
        &self.limits
    }
}

struct CachedComponent {
    component: Component,
    modified_time: SystemTime,
}

pub struct ComponentCache {
    cache: HashMap<PathBuf, CachedComponent>,
    /// Insertion order for FIFO eviction
    order: Vec<PathBuf>,
}

impl ComponentCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            order: Vec::new(),
        }
    }

    const MAX_CACHE_SIZE: usize = 32;

    pub fn get_or_load(&mut self, engine: &Engine, path: &Path) -> Result<Component, TappError> {
        let modified = std::fs::metadata(path)
            .map_err(|e| TappError::WasmLoad(format!("Failed to read file metadata: {}", e)))?
            .modified()
            .map_err(|e| TappError::WasmLoad(format!("Failed to get modified time: {}", e)))?;

        if let Some(cached) = self.cache.get(path) {
            if cached.modified_time == modified {
                return Ok(cached.component.clone());
            }
        }

        let component = Component::from_file(engine, path)
            .map_err(|e| TappError::WasmLoad(format!("Failed to load component from file: {}", e)))?;

        // FIFO eviction: remove the oldest inserted entry
        if self.cache.len() >= Self::MAX_CACHE_SIZE {
            if let Some(oldest) = self.order.first().cloned() {
                self.cache.remove(&oldest);
                self.order.remove(0);
            }
        }

        // Remove from order if re-inserting (updated component)
        self.order.retain(|p| p != path);
        self.order.push(path.to_path_buf());

        self.cache.insert(
            path.to_path_buf(),
            CachedComponent {
                component: component.clone(),
                modified_time: modified,
            },
        );

        Ok(component)
    }

    pub fn invalidate(&mut self, path: &Path) {
        self.cache.remove(path);
        self.order.retain(|p| p != path);
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.order.clear();
    }
}

impl Default for ComponentCache {
    fn default() -> Self {
        Self::new()
    }
}

pub struct WasmHost {
    engine: Engine,
    limits: WasmLimits,
    component_cache: ComponentCache,
    epoch_shutdown: Arc<AtomicBool>,
    epoch_thread: Option<std::thread::JoinHandle<()>>,
}

impl WasmHost {
    pub fn new(limits: WasmLimits) -> Result<Self, TappError> {
        let mut config = Config::new();
        config.async_support(false);
        config.consume_fuel(true);
        config.epoch_interruption(true);
        config.wasm_component_model(true);
        // Note: Memory limits are enforced per-store via StoreLimitsBuilder,
        // not via Config (which is per-engine). See create_store() and WasmInstance::new().

        let engine = Engine::new(&config)
            .map_err(|e| TappError::WasmInit(format!("Failed to create engine: {}", e)))?;

        // Spawn epoch ticker thread: increments the engine epoch every 1 second.
        // Combined with set_epoch_deadline(5) on each WASM call, this gives a ~5s
        // timeout with at most 1s of variance. Tighter interval = more predictable timeouts.
        let epoch_shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_flag = epoch_shutdown.clone();
        let engine_clone = engine.clone();
        let epoch_thread = std::thread::spawn(move || {
            while !shutdown_flag.load(Ordering::Acquire) {
                std::thread::sleep(std::time::Duration::from_secs(1));
                engine_clone.increment_epoch();
            }
        });

        Ok(Self {
            engine,
            limits,
            component_cache: ComponentCache::new(),
            epoch_shutdown,
            epoch_thread: Some(epoch_thread),
        })
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    pub fn limits(&self) -> &WasmLimits {
        &self.limits
    }

    pub fn load_component(&self, wasm_bytes: &[u8]) -> Result<Component, TappError> {
        Component::new(&self.engine, wasm_bytes)
            .map_err(|e| TappError::WasmLoad(format!("Failed to load component: {}", e)))
    }

    pub fn load_component_from_file(&self, path: &Path) -> Result<Component, TappError> {
        Component::from_file(&self.engine, path)
            .map_err(|e| TappError::WasmLoad(format!("Failed to load component from file: {}", e)))
    }

    pub fn load_component_cached(&mut self, path: &Path) -> Result<Component, TappError> {
        self.component_cache.get_or_load(&self.engine, path)
    }

    pub fn invalidate_cache(&mut self, path: &Path) {
        self.component_cache.invalidate(path);
    }

    pub fn clear_cache(&mut self) {
        self.component_cache.clear();
    }

    pub fn create_linker(&self) -> Result<Linker<WasmHostState>, TappError> {
        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::add_to_linker_sync(&mut linker)
            .map_err(|e| TappError::WasmInit(format!("Failed to add WASI to linker: {}", e)))?;
        Ok(linker)
    }

    pub fn create_store(&self, capabilities: WasiCapabilities) -> Result<Store<WasmHostState>, TappError> {
        let mut wasi_builder = WasiCtxBuilder::new();

        if capabilities.inherit_stdout {
            wasi_builder.inherit_stdout();
        }
        if capabilities.inherit_stderr {
            wasi_builder.inherit_stderr();
        }

        for (key, value) in &capabilities.env_vars {
            wasi_builder.env(key, value);
        }

        for path in &capabilities.fs_preopens {
            if path.exists() && !path.is_symlink() {
                let dir_perms = if capabilities.fs_writable {
                    wasmtime_wasi::DirPerms::READ | wasmtime_wasi::DirPerms::MUTATE
                } else {
                    wasmtime_wasi::DirPerms::READ
                };
                let file_perms = if capabilities.fs_writable {
                    wasmtime_wasi::FilePerms::READ | wasmtime_wasi::FilePerms::WRITE
                } else {
                    wasmtime_wasi::FilePerms::READ
                };
                wasi_builder.preopened_dir(
                    path,
                    path.to_string_lossy().to_string(),
                    dir_perms,
                    file_perms,
                ).map_err(|e| TappError::WasmInit(format!("Failed to preopen dir: {}", e)))?;
            }
        }

        let wasi_ctx = wasi_builder.build();
        let state = WasmHostState::new(wasi_ctx, ResourceTable::new(), self.limits.clone());

        let mut store = Store::new(&self.engine, state);
        store.limiter(|s| &mut s.store_limits);
        store.set_fuel(self.limits.max_execution_fuel)
            .map_err(|e| TappError::WasmInit(format!("Failed to set fuel: {}", e)))?;
        store.set_epoch_deadline(5);

        Ok(store)
    }
}

impl Drop for WasmHost {
    fn drop(&mut self) {
        self.epoch_shutdown.store(true, Ordering::Release);
        if let Some(handle) = self.epoch_thread.take() {
            let _ = handle.join();
        }
    }
}

pub type SharedWasmHost = Arc<RwLock<WasmHost>>;

pub fn create_shared_host(limits: WasmLimits) -> Result<SharedWasmHost, TappError> {
    let host = WasmHost::new(limits)?;
    Ok(Arc::new(RwLock::new(host)))
}
