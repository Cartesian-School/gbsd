// kernel/src/memory.rs
// Memory management - paging, allocation

use spin::Mutex;

pub struct MemoryManager {
    total_memory: usize,
    allocated: usize,
}

impl MemoryManager {
    pub fn new() -> Self {
        Self {
            total_memory: 0x100000000,  // 4 GB
            allocated: 0,
        }
    }
}

pub static MEMORY_MANAGER: Mutex<MemoryManager> = Mutex::new(MemoryManager {
    total_memory: 0x100000000,
    allocated: 0,
});

/// Initialize memory management system
pub fn init() {
    // TODO: Detect actual memory from bootloader
    // Initialize paging
    // Set up allocator
}

