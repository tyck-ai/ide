---
name: ""
overview: ""
todos: []
isProject: false
---

# Complete Tapp WASM Runtime and Performance Fix

## Problem Summary

The Tapp extension system has two critical issues:

### Issue 1: WASM Code Never Executes

When you "start" an app:

1. The WASM component is loaded but **immediately discarded** (`let _component = ...`)
2. Only `InstanceHandle` (metadata) is stored - NOT the actual `WasmInstance`
3. `__tapp_init()`, `__tapp_render()`, `__tapp_handle()` are **never called**
4. Tools return stubs: `ToolResult::ok()`
5. Hooks return stubs: `HookResult::pass_through()`
6. Frontend `uiTree` stays `null` forever

### Issue 2: Component Model vs Core Module Export Mismatch (ROOT CAUSE)

**This is the fundamental architectural problem:**

The current setup has a mismatch between how exports are defined and how they're accessed:


| Layer         | Current State             | Problem                           |
| ------------- | ------------------------- | --------------------------------- |
| Build target  | `wasm32-wasip2`           | Produces Component Model binary   |
| Export macros | `#[no_mangle] extern "C"` | Generates **core module** exports |
| Host API      | `wasmtime::component::*`  | Expects **component** exports     |


**Core module exports are NOT automatically lifted to component exports.** The Component Model requires explicit interface definitions via WIT (WebAssembly Interface Types) and `canon lift` to expose core functions.

The binary produced has:

- Magic: `0x0061736d 0d000100` (component, version 13)
- WASI imports: `wasi:io/poll@0.2.6`, `wasi:io/error@0.2.6`, etc.
- **No component-level exports** for `__tapp_init`, `__tapp_render`, etc.

Calling `instance.get_func(&mut store, "__tapp_init")` returns `None` because these functions aren't component exports.

### Issue 3: Slow Load Times

- WASM parsing on every start (no caching)
- Blocking I/O on async runtime
- Redundant IPC calls

## Solution Options

There are two viable approaches:

### Option A: Add WIT Interface (Recommended)

Define a proper WIT interface and use `wit-bindgen` to generate bindings. This is the "correct" Component Model approach.

**Pros:**

- Future-proof (Component Model is the standard)
- Type-safe interface definitions
- Proper WASI P2 support

**Cons:**

- Requires significant changes to tapp-macros
- Need to define and maintain WIT files
- More complex build process

### Option B: Switch to WASI Preview 1

Change build target to `wasm32-wasi` (WASI P1) and use core module APIs (`wasmtime::Module`, `wasmtime::Instance`).

**Pros:**

- Simpler implementation
- `#[no_mangle] extern "C"` exports work directly
- Faster to implement

**Cons:**

- WASI P1 is being deprecated
- No Component Model benefits (composability, etc.)

---

## Implementation Plan (Option A: WIT Interface)

### Phase 1: Define WIT Interface

Create [packages/tapp/wit/tapp.wit](packages/tapp/wit/tapp.wit):

```wit
package tapp:runtime@0.1.0;

interface app {
    // Initialize the app, returns error message or empty string on success
    init: func() -> result<_, string>;
    
    // Render the UI tree as JSON
    render: func() -> result<string, string>;
    
    // Handle an action (JSON in, JSON out)
    handle: func(action: string) -> result<string, string>;
    
    // Shutdown the app
    shutdown: func() -> result<_, string>;
}

interface tools {
    // List available tool definitions as JSON
    list-tools: func() -> result<string, string>;
    
    // Call a tool by name with JSON args, returns JSON result
    call-tool: func(name: string, args: string) -> result<string, string>;
}

interface hooks {
    // List registered hooks as JSON
    list-hooks: func() -> result<string, string>;
    
    // Invoke a hook with JSON data, returns JSON result
    invoke-hook: func(hook-type: string, data: string) -> result<string, string>;
}

world tapp-app {
    export app;
    export tools;
    export hooks;
}
```

### Phase 2: Update tapp-macros to Generate WIT Bindings

Instead of generating `#[no_mangle] extern "C"` functions, generate wit-bindgen exports.

Modify [packages/tapp-macros/src/lib.rs](packages/tapp-macros/src/lib.rs):

