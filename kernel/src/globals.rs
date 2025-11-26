// kernel/src/globals.rs
// Global kernel state management

use spin::Mutex;
use alloc::vec::Vec;
use core::cell::UnsafeCell;

/// Kernel state - shared between all CPU cores
pub struct KernelState {
    pub processes: Vec<ProcessDescriptor>,
    pub ports: Vec<Port>,
    pub capabilities: Vec<Capability>,
    pub current_process_id: u32,
}

impl KernelState {
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            ports: Vec::new(),
            capabilities: Vec::new(),
            current_process_id: 0,
        }
    }
}

/// Process descriptor
#[derive(Clone, Debug)]
pub struct ProcessDescriptor {
    pub id: u32,
    pub name: [u8; 32],
    pub memory_start: u64,
    pub memory_end: u64,
    pub page_table_root: u64,  // CR3 value
    pub state: ProcessState,
    pub stack_pointer: u64,
    pub instruction_pointer: u64,
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum ProcessState {
    Ready,
    Running,
    Sleeping,
    Dead,
}

/// IPC Port - capability-based message queue
#[derive(Clone, Debug)]
pub struct Port {
    pub id: u32,
    pub owner_pid: u32,
    pub queue: [u64; 512],  // Ring buffer: [u64; 8] × 64 messages max
    pub queue_head: u32,
    pub queue_tail: u32,
    pub queue_size: u32,
    pub max_queue_size: u32,
}

impl Port {
    pub fn new(id: u32, owner_pid: u32) -> Self {
        Self {
            id,
            owner_pid,
            queue: [0u64; 512],
            queue_head: 0,
            queue_tail: 0,
            queue_size: 0,
            max_queue_size: 64,  // Max 64 messages of [u64; 8]
        }
    }

    pub fn is_full(&self) -> bool {
        self.queue_size >= self.max_queue_size
    }

    pub fn is_empty(&self) -> bool {
        self.queue_size == 0
    }

    pub fn push_message(&mut self, msg: &[u64; 8]) -> bool {
        if self.is_full() {
            return false;
        }

        let base_idx = (self.queue_tail as usize) * 8;
        for i in 0..8 {
            self.queue[base_idx + i] = msg[i];
        }

        self.queue_tail = (self.queue_tail + 1) % 64;
        self.queue_size += 1;
        true
    }

    pub fn pop_message(&mut self) -> Option<[u64; 8]> {
        if self.is_empty() {
            return None;
        }

        let base_idx = (self.queue_head as usize) * 8;
        let msg = [
            self.queue[base_idx],
            self.queue[base_idx + 1],
            self.queue[base_idx + 2],
            self.queue[base_idx + 3],
            self.queue[base_idx + 4],
            self.queue[base_idx + 5],
            self.queue[base_idx + 6],
            self.queue[base_idx + 7],
        ];

        self.queue_head = (self.queue_head + 1) % 64;
        self.queue_size -= 1;
        Some(msg)
    }
}

/// Capability - unforgeable access token
#[derive(Clone, Debug, Copy)]
pub struct Capability {
    pub id: u32,
    pub owner_pid: u32,
    pub target_id: u32,  // Port or object ID
    pub rights: u32,
    pub revoked: bool,
}

impl Capability {
    pub fn new(id: u32, owner_pid: u32, target_id: u32, rights: u32) -> Self {
        Self {
            id,
            owner_pid,
            target_id,
            rights,
            revoked: false,
        }
    }

    pub fn has_right(&self, right: u32) -> bool {
        !self.revoked && (self.rights & right) != 0
    }

    pub fn revoke(&mut self) {
        self.revoked = true;
    }
}

/// Global kernel state (protected by SpinLock)
pub static KERNEL_STATE: Mutex<KernelState> = Mutex::new(KernelState {
    processes: Vec::new(),
    ports: Vec::new(),
    capabilities: Vec::new(),
    current_process_id: 0,
});

/// Next capability ID counter
pub static NEXT_CAP_ID: Mutex<u32> = Mutex::new(1);

/// Next port ID counter
pub static NEXT_PORT_ID: Mutex<u32> = Mutex::new(1);

/// Next process ID counter
pub static NEXT_PROCESS_ID: Mutex<u32> = Mutex::new(2);  // Start from 2 (1 is init_server)

/// Get current process ID
pub fn current_pid() -> u32 {
    KERNEL_STATE.lock().current_process_id
}

/// Get mutable reference to kernel state
pub fn kernel_state_mut() -> spin::MutexGuard<'static, KernelState> {
    KERNEL_STATE.lock()
}

