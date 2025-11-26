// kernel/src/error.rs
// GBSD Error Code Definitions

/// System error codes (u64 format: 0xFFFFFFFF_XXXXXXXX)
#[repr(u64)]
pub enum SystemError {
    /// Operation completed successfully
    Ok = 0,

    /// Port is invalid or does not exist
    PortInvalid = 0xFFFFFFFF_00000001,

    /// Port queue is full, cannot send
    PortFull = 0xFFFFFFFF_00000002,

    /// Insufficient rights for operation
    NoRights = 0xFFFFFFFF_00000003,

    /// Invalid arguments provided
    Invalid = 0xFFFFFFFF_00000004,

    /// Out of memory
    NoMemory = 0xFFFFFFFF_00000005,

    /// Capability is invalid or revoked
    CapabilityInvalid = 0xFFFFFFFF_00000006,

    /// Process not found
    ProcessNotFound = 0xFFFFFFFF_00000007,

    /// Not the owner of the resource
    NotOwner = 0xFFFFFFFF_00000008,

    /// Address alignment error
    Alignment = 0xFFFFFFFF_00000009,

    /// Invalid system call
    InvalidSyscall = 0xFFFFFFFF_0000000A,
}

impl SystemError {
    pub fn as_u64(&self) -> u64 {
        *self as u64
    }
}

// Error code constants for C usage
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

/// Syscall numbers
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

/// Capability rights bits
pub const CAP_SEND: u32 = 1 << 0;
pub const CAP_RECEIVE: u32 = 1 << 1;
pub const CAP_DESTROY: u32 = 1 << 2;
pub const CAP_DERIVE: u32 = 1 << 3;
pub const CAP_READ: u32 = 1 << 4;
pub const CAP_WRITE: u32 = 1 << 5;
pub const CAP_EXECUTE: u32 = 1 << 6;