```rust
use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput, ItemImpl, Attribute, Meta};

// Inline WIT definition to avoid path resolution issues
const TAPP_WIT: &str = r#"
package tapp:runtime@0.1.0;

interface app {
    init: func() -> result<_, string>;
    render: func() -> result<string, string>;
    handle: func(action: string) -> result<string, string>;
    shutdown: func() -> result<_, string>;
}

interface tools {
    list-tools: func() -> result<string, string>;
    call-tool: func(name: string, args: string) -> result<string, string>;
}

interface hooks {
    list-hooks: func() -> result<string, string>;
    invoke-hook: func(hook-type: string, data: string) -> result<string, string>;
}

world tapp-app {
    export app;
    export tools;
    export hooks;
}
"#;

#[proc_macro_attribute]
pub fn app(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    
    // Embed the WIT inline to avoid path resolution issues
    let wit_content = TAPP_WIT;

    let expanded = quote! {
        #input

        // Generate bindings with inline WIT
        tapp::wit_bindgen::generate!({
            world: "tapp-app",
            inline: #wit_content,
        });

        // Implement the generated App trait
        impl exports::tapp::runtime::app::Guest for #name {
            fn init() -> Result<(), String> {
                use tapp::App;
                let mut app = <#name as Default>::default();
                let ctx = tapp::Context::new();
                match app.init(&ctx) {
                    Ok(()) => {
                        tapp::__internal::set_app_instance(Box::new(app));
                        Ok(())
                    }
                    Err(e) => Err(e.to_string()),
                }
            }

            fn render() -> Result<String, String> {
                use tapp::App;
                match tapp::__internal::with_app::<#name, _, _>(|app| app.render()) {
                    Some(tree) => {
                        serde_json::to_string(&tree)
                            .map_err(|e| format!("Failed to serialize UI tree: {}", e))
                    }
                    None => Err("App not initialized".to_string()),
                }
            }

            fn handle(action: String) -> Result<String, String> {
                use tapp::App;
                let action: tapp::Action = serde_json::from_str(&action)
                    .map_err(|e| format!("Failed to parse action: {}", e))?;
                
                match tapp::__internal::with_app_mut::<#name, _, _>(|app| app.handle(action)) {
                    Some(Ok(response)) => {
                        serde_json::to_string(&response)
                            .map_err(|e| format!("Failed to serialize response: {}", e))
                    }
                    Some(Err(e)) => Err(e.to_string()),
                    None => Err("App not initialized".to_string()),
                }
            }

            fn shutdown() -> Result<(), String> {
                use tapp::App;
                match tapp::__internal::with_app_mut::<#name, _, _>(|app| app.shutdown()) {
                    Some(Ok(())) => Ok(()),
                    Some(Err(e)) => Err(e.to_string()),
                    None => Err("App not initialized".to_string()),
                }
            }
        }

        // Register this type as the app export
        export!(#name);
    };

    TokenStream::from(expanded)
}
```

### Phase 3: Add wit-bindgen Dependencies

Update [packages/tapp/Cargo.toml](packages/tapp/Cargo.toml):

```toml
[package]
name = "tapp"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tapp-macros = { path = "../tapp-macros" }
wit-bindgen = "0.36"

[features]
default = []
```

### Phase 4: Update Host to Use bindgen! Macro

The wasmtime `bindgen!` macro generates type-safe Rust bindings for calling WIT interfaces.

Modify [src-tauri/src/wasm/instance.rs](src-tauri/src/wasm/instance.rs):

