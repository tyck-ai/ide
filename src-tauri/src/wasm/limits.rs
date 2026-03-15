use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmLimits {
    pub max_memory_bytes: usize,
    pub max_execution_fuel: u64,
    pub max_table_elements: u32,
}

impl Default for WasmLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 256 * 1024 * 1024, // 256MB per app
            max_execution_fuel: 100_000_000,      // ~5s of compute
            max_table_elements: 10_000,
        }
    }
}

impl WasmLimits {
    pub fn restricted() -> Self {
        Self {
            max_memory_bytes: 64 * 1024 * 1024, // 64MB
            max_execution_fuel: 10_000_000,      // ~500ms
            max_table_elements: 1_000,
        }
    }

    pub fn generous() -> Self {
        Self {
            max_memory_bytes: 512 * 1024 * 1024, // 512MB
            max_execution_fuel: 500_000_000,      // ~25s
            max_table_elements: 50_000,
        }
    }
}
