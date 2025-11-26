// libgbsd/src/lib.rs
// Common GBSD library - syscall definitions and utilities

#![no_std]

/// Error codes
pub mod error {
    pub const E_OK: u64 = 0;
    pub const E_PORT_INVALID: u64 = 0xFFFFFFFF_00000001;
    pub const E_PORT_FULL: u64 = 0xFFFFFFFF_00000002;
    pub const E_NO_RIGHTS: u64 = 0xFFFFFFFF_00000003;
    pub const E_INVAL: u64 = 0xFFFFFFFF_00000004;
    pub const E_NOMEM: u64 = 0xFFFFFFFF_00000005;
    pub const E_CAP_INVALID: u64 = 0xFFFFFFFF_00000006;
    pub const E_PROCESS_NOT_FOUND: u64 = 0xFFFFFFFF_00000007;
    pub const E_NOT_OWNER: u64 = 0xFFFFFFFF_00000008;
    pub const E_ALIGN: u64 = 0xFFFFFFFF_00000009;
    pub const E_INVALID_SYSCALL: u64 = 0xFFFFFFFF_0000000A;
}

/// Syscall numbers
pub mod syscall {
    pub const SYS_PORT_ALLOCATE: u64 = 1;
    pub const SYS_PORT_SEND: u64 = 2;
    pub const SYS_PORT_RECEIVE: u64 = 3;
    pub const SYS_VM_ALLOCATE: u64 = 4;
    pub const SYS_VM_DEALLOCATE: u64 = 5;
    pub const SYS_CAP_MOVE: u64 = 6;
    pub const SYS_SCHED_SPAWN: u64 = 7;
    pub const SYS_SCHED_YIELD: u64 = 8;
    pub const SYS_SCHED_SWITCH: u64 = 9;
    pub const SYS_TIME: u64 = 10;
}

/// Capability rights
pub mod capability {
    pub const CAP_SEND: u32 = 1 << 0;
    pub const CAP_RECEIVE: u32 = 1 << 1;
    pub const CAP_DESTROY: u32 = 1 << 2;
    pub const CAP_DERIVE: u32 = 1 << 3;
    pub const CAP_READ: u32 = 1 << 4;
    pub const CAP_WRITE: u32 = 1 << 5;
    pub const CAP_EXECUTE: u32 = 1 << 6;
}

/// Message format (8 u64s = 64 bytes)
pub type Message = [u64; 8];

/// Syscall wrappers for userspace
#[cfg(target_arch = "x86_64")]
pub mod x86_64_syscalls {
    use super::*;

    /// Allocate a new port
    #[inline]
    pub unsafe fn port_allocate() -> u64 {
        let result: u64;
        asm!("syscall",
             inout("rax") syscall::SYS_PORT_ALLOCATE => result);
        result
    }

    /// Send a message to a port
    #[inline]
    pub unsafe fn port_send(port: u32, msg: *const Message) -> u64 {
        let result: u64;
        asm!("syscall",
             inout("rax") syscall::SYS_PORT_SEND => result,
             in("rdi") port as u64,
             in("rsi") msg as u64,
             in("rdx") 8);
        result
    }

    /// Receive a message from a port
    #[inline]
    pub unsafe fn port_receive(port: u32, buf: *mut Message) -> u64 {
        let result: u64;
        asm!("syscall",
             inout("rax") syscall::SYS_PORT_RECEIVE => result,
             in("rdi") port as u64,
             in("rsi") buf as u64,
             in("rdx") 8);
        result
    }

    /// Get current time (monotonic clock)
    #[inline]
    pub unsafe fn sys_time() -> u64 {
        let result: u64;
        asm!("syscall",
             inout("rax") syscall::SYS_TIME => result);
        result
    }

    /// Yield CPU to scheduler
    #[inline]
    pub unsafe fn sched_yield() -> u64 {
        let result: u64;
        asm!("syscall",
             inout("rax") syscall::SYS_SCHED_YIELD => result);
        result
    }
}

