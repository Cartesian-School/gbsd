// servers/init_server/src/main.rs
// GBSD init_server - PID 1, bootstrap and service management

#![no_std]
#![no_main]

extern crate core;

use core::panic::PanicInfo;
use core::fmt::Write;

// Message types
const CMD_SERVICE_DIED: u64 = 1;
const CMD_REBOOT: u64 = 2;
const CMD_STATUS: u64 = 3;

// Service IDs
const LOG_SERVER_IDX: usize = 0;
const SCHEDULER_SERVER_IDX: usize = 1;
const MAX_SERVICES: usize = 10;

// Service addresses (will be provided by bootloader in Phase 3)
// For now, use dummy addresses
const LOG_SERVER_ADDR: u64 = 0x100000;
const SCHEDULER_SERVER_ADDR: u64 = 0x200000;

/// Service descriptor
#[repr(C)]
struct ServiceDescriptor {
    name: [u8; 32],
    binary_addr: u64,
    port: u32,
    pid: u32,
    status: u32,  // 0=Stopped, 1=Starting, 2=Running, 3=Failed
}

const SERVICE_STATUS_STOPPED: u32 = 0;
const SERVICE_STATUS_STARTING: u32 = 1;
const SERVICE_STATUS_RUNNING: u32 = 2;
const SERVICE_STATUS_FAILED: u32 = 3;

impl ServiceDescriptor {
    fn new(name: &[u8]) -> Self {
        let mut desc = ServiceDescriptor {
            name: [0u8; 32],
            binary_addr: 0,
            port: 0,
            pid: 0,
            status: SERVICE_STATUS_STOPPED,
        };

        // Copy name
        for (i, &byte) in name.iter().enumerate() {
            if i >= 32 { break; }
            desc.name[i] = byte;
        }

        desc
    }
}

// Global service table
static mut SERVICES: [ServiceDescriptor; MAX_SERVICES] = [
    ServiceDescriptor {
        name: [0; 32],
        binary_addr: 0,
        port: 0,
        pid: 0,
        status: SERVICE_STATUS_STOPPED,
    }; MAX_SERVICES
];

/// Print a string to serial console
fn print_str(s: &str) {
    for c in s.bytes() {
        unsafe {
            // Write to serial port 0x3F8 (COM1)
            core::arch::x86_64::asm!("out al, dx", in("al") c, in("dx") 0x3F8u16);
        }
    }
}

/// Allocate a port via syscall
unsafe fn allocate_port() -> u32 {
    let result: u64;
    core::arch::x86_64::asm!(
        "syscall",
        inout("rax") 1u64 => result,  // SYS_PORT_ALLOCATE = 1
    );
    result as u32
}

/// Spawn a process via syscall
unsafe fn spawn_process(entry: u64, stack: u64) -> u32 {
    let result: u64;
    core::arch::x86_64::asm!(
        "syscall",
        inout("rax") 7u64 => result,  // SYS_SCHED_SPAWN = 7
        in("rdi") entry,
        in("rsi") stack,
        in("rdx") 0,  // name pointer
    );
    result as u32
}

/// Send a message to a port via syscall
unsafe fn send_message(port: u32, msg: &[u64; 8]) -> u64 {
    let result: u64;
    core::arch::x86_64::asm!(
        "syscall",
        inout("rax") 2u64 => result,  // SYS_PORT_SEND = 2
        in("rdi") port as u64,
        in("rsi") msg as *const [u64; 8] as u64,
        in("rdx") 8,
    );
    result
}

/// Receive a message from a port via syscall
unsafe fn recv_message(port: u32, buf: &mut [u64; 8]) -> u64 {
    let result: u64;
    core::arch::x86_64::asm!(
        "syscall",
        inout("rax") 3u64 => result,  // SYS_PORT_RECEIVE = 3
        in("rdi") port as u64,
        in("rsi") buf as *mut [u64; 8] as u64,
        in("rdx") 8,
    );
    result
}

/// Start log_server
unsafe fn start_log_server() -> u32 {
    print_str("[init] Starting log_server...\n");

    let log_port = allocate_port();
    let pid = spawn_process(LOG_SERVER_ADDR, 0x500000);

    let desc = &mut SERVICES[LOG_SERVER_IDX];
    desc.name[0..10].copy_from_slice(b"log_server");
    desc.port = log_port;
    desc.pid = pid;
    desc.binary_addr = LOG_SERVER_ADDR;
    desc.status = SERVICE_STATUS_RUNNING;

    print_str("[init] log_server started (PID ");
    print_u32(pid);
    print_str(")\n");

    pid
}

/// Start scheduler_server
unsafe fn start_scheduler_server() -> u32 {
    print_str("[init] Starting scheduler_server...\n");

    let sched_port = allocate_port();
    let pid = spawn_process(SCHEDULER_SERVER_ADDR, 0x600000);

    let desc = &mut SERVICES[SCHEDULER_SERVER_IDX];
    desc.name[0..16].copy_from_slice(b"scheduler_server");
    desc.port = sched_port;
    desc.pid = pid;
    desc.binary_addr = SCHEDULER_SERVER_ADDR;
    desc.status = SERVICE_STATUS_RUNNING;

    print_str("[init] scheduler_server started (PID ");
    print_u32(pid);
    print_str(")\n");

    pid
}

/// Helper to print u32
fn print_u32(n: u32) {
    if n == 0 {
        print_str("0");
        return;
    }

    let mut buf = [0u8; 10];
    let mut i = 0;
    let mut num = n;

    while num > 0 {
        buf[i] = b'0' + (num % 10) as u8;
        i += 1;
        num /= 10;
    }

    for j in (0..i).rev() {
        unsafe {
            core::arch::x86_64::asm!("out al, dx", in("al") buf[j], in("dx") 0x3F8u16);
        }
    }
}

/// Main entry point
#[no_mangle]
pub extern "C" fn _start() -> ! {
    print_str("[init] init_server started (PID 1)\n");

    unsafe {
        // Allocate init_server's own port
        let init_port = allocate_port();
        print_str("[init] Allocated port ");
        print_u32(init_port);
        print_str(" for init_server\n");

        // Start bootstrap services
        start_log_server();
        start_scheduler_server();

        print_str("[init] All bootstrap services started\n");
        print_str("[init] Waiting for events...\n");

        // Main event loop - handle service messages
        loop {
            let mut msg = [0u64; 8];
            let result = recv_message(init_port, &mut msg);

            if result == 0 {
                match msg[0] {
                    CMD_SERVICE_DIED => {
                        let pid = msg[1] as u32;
                        print_str("[init] Service PID ");
                        print_u32(pid);
                        print_str(" died\n");
                    }
                    CMD_REBOOT => {
                        print_str("[init] Reboot requested\n");
                    }
                    _ => {
                        print_str("[init] Unknown message: ");
                        print_u32(msg[0] as u32);
                        print_str("\n");
                    }
                }
            }
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print_str("[init] PANIC: ");
    if let Some(s) = info.payload().downcast_ref::<&str>() {
        print_str(s);
    }
    print_str("\n");

    loop {
        core::arch::x86_64::hlt();
    }
}