```rust
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

use crate::apps::error::TappError;
use crate::apps::ui_types::UITree;

// Generate bindings from the WIT interface
wasmtime::component::bindgen!({
    world: "tapp-app",
    path: "packages/tapp/wit",
    async: true,
});

pub struct WasmHostState {
    ctx: WasiCtx,
    table: ResourceTable,
}

impl WasiView for WasmHostState {
    fn ctx(&mut self) -> &mut WasiCtx { &mut self.ctx }
    fn table(&mut self) -> &mut ResourceTable { &mut self.table }
}

pub struct WasmInstance {
    store: Store<WasmHostState>,
    bindings: TappApp,
    app_id: String,
}

impl WasmInstance {
    pub async fn new(
        engine: &Engine,
        component: &Component,
        app_id: String,
    ) -> Result<Self, TappError> {
        let mut linker = Linker::new(engine);
        wasmtime_wasi::add_to_linker_async(&mut linker)
            .map_err(|e| TappError::WasmInit(e.to_string()))?;

        let wasi_ctx = WasiCtxBuilder::new().build();
        let state = WasmHostState {
            ctx: wasi_ctx,
            table: ResourceTable::new(),
        };
        let mut store = Store::new(engine, state);

        // Instantiate using the generated bindings
        let bindings = TappApp::instantiate_async(&mut store, component, &linker)
            .await
            .map_err(|e| TappError::WasmInit(format!("Failed to instantiate: {}", e)))?;

        Ok(Self { store, bindings, app_id })
    }

    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    /// Call the app's init function
    pub async fn call_init(&mut self) -> Result<(), TappError> {
        self.bindings
            .tapp_runtime_app()
            .call_init(&mut self.store)
            .await
            .map_err(|e| TappError::WasmCall(e.to_string()))?
            .map_err(|e| TappError::WasmCall(e))
    }

    /// Call the app's render function
    pub async fn call_render(&mut self) -> Result<UITree, TappError> {
        let json = self.bindings
            .tapp_runtime_app()
            .call_render(&mut self.store)
            .await
            .map_err(|e| TappError::WasmCall(e.to_string()))?
            .map_err(|e| TappError::WasmCall(e))?;

        serde_json::from_str(&json)
            .map_err(|e| TappError::SerializationError(format!("Failed to parse UI tree: {}", e)))
    }

    /// Call the app's handle function
    pub async fn call_handle(&mut self, action: &serde_json::Value) -> Result<serde_json::Value, TappError> {
        let action_json = serde_json::to_string(action)
            .map_err(|e| TappError::SerializationError(e.to_string()))?;

        let response_json = self.bindings
            .tapp_runtime_app()
            .call_handle(&mut self.store, &action_json)
            .await
            .map_err(|e| TappError::WasmCall(e.to_string()))?
            .map_err(|e| TappError::WasmCall(e))?;

        serde_json::from_str(&response_json)
            .map_err(|e| TappError::SerializationError(format!("Failed to parse response: {}", e)))
    }

    /// List tools from the app
    pub async fn call_list_tools(&mut self) -> Result<Vec<crate::apps::tools::ToolDefinition>, TappError> {
        let json = self.bindings
            .tapp_runtime_tools()
            .call_list_tools(&mut self.store)
            .await
            .map_err(|e| TappError::WasmCall(e.to_string()))?
            .map_err(|e| TappError::WasmCall(e))?;

        serde_json::from_str(&json)
            .map_err(|e| TappError::SerializationError(format!("Failed to parse tools: {}", e)))
    }

    /// Call a specific tool
    pub async fn call_tool(&mut self, name: &str, args: &serde_json::Value) -> Result<serde_json::Value, TappError> {
        let args_json = serde_json::to_string(args)
            .map_err(|e| TappError::SerializationError(e.to_string()))?;

        let result_json = self.bindings
            .tapp_runtime_tools()
            .call_call_tool(&mut self.store, name, &args_json)
            .await
            .map_err(|e| TappError::WasmCall(e.to_string()))?
            .map_err(|e| TappError::WasmCall(e))?;

        serde_json::from_str(&result_json)
            .map_err(|e| TappError::SerializationError(format!("Failed to parse tool result: {}", e)))
    }

    /// List hooks from the app
    pub async fn call_list_hooks(&mut self) -> Result<Vec<crate::apps::hooks::HookRegistration>, TappError> {
        let json = self.bindings
            .tapp_runtime_hooks()
            .call_list_hooks(&mut self.store)
            .await
            .map_err(|e| TappError::WasmCall(e.to_string()))?
            .map_err(|e| TappError::WasmCall(e))?;

        serde_json::from_str(&json)
            .map_err(|e| TappError::SerializationError(format!("Failed to parse hooks: {}", e)))
    }

    /// Invoke a hook
    pub async fn invoke_hook(&mut self, hook_type: &str, data: &serde_json::Value) -> Result<serde_json::Value, TappError> {
        let data_json = serde_json::to_string(data)
            .map_err(|e| TappError::SerializationError(e.to_string()))?;

        let result_json = self.bindings
            .tapp_runtime_hooks()
            .call_invoke_hook(&mut self.store, hook_type, &data_json)
            .await
            .map_err(|e| TappError::WasmCall(e.to_string()))?
            .map_err(|e| TappError::WasmCall(e))?;

        serde_json::from_str(&result_json)
            .map_err(|e| TappError::SerializationError(format!("Failed to parse hook result: {}", e)))
    }
}
```

### Phase 5: Update AppManager

Store live `WasmInstance` and actually call the WASM functions.

Key changes to [src-tauri/src/apps/manager.rs](src-tauri/src/apps/manager.rs):

```rust
pub struct AppManager {
    engine: Engine,  // Share engine for efficiency
    instances: Arc<RwLock<HashMap<String, WasmInstance>>>,
    // ... rest of fields
}

pub async fn start_app(&self, app_id: &str) -> TappResult<()> {
    let registry = self.registry.read().await;
    let installed = registry.get(app_id)
        .ok_or_else(|| TappError::AppNotFound(app_id.to_string()))?;

    if self.is_running(app_id).await {
        return Err(TappError::AppAlreadyRunning(app_id.to_string()));
    }

    // Load the component (with caching for performance)
    let component = self.load_component_cached(&installed.wasm_path).await?;

    // Create a live WasmInstance
    let mut instance = WasmInstance::new(&self.engine, &component, app_id.to_string()).await?;

    // Actually call __tapp_init()!
    instance.call_init().await?;

    // Discover and register tools
    if let Ok(tools) = instance.call_list_tools().await {
        let mut tool_reg = self.tool_registry.write().await;
        for tool in tools {
            tool_reg.register(app_id, tool);
        }
    }

    // Discover and register hooks
    if let Ok(hooks) = instance.call_list_hooks().await {
        let mut hook_disp = self.hook_dispatcher.write().await;
        for hook in hooks {
            hook_disp.register(app_id, hook);
        }
    }

    // Store the live instance
    {
        let mut instances = self.instances.write().await;
        instances.insert(app_id.to_string(), instance);
    }

    log::info!("Started app: {}", app_id);
    Ok(())
}

/// Get the UI tree from a running app
pub async fn get_ui(&self, app_id: &str) -> TappResult<UITree> {
    let mut instances = self.instances.write().await;
    let instance = instances.get_mut(app_id)
        .ok_or_else(|| TappError::AppNotRunning(app_id.to_string()))?;

    instance.call_render().await
}

/// Handle an action from the frontend
pub async fn handle_action(&self, app_id: &str, action: serde_json::Value) -> TappResult<serde_json::Value> {
    let mut instances = self.instances.write().await;
    let instance = instances.get_mut(app_id)
        .ok_or_else(|| TappError::AppNotRunning(app_id.to_string()))?;

    instance.call_handle(&action).await
}
```

