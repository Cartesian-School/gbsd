# GBSD Phase 2: Bootstrap Services Implementation Spec

**Timeline**: Weeks 2-4  
**Status**: Ready to implement  
**Owner**: Development Team  

---

## Phase 2 Overview

Phase 2 focuses on expanding the three bootstrap services that were created as stubs in Phase 1:

1. **init_server** (PID 1) - Bootstrap and service management
2. **log_server** - Centralized logging  
3. **scheduler_server** - Preemptive scheduling

By end of Phase 2, we want:
- ✅ init_server can boot and load other services
- ✅ log_server can receive and store log messages
- ✅ scheduler_server can manage ready queue
- ✅ IPC communication working between services
- ✅ First userspace output visible

---

## init_server Specification

### Purpose
- PID 1 (first userspace process)
- Bootstrap system and manage service lifecycle
- Allocate ports for other services
- Handle service failures and restarts

### Implementation Steps

**Step 1: Syscall Wrappers**
```rust
use libgbsd::*;

unsafe fn allocate_port() -> u32 {
    x86_64_syscalls::port_allocate() as u32
}

unsafe fn send_message(port: u32, msg: &Message) -> u64 {
    x86_64_syscalls::port_send(port, msg)
}

unsafe fn recv_message(port: u32, buf: &mut Message) -> u64 {
    x86_64_syscalls::port_receive(port, buf)
}
```

**Step 2: Service Registry**
```rust
struct ServiceDescriptor {
    name: [u8; 32],
    binary_addr: u64,  // Will be set by bootloader
    port: u32,
    pid: u32,
}

static mut SERVICES: [ServiceDescriptor; 10] = [/* ... */];
```

**Step 3: Main Event Loop**
```rust
fn main() -> ! {
    // 1. Allocate init_server's own port
    let init_port = allocate_port();
    
    // 2. Start log_server and scheduler_server
    start_log_server();
    start_scheduler_server();
    
    // 3. Main event loop - handle service messages
    loop {
        let mut msg = [0u64; 8];
        recv_message(init_port, &mut msg);
        
        match msg[0] {
            CMD_SERVICE_DIED => handle_service_death(msg[1] as u32),
            CMD_REBOOT => reboot(),
            _ => {}
        }
    }
}
```

**Step 4: Service Startup**
```rust
fn start_log_server() -> u32 {
    // 1. Allocate port for log_server
    let log_port = allocate_port();
    
    // 2. Spawn log_server process
    let pid = spawn_process(LOG_SERVER_ADDR, LOG_SERVER_STACK);
    
    // 3. Store in service table
    SERVICES[LOG_SERVICE_IDX].pid = pid;
    SERVICES[LOG_SERVICE_IDX].port = log_port;
    
    pid
}
```

### Messaging Protocol

**Service Status Request**
```
Message: [CMD_STATUS, service_id, 0, 0, 0, 0, 0, 0]
Reply:   [STATUS_OK, pid, uptime_ms, memory_used, 0, 0, 0, 0]
```

**Service Failure Notification**
```
Message: [CMD_SERVICE_DIED, pid, exit_code, 0, 0, 0, 0, 0]
Action:  Restart service with backoff delay
```

### Target Behavior
```
[init] init_server started (PID 1)
[init] Allocating port...
[init] Starting log_server...
[init] Starting scheduler_server...
[init] All services started
[init] Waiting for events...
```

---

## log_server Specification

### Purpose
- Receive log messages from all services
- Store in ring buffer (4 MB)
- Write to persistent storage (future)
- Provide log retrieval interface

### Data Structure

```rust
#[repr(C)]
struct LogEntry {
    timestamp: u64,        // monotonic time in milliseconds
    source_pid: u32,       // Which service logged this
    level: u32,            // DEBUG=0, INFO=1, WARN=2, ERROR=3
    message: [u8; 256],    // Null-terminated string
}

struct LogRingBuffer {
    buffer: [LogEntry; 16384],  // 4 MB total
    head: usize,                // Next write position
    tail: usize,                // Oldest entry
    count: usize,
}
```

### Syscall Wrappers

```rust
fn recv_log_message(log_port: u32) -> Message {
    let mut msg = [0u64; 8];
    unsafe {
        x86_64_syscalls::port_receive(log_port, &mut msg);
    }
    msg
}

fn write_log_entry(entry: &LogEntry) {
    // Add to ring buffer
    // Rotate buffer if full
}
```

### Main Loop

```rust
fn main() -> ! {
    let log_port = allocate_port();
    let mut buffer = LogRingBuffer::new();
    
    loop {
        let msg = recv_log_message(log_port);
        
        match msg[0] {
            LOG_WRITE => {
                let entry = parse_log_message(&msg);
                buffer.write(&entry);
                print_to_serial(&entry);
            }
            LOG_READ_TAIL => {
                // Return last N entries
            }
            LOG_FLUSH => {
                // Write to disk (future)
            }
            _ => {}
        }
    }
}
```

### Target Output
```
[log] log_server started (PID 2)
[log] Allocated port for logging
[log] Ready for log messages
```

---

## scheduler_server Specification

### Purpose
- Implement preemptive round-robin scheduling
- Maintain ready queue and sleeping task map
- Handle timer interrupts
- Perform context switching

### Data Structures

```rust
struct Scheduler {
    ready_queue: [u32; 256],     // PIDs of ready tasks
    queue_head: usize,
    queue_tail: usize,
    queue_size: usize,
    
    sleeping: [(u32, u64); 256], // (PID, wake_time)
    sleeping_count: usize,
    
    current_pid: u32,
    last_switch_time: u64,
}
```

