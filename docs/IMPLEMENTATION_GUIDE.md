# GBSD Implementation Guide

**Step-by-Step Guide to Implementing GBSD Services and Components**

---

## Table of Contents

1. [Development Environment Setup](#1-development-environment-setup)
2. [Kernel Development](#2-kernel-development)
3. [Writing Your First Service](#3-writing-your-first-service)
4. [IPC Communication](#4-ipc-communication)
5. [Testing & Debugging](#5-testing--debugging)
6. [Performance Optimization](#6-performance-optimization)
7. [Common Pitfalls & Solutions](#7-common-pitfalls--solutions)

---

## 1. Development Environment Setup

### 1.1 Install Dependencies

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    curl \
    qemu-system-x86 \
    grub-common \
    xorriso \
    mtools

# macOS (with Homebrew)
brew install qemu grub xorriso
```

### 1.2 Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Add nightly and necessary components
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
```

### 1.3 Project Structure

```bash
cd /path/to/gbsd
cargo build --release
```

---

## 2. Kernel Development

### 2.1 Understanding Kernel Layers

```
┌─────────────────────────────────────┐
│  User Code Entry                     │
│  syscall instruction                 │
└────────────┬────────────────────────┘
             │
┌────────────▼────────────────────────┐
│  Syscall Dispatcher                  │
│  kernel/src/syscall.rs              │
│  → Routes to specific handler        │
└────────────┬────────────────────────┘
             │
┌────────────▼────────────────────────┐
│  Syscall Handlers                    │
│  sys_port_allocate(),               │
│  sys_port_send(),                   │
│  sys_vm_allocate(),                 │
│  etc.                               │
└────────────┬────────────────────────┘
             │
┌────────────▼────────────────────────┐
│  Kernel State Management             │
│  kernel/src/globals.rs              │
│  KERNEL_STATE, CapabilityTable      │
│  ProcessTable, etc.                 │
└─────────────────────────────────────┘
```

### 2.2 Adding a Syscall

**Step 1**: Define the syscall number in `kernel/src/syscall.rs`

```rust
// kernel/src/syscall.rs
pub const SYS_PORT_ALLOCATE: u64 = 1;
pub const SYS_PORT_SEND: u64 = 2;
pub const SYS_PORT_RECEIVE: u64 = 3;
pub const SYS_VM_ALLOCATE: u64 = 4;
pub const SYS_VM_DEALLOCATE: u64 = 5;
pub const SYS_CAP_MOVE: u64 = 6;
pub const SYS_SCHED_SPAWN: u64 = 7;
pub const SYS_SCHED_YIELD: u64 = 8;
pub const SYS_SCHED_SWITCH: u64 = 9;
pub const SYS_TIME: u64 = 10;  // New: get current time
```

**Step 2**: Implement the syscall handler

```rust
// kernel/src/syscall.rs
fn sys_time() -> u64 {
    // Read from x86 TSC (Time Stamp Counter)
    unsafe { x86_64::registers::model_specific::Msr::new(0x10).read() }
}
```

**Step 3**: Add to dispatcher

```rust
// kernel/src/syscall.rs
pub extern "C" fn syscall_entry(frame: &mut InterruptFrame) {
    match frame.rax {
        1 => frame.rax = sys_port_allocate(frame),
        2 => frame.rax = sys_port_send(frame.rdi, frame.rsi, frame.rdx),
        // ... existing syscalls ...
        10 => frame.rax = sys_time(),  // New
        _ => frame.rax = ERROR_INVALID_SYSCALL,
    }
}
```

**Step 4**: Create userspace wrapper (libgbsd)

```c
// libgbsd/include/gbsd/syscall.h
static inline uint64_t sys_time(void) {
    register uint64_t result asm("rax");
    asm volatile("syscall" : "=r" (result) : "a" (10));
    return result;
}

// Usage in user code:
uint64_t now = sys_time();
```

### 2.3 Testing the Syscall

```rust
// tests/syscall_test.rs
#[test]
fn test_sys_time() {
    let t1 = sys_time();
    thread::sleep(Duration::from_millis(10));
    let t2 = sys_time();
    
    assert!(t2 > t1);  // Time should advance
}
```

---

## 3. Writing Your First Service

### 3.1 Service Template

```rust
// servers/example_server/src/main.rs
#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use gbsd::{syscall, port, message};

/// Service ID (assigned by init_server)
const SERVICE_ID: u32 = 100;

/// Message types
const MSG_ECHO: u64 = 1;
const MSG_SHUTDOWN: u64 = 2;

/// Main event loop
fn main() {
    // Allocate our service port
    let service_port = syscall::port_allocate()
        .expect("Failed to allocate port");
    
    println!("[example_server] Allocated port {}", service_port);
    
    // Main loop: receive and handle messages
    loop {
        match syscall::port_receive(service_port) {
            Ok(msg) => handle_message(msg),
            Err(e) => eprintln!("[example_server] Receive error: {:?}", e),
        }
    }
}

/// Handle incoming messages
fn handle_message(msg: &message::Message) {
    match msg[0] {
        MSG_ECHO => {
            // Echo back the message
            println!("[example_server] ECHO: {:?}", msg);
            
            let reply = [
                msg[0],  // Echo back command
                msg[1],  // Echo back data
                0,       // Success
                0, 0, 0, 0, 0,
            ];
            
            if let Err(e) = syscall::port_send(msg[4] as u32, &reply, 8) {
                eprintln!("[example_server] Send error: {:?}", e);
            }
        }
        MSG_SHUTDOWN => {
            println!("[example_server] Shutting down");
            core::arch::x86_64::hlt();  // Halt
        }
        _ => {
            eprintln!("[example_server] Unknown message: {}", msg[0]);
        }
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    eprintln!("[example_server] PANIC: {:?}", info);
    loop {
        core::arch::x86_64::hlt();
    }
}
```

### 3.2 Service Cargo.toml

```toml
# servers/example_server/Cargo.toml
[package]
name = "example_server"
version = "0.1.0"
edition = "2021"

[dependencies]
gbsd = { path = "../../libgbsd" }

[profile.release]
opt-level = "z"
lto = true
panic = "abort"
```

### 3.3 Registering with init_server

Create a service descriptor:

```toml
# /etc/services.toml
[[services]]
name = "example_server"
binary = "/bin/example_server"
auto_start = true
priority = 50
max_instances = 1

[[services.ports]]
name = "main"
capabilities = ["send", "receive"]
```

init_server reads this config and spawns services in order.

---

## 4. IPC Communication

### 4.1 Request/Reply Pattern (Synchronous)

**Client Side**:
```rust
// Create a reply port (one-time use)
let reply_port = port_allocate()?;

// Prepare request message
let request = [
    OP_READ,           // Command
    fd,                // File descriptor
    buffer_addr,       // Where to read
    4096,              // How many bytes
    reply_port as u64, // Where to send reply
    0, 0, 0,
];

// Send request to server
port_send(vfs_server_port, &request, 8)?;

// Wait for reply (blocking)
let reply = port_receive(reply_port)?;

// Check result
if reply[1] == 0 {  // No error
    let bytes_read = reply[0] as usize;
    println!("Read {} bytes", bytes_read);
} else {
    eprintln!("Server error: {:#x}", reply[1]);
}
```

**Server Side**:
```rust
// Main loop
loop {
    let request = port_receive(server_port)?;
    
    match request[0] {
        OP_READ => {
            let fd = request[1] as u32;
            let buf_addr = request[2];
            let len = request[3] as usize;
            let reply_port = request[4] as u32;
            
            // Perform operation
            let result = vfs_read(fd, buf_addr, len);
            
            // Send reply
            let reply = [
                result as u64,  // bytes_read
                0,              // error code (0 = success)
                0, 0, 0, 0, 0, 0,
            ];
            
            port_send(reply_port, &reply, 8)?;
        }
        _ => {}
    }
}
```

### 4.2 Fire-and-Forget Pattern (Asynchronous)

**Sender**:
```rust
let log_message = [
    LOG_WRITE,
    log_level,
    timestamp,
    pid,
    0, 0, 0, 0,
];

// Just send, don't wait for reply
port_send(log_server_port, &log_message, 8)?;
// Continue immediately
```

**Receiver**:
```rust
loop {
    let msg = port_receive(log_port)?;
    
    match msg[0] {
        LOG_WRITE => {
            let level = msg[1];
            let timestamp = msg[2];
            let pid = msg[3];
            
            // Log to persistent storage
            write_log_entry(level, timestamp, pid);
            // No reply sent
        }
        _ => {}
    }
}
```

### 4.3 Capability-Protected Communication

```rust
// Only vfs_server can receive on vfs_port
let vfs_port = port_allocate()?;

// Grant only SEND right to clients (read-only)
let client_cap = cap_move(vfs_port, client_pid, CAP_SEND)?;

// Client can now only SEND to vfs_server
// (Cannot RECEIVE or modify)
port_send(client_cap, &request, 8)?;  // OK
let msg = port_receive(client_cap)?;    // ERROR: No rights

// Revoke access if needed
cap_revoke(client_cap)?;
```

---

## 5. Testing & Debugging

### 5.1 Serial Console Output

```rust
// Print to serial (for debugging in QEMU)
println!("[my_service] Debug message");

// Redirects to QEMU `-serial mon:stdio` output
```

**Run with Serial**:
```bash
qemu-system-x86_64 \
    -cdrom gbsd.iso \
    -m 2G \
    -serial mon:stdio \
    -monitor telnet:127.0.0.1:55555,server,nowait
```

### 5.2 Gdb Debugging

```bash
# Terminal 1: Run QEMU with gdb server
qemu-system-x86_64 \
    -cdrom gbsd.iso \
    -s \
    -S \
    -m 2G

# Terminal 2: Connect gdb
gdb ./target/release/kernel

(gdb) target remote :1234
(gdb) break _start
(gdb) continue
(gdb) stepi  # Step one instruction
(gdb) info registers
(gdb) x/16x $rsp  # Examine stack
```

### 5.3 Unit Tests

```rust
// servers/my_server/src/lib.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_parsing() {
        let msg = [123, 456, 789, 0, 0, 0, 0, 0];
        
        let parsed = parse_message(&msg).unwrap();
        assert_eq!(parsed.command, 123);
        assert_eq!(parsed.arg1, 456);
    }

    #[test]
    fn test_error_handling() {
        let result = handle_invalid_command(999);
        assert!(result.is_err());
    }
}
```

**Run tests**:
```bash
cargo test --lib
```

### 5.4 Integration Tests

```bash
# tests/integration_test.sh
#!/bin/bash
set -e

echo "Building GBSD..."
cargo build --release

echo "Creating ISO..."
grub-mkrescue -o test.iso iso/

echo "Running in QEMU..."
timeout 30 qemu-system-x86_64 \
    -cdrom test.iso \
    -m 2G \
    -nographic \
    -serial mon:stdio \
    | tee test-output.log

echo "Checking results..."
if grep -q "All services started" test-output.log; then
    echo "✅ Test PASSED"
    exit 0
else
    echo "❌ Test FAILED"
    exit 1
fi
```

**Run**:
```bash
chmod +x tests/integration_test.sh
./tests/integration_test.sh
```

---

## 6. Performance Optimization

### 6.1 Profiling Syscalls

```rust
// kernel/src/perf.rs
pub struct SyscallStats {
    pub call_count: u64,
    pub total_time_ns: u64,
    pub max_time_ns: u64,
    pub min_time_ns: u64,
}

pub static mut SYSCALL_STATS: [SyscallStats; 16] = [SyscallStats::default(); 16];

pub fn record_syscall(syscall_num: u64, duration_ns: u64) {
    unsafe {
        let stat = &mut SYSCALL_STATS[syscall_num as usize];
        stat.call_count += 1;
        stat.total_time_ns += duration_ns;
        stat.max_time_ns = stat.max_time_ns.max(duration_ns);
        stat.min_time_ns = stat.min_time_ns.min(duration_ns);
    }
}

pub fn print_stats() {
    unsafe {
        for (i, stat) in SYSCALL_STATS.iter().enumerate() {
            let avg = if stat.call_count > 0 {
                stat.total_time_ns / stat.call_count
            } else {
                0
            };
            println!("Syscall {} - avg: {} ns, max: {} ns, min: {} ns",
                     i, avg, stat.max_time_ns, stat.min_time_ns);
        }
    }
}
```

### 6.2 Measuring Context Switch Time

```rust
// servers/scheduler_server/src/main.rs
use std::time::Instant;

fn measure_context_switch_time() {
    let start = Instant::now();
    
    // Record TSC before
    let tsc_before = sys_time();
    
    // Yield
    sys_sched_yield();
    
    // Record TSC after
    let tsc_after = sys_time();
    
    let elapsed = start.elapsed();
    let tsc_delta = tsc_after - tsc_before;
    
    // TSC frequency (e.g., 2.4 GHz)
    let ns_per_cycle = 1_000_000_000 / TSC_FREQUENCY_HZ;
    let context_switch_ns = tsc_delta * ns_per_cycle;
    
    println!("Context switch: {} ns", context_switch_ns);
}
```

### 6.3 Zero-Copy IPC

```rust
// Instead of copying data through messages:
// Map shared memory between processes

let shared_region = vm_allocate(4096, VMF_SHARED)?;

// Grant access to another process
cap_move(shared_region_cap, other_pid, CAP_READ | CAP_WRITE)?;

// Both processes can now read/write the same memory
unsafe {
    *(shared_region as *mut u64) = 42;  // Writer
}

// Reader
unsafe {
    let value = *(shared_region as *const u64);  // 42
}
```

---

## 7. Common Pitfalls & Solutions

### Pitfall 1: Port Running Out of Memory

**Problem**: Port queue overflows
```rust
// ✗ BAD: Doesn't check queue size
port_send(busy_port, &msg, 8)?;  // Might fail with E_PORT_FULL
```

**Solution**: Implement backpressure
```rust
// ✓ GOOD: Handle queue full
match port_send(busy_port, &msg, 8) {
    Ok(()) => {}
    Err(Error::PortFull) => {
        // Retry with exponential backoff
        thread::sleep(Duration::from_millis(1));
        port_send(busy_port, &msg, 8)?;
    }
    Err(e) => return Err(e),
}
```

### Pitfall 2: Capability Revocation Race

**Problem**: Capability used after revocation
```rust
// ✗ RACE CONDITION
cap_revoke(cap)?;
port_send(cap, &msg, 8)?;  // Might succeed or fail unpredictably
```

**Solution**: Synchronize revocation
```rust
// ✓ GOOD: Wait for confirmation
cap_revoke(cap)?;
// Drain in-flight messages
let in_flight = drain_port_messages(cap);
println!("Revoked, {} messages dropped", in_flight);
```

### Pitfall 3: Deadlock with Synchronous IPC

**Problem**: Circular wait
```rust
// ✗ DEADLOCK
// Service A: waiting for reply from B on reply_port
// Service B: blocked trying to send to A
```

**Solution**: Use timeout or async patterns
```rust
// ✓ GOOD: Timeout on receive
match port_receive_timeout(reply_port, Duration::from_secs(5)) {
    Ok(reply) => handle_reply(reply),
    Err(Timeout) => {
        eprintln!("Timeout waiting for reply");
        // Fallback or error handling
    }
}

// Or use async fire-and-forget
port_send(server_port, &msg, 8)?;  // Don't wait
// Continue with other work
```

### Pitfall 4: Memory Fragmentation

**Problem**: Heap becomes fragmented
```rust
// ✗ Many allocations → fragmentation
for i in 0..10000 {
    let vec = vec![0u8; random_size()];
    // Later freed in random order
}
```

**Solution**: Use appropriate allocator
```rust
// ✓ Use slab allocator for fixed-size objects
let pool = SlabAllocator::new(sizeof(Message), 1000);

// or use bump allocator + arena clearing
arena.allocate_many(messages)?;
arena.reset();  // Free all at once
```

### Pitfall 5: Race Condition in Port Access

**Problem**: Multiple threads access same port
```rust
// ✗ RACE CONDITION
let msg = port_receive(port)?;  // Thread 1
// Another thread also calls port_receive(port)?
// Only one will get the message
```

**Solution**: Use per-thread or exclusive ports
```rust
// ✓ GOOD: Each thread has own receive port
let my_port = port_allocate()?;
cap_move(shared_port, my_process, CAP_SEND)?;

// Server sends specifically to each thread's port
let thread_ports = vec![thread1_port, thread2_port, ...];
for (i, msg) in messages.iter().enumerate() {
    port_send(thread_ports[i % thread_ports.len()], msg, 8)?;
}
```

### Pitfall 6: Panic in Service

**Problem**: Service crashes without notification
```rust
// ✗ BAD: Service panics
panic!("Unexpected state");  // Kernel terminates service
// init_server doesn't know why it crashed
```

**Solution**: Explicit error handling + logging
```rust
// ✓ GOOD: Log and notify
fn handle_error(e: Error) {
    eprintln!("[my_service] Error: {:?}", e);
    
    // Send error notification to init_server
    let error_msg = [
        MSG_SERVICE_ERROR,
        e.code() as u64,
        0, 0, 0, 0, 0, 0,
    ];
    let _ = port_send(init_server_port, &error_msg, 8);
    
    // Clean up state and continue (or graceful shutdown)
}
```

---

## Checklist for New Services

Before deploying a new service, ensure:

- [ ] Service compiles without warnings
- [ ] All syscall results checked for errors
- [ ] No `unwrap()` calls (except in test code)
- [ ] No `panic!()` calls (except panic_handler)
- [ ] Message format documented
- [ ] Ports allocated and freed correctly
- [ ] Capabilities properly checked
- [ ] Timeouts implemented for blocking operations
- [ ] Logging statements added at key points
- [ ] Integration tests pass
- [ ] Performance profiled (< 1% overhead expected)

---

## Next Steps

1. **Set up development environment** (Section 1)
2. **Build kernel with sample syscall** (Section 2)
3. **Write init_server** (Section 3)
4. **Test IPC communication** (Section 4)
5. **Debug with QEMU** (Section 5)
6. **Optimize performance** (Section 6)