### Phase 6: Add UI Commands

Add to [src-tauri/src/apps/commands.rs](src-tauri/src/apps/commands.rs):

```rust
#[tauri::command]
pub async fn tapp_get_ui(
    app_id: String,
    manager: State<'_, SharedAppManager>,
) -> Result<UITree, String> {
    let manager = manager.read().await;
    manager.get_ui(&app_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tapp_dispatch_action(
    app_id: String,
    action: Value,
    manager: State<'_, SharedAppManager>,
) -> Result<Value, String> {
    let manager = manager.read().await;
    manager.handle_action(&app_id, action).await.map_err(|e| e.to_string())
}
```

### Phase 7: Frontend Integration

Update [src/lib/components/tapp/TappContainer.svelte](src/lib/components/tapp/TappContainer.svelte):

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { UITree, UIEvent, AppInfo, LayoutMode } from './types';
  import TappRenderer from './TappRenderer.svelte';

  let { appId, layout = 'sidebar', onClose }: Props = $props();

  let uiTree: UITree | null = $state(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let appInfo: AppInfo | null = $state(null);

  async function loadApp() {
    try {
      loading = true;
      error = null;

      // Start the app
      await invoke('tapp_start_app', { appId });

      // Get app info
      const apps = await invoke<AppInfo[]>('tapp_list_apps');
      appInfo = apps.find(a => a.id === appId) || null;

      // Get the UI tree
      try {
        uiTree = await invoke<UITree>('tapp_get_ui', { appId });
      } catch (e) {
        // App might not have UI (tools-only app)
        console.log('No UI available for app:', appId);
        uiTree = null;
      }

      loading = false;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      loading = false;
    }
  }

  async function handleEvent(event: UIEvent) {
    try {
      const action = {
        type: event.data?.action || event.event_type,
        ...event.data,
      };
      
      const response = await invoke<{ render?: boolean }>('tapp_dispatch_action', {
        appId,
        action,
      });

      // Re-render if the response indicates UI changed
      if (response?.render !== false) {
        uiTree = await invoke<UITree>('tapp_get_ui', { appId });
      }
    } catch (e) {
      console.error('Error handling tapp event:', e);
    }
  }

  // ... rest of component
</script>
```

### Phase 8: Performance Optimizations

#### 8.1 Component Cache with mtime Invalidation

```rust
pub struct ComponentCache {
    cache: HashMap<PathBuf, (SystemTime, Component)>,
}

impl ComponentCache {
    pub fn get_or_load(&mut self, engine: &Engine, path: &Path) -> Result<Component, TappError> {
        let modified = std::fs::metadata(path)?.modified()?;
        
        if let Some((cached_time, component)) = self.cache.get(path) {
            if *cached_time == modified {
                return Ok(component.clone());
            }
        }
        
        let component = Component::from_file(engine, path)?;
        self.cache.insert(path.to_path_buf(), (modified, component.clone()));
        Ok(component)
    }
}
```

#### 8.2 Async Loading with spawn_blocking

```rust
pub async fn load_component_cached(&self, path: &Path) -> TappResult<Component> {
    let path = path.to_path_buf();
    let engine = self.engine.clone();
    
    tokio::task::spawn_blocking(move || {
        Component::from_file(&engine, &path)
    })
    .await
    .map_err(|e| TappError::WasmLoad(format!("Task join error: {}", e)))?
    .map_err(|e| TappError::WasmLoad(e.to_string()))
}
```

---

## Implementation Plan (Option B: WASI Preview 1)

If Option A is too complex, here's the simpler alternative:

### Changes Required

1. **Build target**: Change from `wasm32-wasip2` to `wasm32-wasi`
2. **Host API**: Use `wasmtime::Module` and `wasmtime::Instance` instead of component APIs
3. **WASI**: Use `wasmtime_wasi::preview1` instead of preview 2

#### Update tapp-cli build.rs

```rust
cmd.arg("--target")
    .arg("wasm32-wasi")  // Changed from wasm32-wasip2
```

#### Update host.rs

```rust
use wasmtime::{Config, Engine, Module, Store, Instance, Linker};
use wasmtime_wasi::preview1::{WasiP1Ctx, add_to_linker_sync};

pub struct WasmHost {
    engine: Engine,
    limits: WasmLimits,
}

impl WasmHost {
    pub fn new(limits: WasmLimits) -> Result<Self, TappError> {
        let mut config = Config::new();
        config.async_support(true);
        config.consume_fuel(true);
        // No component model needed
        
        let engine = Engine::new(&config)?;
        Ok(Self { engine, limits })
    }

    pub fn load_module_from_file(&self, path: &Path) -> Result<Module, TappError> {
        Module::from_file(&self.engine, path)
            .map_err(|e| TappError::WasmLoad(e.to_string()))
    }
}
```

#### Update instance.rs

```rust
use wasmtime::{Instance, Store, TypedFunc, Memory};

pub struct WasmInstance {
    store: Store<WasiP1Ctx>,
    instance: Instance,
    memory: Memory,
    // Cached typed functions
    init_func: TypedFunc<(), i32>,
    render_func: TypedFunc<(), i32>,
    handle_func: TypedFunc<(i32, i32), i32>,
    alloc_func: TypedFunc<i32, i32>,
    free_func: TypedFunc<(i32, i32), ()>,
    get_response_ptr: TypedFunc<(), i32>,
    get_response_len: TypedFunc<(), i32>,
}

impl WasmInstance {
    pub async fn new(
        host: &WasmHost,
        module: &Module,
        app_id: String,
    ) -> Result<Self, TappError> {
        let mut linker = Linker::new(host.engine());
        wasmtime_wasi::preview1::add_to_linker_sync(&mut linker, |ctx| ctx)?;

        let wasi = WasiCtxBuilder::new().build_p1();
        let mut store = Store::new(host.engine(), wasi);

        let instance = linker.instantiate_async(&mut store, module).await?;

        // Get typed function handles
        let init_func = instance.get_typed_func::<(), i32>(&mut store, "__tapp_init")?;
        let render_func = instance.get_typed_func::<(), i32>(&mut store, "__tapp_render")?;
        let handle_func = instance.get_typed_func::<(i32, i32), i32>(&mut store, "__tapp_handle")?;
        let alloc_func = instance.get_typed_func::<i32, i32>(&mut store, "__tapp_alloc")?;
        let free_func = instance.get_typed_func::<(i32, i32), ()>(&mut store, "__tapp_free")?;
        let get_response_ptr = instance.get_typed_func::<(), i32>(&mut store, "__tapp_get_response_ptr")?;
        let get_response_len = instance.get_typed_func::<(), i32>(&mut store, "__tapp_get_response_len")?;
        
        let memory = instance.get_memory(&mut store, "memory")
            .ok_or_else(|| TappError::WasmInit("No memory export".into()))?;

        Ok(Self {
            store, instance, memory,
            init_func, render_func, handle_func,
            alloc_func, free_func,
            get_response_ptr, get_response_len,
        })
    }

    pub async fn call_init(&mut self) -> Result<(), TappError> {
        let result = self.init_func.call_async(&mut self.store, ()).await?;
        if result != 0 {
            return Err(TappError::WasmCall(self.read_error()?));
        }
        Ok(())
    }

    pub async fn call_render(&mut self) -> Result<UITree, TappError> {
        let result = self.render_func.call_async(&mut self.store, ()).await?;
        if result != 0 {
            return Err(TappError::WasmCall(self.read_error()?));
        }
        let json = self.read_response()?;
        serde_json::from_slice(&json).map_err(|e| TappError::SerializationError(e.to_string()))
    }

    fn read_response(&mut self) -> Result<Vec<u8>, TappError> {
        let ptr = self.get_response_ptr.call(&mut self.store, ())?;
        let len = self.get_response_len.call(&mut self.store, ())?;
        
        let mut buf = vec![0u8; len as usize];
        self.memory.read(&self.store, ptr as usize, &mut buf)?;
        Ok(buf)
    }
}
```

---

## File Changes Summary

### Option A (WIT Interface)


| File                                           | Changes                                              |
| ---------------------------------------------- | ---------------------------------------------------- |
| `packages/tapp/wit/tapp.wit`                   | **NEW** - WIT interface definition                   |
| `packages/tapp/Cargo.toml`                     | Add wit-bindgen dependency                           |
| `packages/tapp-macros/src/lib.rs`              | Generate wit-bindgen exports instead of `extern "C"` |
| `src-tauri/Cargo.toml`                         | Already has wasmtime component-model                 |
| `src-tauri/src/wasm/instance.rs`               | Use `bindgen!` macro for type-safe calls             |
| `src-tauri/src/wasm/host.rs`                   | Add component cache                                  |
| `src-tauri/src/apps/manager.rs`                | Store live instances, call WASM functions            |
| `src-tauri/src/apps/commands.rs`               | Add `tapp_get_ui`, `tapp_dispatch_action`            |
| `src/lib/components/tapp/TappContainer.svelte` | Use new commands                                     |


### Option B (WASI P1)


| File                                           | Changes                                         |
| ---------------------------------------------- | ----------------------------------------------- |
| `packages/tapp-cli/src/commands/build.rs`      | Target `wasm32-wasi` instead of `wasm32-wasip2` |
| `src-tauri/src/wasm/host.rs`                   | Use `wasmtime::Module` instead of `Component`   |
| `src-tauri/src/wasm/instance.rs`               | Use core module APIs with `TypedFunc`           |
| `src-tauri/src/apps/manager.rs`                | Store live instances, call WASM functions       |
| `src-tauri/src/apps/commands.rs`               | Add `tapp_get_ui`, `tapp_dispatch_action`       |
| `src/lib/components/tapp/TappContainer.svelte` | Use new commands                                |


## Recommendation

**Option A (WIT Interface)** is recommended because:

1. Component Model is the future of WebAssembly
2. Better type safety and interface evolution
3. Already using `wasm32-wasip2` target
4. Avoids technical debt of deprecated WASI P1

However, **Option B** is acceptable for a quick fix if:

1. Time is critical
2. Team is unfamiliar with WIT
3. Can be migrated to Component Model later

---

## Pre-Implementation Checklist

Before implementing either option, these preparatory changes are required:

### 1. Add `HookRegistration` struct to host hooks.rs

The host-side `hooks.rs` needs a `HookRegistration` struct that matches the guest SDK. Currently only `RegisteredHook` exists.

**File:** `src-tauri/src/apps/hooks.rs`

```rust
// Add this struct (matches packages/tapp/src/hooks.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookRegistration {
    pub hook_type: HookType,
    pub handler: String,
}