### Main Loop

```rust
fn scheduler_loop(sched: &mut Scheduler, sched_port: u32) -> ! {
    loop {
        // Check for woken tasks
        let now = current_time();
        wake_expired_tasks(sched, now);
        
        // Receive event (non-blocking for now, or poll)
        if let Ok(msg) = try_recv_message(sched_port) {
            match msg[0] {
                MSG_TIMER_TICK => handle_timer_tick(sched),
                MSG_TASK_YIELD => handle_yield(sched, msg[1] as u32),
                MSG_TASK_SLEEP => {
                    let pid = msg[1] as u32;
                    let duration = msg[2];
                    sleep_task(sched, pid, duration);
                }
                _ => {}
            }
        }
        
        // Pick next task
        if let Some(next_pid) = pick_next_task(sched) {
            context_switch(next_pid);  // noreturn - returns with new process
        }
    }
}
```

### Round-Robin Logic

```rust
fn handle_timer_tick(sched: &mut Scheduler) {
    // Put current task back in queue
    if sched.current_pid != 0 {
        sched.enqueue(sched.current_pid);
    }
    
    // Pick next from queue
    if let Some(next) = sched.dequeue() {
        sched.current_pid = next;
        context_switch(next);
    }
}

fn handle_yield(sched: &mut Scheduler, yielding_pid: u32) {
    // Put yielding task back in queue
    sched.enqueue(yielding_pid);
    
    // Pick next
    if let Some(next) = sched.dequeue() {
        sched.current_pid = next;
        context_switch(next);
    }
}

fn sleep_task(sched: &mut Scheduler, pid: u32, duration: u64) {
    let wake_time = current_time() + duration;
    sched.sleeping[sched.sleeping_count] = (pid, wake_time);
    sched.sleeping_count += 1;
    
    // Switch to next ready task
    if let Some(next) = sched.dequeue() {
        sched.current_pid = next;
        context_switch(next);
    }
}

fn wake_expired_tasks(sched: &mut Scheduler, now: u64) {
    let mut i = 0;
    while i < sched.sleeping_count {
        if sched.sleeping[i].1 <= now {
            sched.enqueue(sched.sleeping[i].0);
            sched.sleeping.swap(i, sched.sleeping_count - 1);
            sched.sleeping_count -= 1;
        } else {
            i += 1;
        }
    }
}
```

### Target Output
```
[scheduler] scheduler_server started (PID 3)
[scheduler] Allocated scheduler port
[scheduler] Ready queue: empty
[scheduler] Waiting for events...
```

---

## IPC Communication Flow

### init_server → log_server

```
init_server allocates port for log_server
         ↓
init_server sends: [MSG_START, log_server_addr, ...]
         ↓
kernel spawns log_server process (PID 2)
         ↓
log_server receives and starts
         ↓
log_server allocates its own port
         ↓
log_server sends reply: [MSG_READY, port_id, ...]
```

### Any Service → log_server

```
Service prepares log message:
msg = [LOG_WRITE, timestamp, level, pid, ...]

Service sends via syscall:
syscall(SYS_PORT_SEND, log_port, &msg)
         ↓
kernel routes to log_server's port
         ↓
log_server receives in message loop
         ↓
log_server appends to ring buffer
         ↓
log_server outputs to serial console
```

---

## Testing During Phase 2

### Unit Tests
```rust
#[test]
fn test_service_startup() {
    // Verify services allocate ports
}

#[test]
fn test_log_ring_buffer() {
    // Verify ring buffer wraps correctly
}

#[test]
fn test_scheduler_ready_queue() {
    // Verify queue operations
}
```

### Integration Tests
```bash
# Boot in QEMU and verify output
qemu-system-x86_64 -kernel kernel -serial mon:stdio

# Expected output:
# [init] init_server started (PID 1)
# [init] Starting services...
# [log] log_server started (PID 2)
# [scheduler] scheduler_server started (PID 3)
```

### Manual Testing
1. Boot system in QEMU
2. Verify serial output shows all services starting
3. Send test message to log_server via IPC
4. Verify log output appears on serial console
5. Verify scheduler picks tasks correctly

---

## Compilation Requirements

All services must compile with:
```bash
cargo build -p init_server
cargo build -p log_server
cargo build -p scheduler_server
```

No warnings allowed. Use `#[deny(warnings)]` if necessary.

---

## Success Criteria

### init_server
- ✅ Compiles without errors
- ✅ Allocates port via syscall
- ✅ Can start other services
- ✅ Runs main event loop without panic

### log_server
- ✅ Compiles without errors
- ✅ Allocates port via syscall
- ✅ Receives messages via IPC
- ✅ Stores in ring buffer
- ✅ Outputs to serial console

### scheduler_server
- ✅ Compiles without errors
- ✅ Allocates port via syscall
- ✅ Maintains ready queue
- ✅ Handles task yielding
- ✅ Implements round-robin

### Integration
- ✅ All three services start successfully
- ✅ Services can communicate via IPC
- ✅ No panics or crashes
- ✅ Boots in QEMU with expected output

---

## Resources & References

- Architecture Guide: `/home/ssb/Code/gbsd/docs/ARCHITECTURE_GUIDE.md` § 5 (IPC)
- Implementation Guide: `/home/ssb/Code/gbsd/docs/IMPLEMENTATION_GUIDE.md` § 3-4
- Code Templates: `/home/ssb/Code/gbsd/docs/SOLUTIONS_AND_RECOMMENDATIONS.md` § 2

---

**Document Version**: 1.0  
**Created**: November 25, 2025  
**Status**: Ready for implementation  


