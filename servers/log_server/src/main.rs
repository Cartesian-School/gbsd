// servers/log_server/src/main.rs
// GBSD log_server - centralized logging

#![no_std]
#![no_main]

extern crate core;

use core::panic::PanicInfo;

// Log message types
const LOG_WRITE: u64 = 1;
const LOG_FLUSH: u64 = 2;
const LOG_READ_TAIL: u64 = 3;

// Log levels
const LOG_DEBUG: u32 = 0;
const LOG_INFO: u32 = 1;
const LOG_WARN: u32 = 2;
const LOG_ERROR: u32 = 3;

/// Log entry
#[repr(C)]
struct LogEntry {
    timestamp: u64,
    source_pid: u32,
    level: u32,
    message: [u8; 256],
}

impl LogEntry {
    fn new() -> Self {
        LogEntry {
            timestamp: 0,
            source_pid: 0,
            level: 0,
            message: [0u8; 256],
        }
    }
}

/// Ring buffer for log entries
struct LogRingBuffer {
    buffer: [LogEntry; 16384],  // 4 MB total
    head: usize,                 // Next write position
    tail: usize,                 // Oldest entry
    count: usize,
}

impl LogRingBuffer {
    fn new() -> Self {
        LogRingBuffer {
            buffer: [LogEntry::new(); 16384],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    fn write(&mut self, entry: &LogEntry) {
        self.buffer[self.head] = LogEntry {
            timestamp: entry.timestamp,
            source_pid: entry.source_pid,
            level: entry.level,
            message: entry.message,
        };

        self.head = (self.head + 1) % 16384;

        if self.count < 16384 {
            self.count += 1;
        } else {
            self.tail = (self.tail + 1) % 16384;
        }
    }

    fn is_full(&self) -> bool {
        self.count >= 16384
    }
}

/// Print a string to serial console
fn print_str(s: &str) {
    for c in s.bytes() {
        unsafe {
            core::arch::x86_64::asm!("out al, dx", in("al") c, in("dx") 0x3F8u16);
        }
    }
}

/// Print a log entry to serial
fn print_log_entry(entry: &LogEntry) {
    // Level prefix
    match entry.level {
        LOG_DEBUG => print_str("[DEBUG] "),
        LOG_INFO => print_str("[INFO]  "),
        LOG_WARN => print_str("[WARN]  "),
        LOG_ERROR => print_str("[ERROR] "),
        _ => print_str("[?]     "),
    }

    // Source PID
    print_str("PID ");
    print_u32(entry.source_pid);
    print_str(" | ");

    // Message (null-terminated)
    let mut i = 0;
    while i < 256 && entry.message[i] != 0 {
        unsafe {
            core::arch::x86_64::asm!("out al, dx", in("al") entry.message[i], in("dx") 0x3F8u16);
        }
        i += 1;
    }

    print_str("\n");
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

/// Allocate a port via syscall
unsafe fn allocate_port() -> u32 {
    let result: u64;
    core::arch::x86_64::asm!(
        "syscall",
        inout("rax") 1u64 => result,  // SYS_PORT_ALLOCATE = 1
    );
    result as u32
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

/// Get current time via syscall
unsafe fn sys_time() -> u64 {
    let result: u64;
    core::arch::x86_64::asm!(
        "syscall",
        inout("rax") 10u64 => result,  // SYS_TIME = 10
    );
    result
}

/// Main entry point
#[no_mangle]
pub extern "C" fn _start() -> ! {
    print_str("[log] log_server started (PID 2)\n");

    unsafe {
        let log_port = allocate_port();
        print_str("[log] Allocated port ");
        print_u32(log_port);
        print_str(" for logging\n");

        let mut buffer = LogRingBuffer::new();
        print_str("[log] Ready for log messages\n");

        // Main loop
        loop {
            let mut msg = [0u64; 8];
            let result = recv_message(log_port, &mut msg);

            if result == 0 {
                match msg[0] {
                    LOG_WRITE => {
                        // Parse log message: [LOG_WRITE, timestamp, level, pid, ...]
                        let entry = LogEntry {
                            timestamp: msg[1],
                            source_pid: msg[3] as u32,
                            level: msg[2] as u32,
                            message: [0u8; 256],  // TODO: copy from msg
                        };

                        buffer.write(&entry);
                        print_log_entry(&entry);
                    }
                    LOG_READ_TAIL => {
                        print_str("[log] Tail read requested\n");
                    }
                    LOG_FLUSH => {
                        print_str("[log] Flush requested\n");
                    }
                    _ => {
                        print_str("[log] Unknown message type: ");
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
    print_str("[log] PANIC: ");
    if let Some(s) = info.payload().downcast_ref::<&str>() {
        print_str(s);
    }
    print_str("\n");

    loop {
        core::arch::x86_64::hlt();
    }
}