// Update HookDispatcher::register to accept HookRegistration
impl HookDispatcher {
    pub fn register(&mut self, app_id: &str, registration: HookRegistration) {
        let hook = RegisteredHook {
            app_id: app_id.to_string(),
            hook_type: registration.hook_type,
            wasm_handler: registration.handler,
            priority: 0, // Default priority
        };
        let hooks = self.hooks.entry(hook.hook_type).or_insert_with(Vec::new);
        hooks.push(hook);
        hooks.sort_by_key(|h| -h.priority);
    }
}
```

### 2. Align `ToolDefinition` between host and guest

The host `ToolDefinition` has different fields than the guest version:


| Field         | Host (`src-tauri`) | Guest (`packages/tapp`) |
| ------------- | ------------------ | ----------------------- |
| `name`        | ✓                  | ✓                       |
| `description` | ✓                  | ✓                       |
| `handler`     | ✗                  | ✓                       |
| `parameters`  | ✓                  | ✗                       |
| `returns`     | ✓                  | ✗                       |


**Option 1:** Add `handler` to host `ToolDefinition` and use it when deserializing from WASM:

**File:** `src-tauri/src/apps/tools.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub handler: String,  // ADD: WASM export name
    #[serde(default)]
    pub parameters: Vec<ToolParameter>,
    pub returns: Option<String>,
}
```

**Option 2:** Create a separate `GuestToolDefinition` for deserialization:

```rust
/// Tool definition as returned by WASM guest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestToolDefinition {
    pub name: String,
    pub description: String,
    pub handler: String,
}

impl GuestToolDefinition {
    pub fn into_tool_definition(self) -> (ToolDefinition, String) {
        let def = ToolDefinition {
            name: self.name,
            description: self.description,
            parameters: vec![],
            returns: None,
        };
        (def, self.handler)
    }
}
```

### 3. Update `ToolRegistry::register` signature

Current signature requires separate `handler` argument, but the plan passes a single struct:

**Current:**

```rust
pub fn register(&mut self, app_id: &str, definition: ToolDefinition, handler: String) -> TappResult<()>
```

**Change to accept struct with handler:**

```rust
pub fn register(&mut self, app_id: &str, definition: ToolDefinition) -> TappResult<()> {
    let full_name = format!("{}:{}", app_id, definition.name);
    
    if self.tools.contains_key(&full_name) {
        return Err(TappError::ToolError(format!(
            "Tool '{}' already registered",
            full_name
        )));
    }
    
    self.tools.insert(
        full_name,
        RegisteredTool {
            app_id: app_id.to_string(),
            wasm_handler: definition.handler.clone(),
            definition,
        },
    );
    
    Ok(())
}
```

### 4. Fix error variant naming in instance.rs

The plan uses `TappError::Serialization(...)` but the actual variant is `TappError::SerializationError(...)`.

**Either update the plan code to use:**

```rust
TappError::SerializationError(format!("..."))
```

**Or add an alias (not recommended).**

### 5. Create WIT file directory structure

```bash
mkdir -p packages/tapp/wit
```

Create `packages/tapp/wit/tapp.wit` with the interface definition from Phase 1.

### 6. Update all three macros in tapp-macros

The `#[tools]` and `#[hooks]` macros also need to be updated to implement the WIT-generated traits.

**File:** `packages/tapp-macros/src/lib.rs`

```rust
#[proc_macro_attribute]
pub fn tools(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    let self_ty = &input.self_ty;

    // Parse tool methods and build definitions
    let mut tool_entries = Vec::new();
    for item in &input.items {
        if let syn::ImplItem::Fn(method) = item {
            if let Some(attr) = method.attrs.iter().find(|a| a.path().is_ident("tool")) {
                let method_name = &method.sig.ident;
                let (name, description) = parse_tool_attr(attr);
                let name = name.unwrap_or_else(|| method_name.to_string());
                let handler = format!("__tapp_tool_{}", method_name);
                tool_entries.push((name, description.unwrap_or_default(), handler, method_name.clone()));
            }
        }
    }

    // Generate tool definition structs
    let tool_defs: Vec<_> = tool_entries.iter().map(|(name, desc, handler, _)| {
        quote! {
            tapp::ToolDefinition {
                name: #name.to_string(),
                description: #desc.to_string(),
                handler: #handler.to_string(),
            }
        }
    }).collect();

    // Generate match arms for call_tool - COMPLETE IMPLEMENTATION
    let tool_match_arms: Vec<_> = tool_entries.iter().map(|(name, _, _, method_name)| {
        quote! {
            #name => {
                match tapp::__internal::with_app_mut::<#self_ty, _, _>(|app| {
                    app.#method_name(args.clone())
                }) {
                    Some(result) => result,
                    None => tapp::ToolResult::error("App not initialized"),
                }
            }
        }
    }).collect();

    let expanded = quote! {
        #input

        // Implement the tools interface
        impl exports::tapp::runtime::tools::Guest for #self_ty {
            fn list_tools() -> Result<String, String> {
                let tools = vec![#(#tool_defs),*];
                serde_json::to_string(&tools)
                    .map_err(|e| format!("Failed to serialize tools: {}", e))
            }

            fn call_tool(name: String, args: String) -> Result<String, String> {
                let args: serde_json::Value = serde_json::from_str(&args)
                    .map_err(|e| format!("Failed to parse args: {}", e))?;
                
                let result: tapp::ToolResult = match name.as_str() {
                    #(#tool_match_arms)*
                    _ => return Err(format!("Unknown tool: {}", name)),
                };
                
                serde_json::to_string(&result)
                    .map_err(|e| format!("Failed to serialize result: {}", e))
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn hooks(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    let self_ty = &input.self_ty;

    // Parse hook methods and build registrations
    let mut hook_entries = Vec::new();
    for item in &input.items {
        if let syn::ImplItem::Fn(method) = item {
            if let Some(attr) = method.attrs.iter().find(|a| a.path().is_ident("hook")) {
                let method_name = &method.sig.ident;
                let hook_type = parse_hook_attr(attr);
                let handler = format!("__tapp_hook_{}", method_name);
                if let Some(ht) = hook_type {
                    // Store as (hook_type_ident, hook_type_str, handler, method_name)
                    let hook_type_str = match ht.to_string().as_str() {
                        "BeforeInput" => "before_input",
                        "AfterOutput" => "after_output",
                        "OnToolCall" => "on_tool_call",
                        "SessionStart" => "session_start",
                        "SessionEnd" => "session_end",
                        _ => "unknown",
                    };
                    hook_entries.push((ht, hook_type_str.to_string(), handler, method_name.clone()));
                }
            }
        }
    }

    // Generate hook registration structs
    let hook_regs: Vec<_> = hook_entries.iter().map(|(hook_type, _, handler, _)| {
        quote! {
            tapp::HookRegistration {
                hook_type: tapp::HookType::#hook_type,
                handler: #handler.to_string(),
            }
        }
    }).collect();

    // Generate match arms for invoke_hook - COMPLETE IMPLEMENTATION
    let hook_match_arms: Vec<_> = hook_entries.iter().map(|(_, hook_type_str, _, method_name)| {
        quote! {
            #hook_type_str => {
                match tapp::__internal::with_app::<#self_ty, _, _>(|app| {
                    app.#method_name(&data)
                }) {
                    Some(result) => result,
                    None => tapp::HookResult::pass_through(),
                }
            }
        }
    }).collect();

    let expanded = quote! {
        #input

        // Implement the hooks interface
        impl exports::tapp::runtime::hooks::Guest for #self_ty {
            fn list_hooks() -> Result<String, String> {
                let hooks = vec![#(#hook_regs),*];
                serde_json::to_string(&hooks)
                    .map_err(|e| format!("Failed to serialize hooks: {}", e))
            }

            fn invoke_hook(hook_type: String, data: String) -> Result<String, String> {
                let data: serde_json::Value = serde_json::from_str(&data)
                    .map_err(|e| format!("Failed to parse data: {}", e))?;
                
                let result: tapp::HookResult = match hook_type.as_str() {
                    #(#hook_match_arms)*
                    _ => tapp::HookResult::pass_through(),
                };
                
                serde_json::to_string(&result)
                    .map_err(|e| format!("Failed to serialize result: {}", e))
            }
        }
    };

    TokenStream::from(expanded)
}
```

### 7. Add shutdown call in `stop_app()`

**File:** `src-tauri/src/apps/manager.rs`

```rust
pub async fn stop_app(&self, app_id: &str) -> TappResult<()> {
    // Call shutdown on the WASM instance before cleanup
    {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(app_id) {
            if let Err(e) = instance.call_shutdown().await {
                log::warn!("Shutdown failed for {}: {}", app_id, e);
            }
        }
    }

    // Existing cleanup code...
    {
        let mut tool_reg = self.tool_registry.write().await;
        tool_reg.unregister_app(app_id);
    }
    // ... rest of cleanup
}
```

Add to `WasmInstance`:

```rust
pub async fn call_shutdown(&mut self) -> Result<(), TappError> {
    self.bindings
        .tapp_runtime_app()
        .call_shutdown(&mut self.store)
        .await
        .map_err(|e| TappError::WasmCall(e.to_string()))?
        .map_err(|e| TappError::WasmCall(e))
}
```

### 8. Register new commands in lib.rs

**File:** `src-tauri/src/lib.rs`

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    apps::commands::tapp_get_ui,
    apps::commands::tapp_dispatch_action,
])
```

### 9. WIT path resolution in tapp-macros

The `wit_bindgen::generate!` macro needs the correct path to the WIT files. Since the macro runs at compile time from the **app's directory** (not the tapp crate), path resolution is tricky.

**Recommended: Use `inline:` option to embed WIT content**

This is the most reliable approach - embed the WIT content directly:

**In `packages/tapp/src/lib.rs`:**

```rust
// Re-export wit-bindgen for use by macro-generated code
pub use wit_bindgen;

// The WIT content is embedded, so apps don't need the wit file at build time
pub const TAPP_WIT: &str = include_str!("../wit/tapp.wit");
```

**In the generated macro code:**

```rust
wit_bindgen::generate!({
    world: "tapp-app",
    inline: r#"
package tapp:runtime@0.1.0;

interface app {
    init: func() -> result<_, string>;
    render: func() -> result<string, string>;
    handle: func(action: string) -> result<string, string>;
    shutdown: func() -> result<_, string>;
}

interface tools {
    list-tools: func() -> result<string, string>;
    call-tool: func(name: string, args: string) -> result<string, string>;
}

interface hooks {
    list-hooks: func() -> result<string, string>;
    invoke-hook: func(hook-type: string, data: string) -> result<string, string>;
}

world tapp-app {
    export app;
    export tools;
    export hooks;
}
"#,
});
```

**Alternative: Use a build script in tapp crate**

Create `packages/tapp/build.rs` that generates a Rust file with the `wit_bindgen::generate!` call, ensuring paths are resolved at the tapp crate's build time, not the app's.

---

## Checklist Summary


| Task                                        | File                              | Status | Notes                                   |
| ------------------------------------------- | --------------------------------- | ------ | --------------------------------------- |
| Add `HookRegistration` struct               | `src-tauri/src/apps/hooks.rs`     | ⬜      | Code in checklist item 1                |
| Update `HookDispatcher::register` signature | `src-tauri/src/apps/hooks.rs`     | ⬜      | Code in checklist item 1                |
| Add `handler` field to `ToolDefinition`     | `src-tauri/src/apps/tools.rs`     | ⬜      | Code in checklist item 2                |
| Update `ToolRegistry::register` signature   | `src-tauri/src/apps/tools.rs`     | ⬜      | Code in checklist item 3                |
| Fix `Serialization` → `SerializationError`  | Plan code / `instance.rs`         | ✅      | Fixed in plan                           |
| Create `packages/tapp/wit/tapp.wit`         | New file                          | ⬜      | Optional - WIT is inlined in macros     |
| Update `#[app]` macro for WIT               | `packages/tapp-macros/src/lib.rs` | ⬜      | Complete code in Phase 2                |
| Update `#[tools]` macro for WIT             | `packages/tapp-macros/src/lib.rs` | ⬜      | Complete code with match arms in item 6 |
| Update `#[hooks]` macro for WIT             | `packages/tapp-macros/src/lib.rs` | ⬜      | Complete code with match arms in item 6 |
| Add `call_shutdown()` to `WasmInstance`     | `src-tauri/src/wasm/instance.rs`  | ⬜      | Code in checklist item 7                |
| Call shutdown in `stop_app()`               | `src-tauri/src/apps/manager.rs`   | ⬜      | Code in checklist item 7                |
| Register `tapp_get_ui` command              | `src-tauri/src/lib.rs`            | ⬜      | Code in checklist item 8                |
| Register `tapp_dispatch_action` command     | `src-tauri/src/lib.rs`            | ⬜      | Code in checklist item 8                |
| Add wit-bindgen to tapp Cargo.toml          | `packages/tapp/Cargo.toml`        | ⬜      | Code in Phase 3                         |
| Re-export wit-bindgen from tapp             | `packages/tapp/src/lib.rs`        | ⬜      | `pub use wit_bindgen;`                  |


### Implementation Notes

1. **WIT is inlined in macros** - No separate `.wit` file needed at app build time. The WIT definition is embedded directly in the macro code using the `inline:` option. A `.wit` file can still be created for documentation/reference.
2. **Match arm generation is complete** - Both `#[tools]` and `#[hooks]` macros now include full match arm generation for `call_tool` and `invoke_hook` respectively.
3. **wit-bindgen re-export** - The tapp crate must re-export `wit_bindgen` so macro-generated code can use `tapp::wit_bindgen::generate!`.

---

## Test Criteria

1. App loads in <500ms on first launch
2. App loads in <100ms on subsequent launches (cached)
3. UI tree renders (not "coming soon")
4. Button clicks trigger actions
5. UI re-renders after action with `render: true`
6. Tools return actual results from WASM
7. Hooks intercept agent I/O
8. App shutdown is called when stopping

