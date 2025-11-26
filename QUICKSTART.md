# GBSD Developer Quick Start - Phase 1 Complete

**Last Updated**: November 25, 2025  
**Current Phase**: Phase 1 ✅ Complete - Ready for Phase 2  
**Status**: All systems go for Phase 2 development

---

## TL;DR - What Happened

Phase 1 of GBSD microkernel OS implementation is complete:

✅ Kernel with 10 syscalls  
✅ IPC system (ports + capabilities)  
✅ Three bootstrap services (stubs)  
✅ All compiles successfully  

**Next**: Expand services and test in QEMU

---

## Project Structure

```
/home/ssb/Code/gbsd/
├── kernel/                    # Microkernel source
│   └── src/
│       ├── error.rs          # Error definitions
│       ├── globals.rs        # Global kernel state
│       ├── ipc.rs            # IPC implementation
│       ├── syscall.rs        # Syscall dispatcher (10 syscalls)
│       └── arch/
│           ├── x86_64/       # x86_64 support
│           └── arm64/        # ARM64 stub
│
├── servers/                   # Userspace microservices
│   ├── init_server/          # PID 1, bootstrap
│   ├── log_server/           # Centralized logging
│   └── scheduler_server/     # Task scheduling
│
├── libgbsd/                   # Common library
│   └── src/lib.rs            # Syscall wrappers
│
├── docs/                      # Documentation
│   ├── ANALYSIS_AND_SOLUTIONS.md
│   ├── ARCHITECTURE_GUIDE.md
│   ├── IMPLEMENTATION_GUIDE.md
│   └── SOLUTIONS_AND_RECOMMENDATIONS.md
│
├── PHASE1_COMPLETE.md         # Phase 1 summary
├── PHASE2_SPECIFICATION.md    # Phase 2 detailed spec
└── IMPLEMENTATION_PROGRESS.md # Status tracking
```

---

## Quick Commands

### Build Everything
```bash
cd /home/ssb/Code/gbsd
cargo build --release        # Optimized build
cargo build                  # Debug build
cargo check                  # Fast check
```

### Build Individual Packages
```bash
cargo build -p kernel
cargo build -p init_server
cargo build -p log_server
cargo build -p scheduler_server
cargo build -p libgbsd
```

### Clean Build
```bash
cargo clean
cargo build --release
```

---

## What Works Right Now

### Kernel Features ✅
- 10 complete syscalls
- Error handling system
- IPC port management
- Capability-based security
- x86_64 exception handling

### Service Framework ✅
- Three bootstrap services created
- Cargo workspace configured
- All services compile

### Not Yet Implemented
- Service implementations (just stubs)
- Actual context switching
- Memory paging
- Running in QEMU

---

## Phase 2: What's Next

### Week 1-2: Expand init_server
```rust
// Current state: empty main loop
#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop { hlt() }
}

// Should become:
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 1. Allocate own port
    let init_port = port_allocate();
    
    // 2. Start log_server
    start_service("log_server", LOG_SERVER_ADDR);
    
    // 3. Start scheduler_server
    start_service("scheduler_server", SCHED_ADDR);
    
    // 4. Main event loop
    loop {
        let msg = port_receive(init_port);
        handle_message(&msg);
    }
}
```

### Week 2-3: Expand log_server
```rust
// Receive log messages and store in ring buffer
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let log_port = port_allocate();
    let mut buffer = RingBuffer::new();
    
    loop {
        let msg = port_receive(log_port);
        match msg[0] {
            LOG_WRITE => {
                buffer.append(&msg);
                print_to_serial(&msg);
            }
            _ => {}
        }
    }
}
```

### Week 3-4: Expand scheduler_server
```rust
// Implement round-robin scheduling
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut sched = Scheduler::new();
    let sched_port = port_allocate();
    
    loop {
        let msg = port_receive(sched_port);
        match msg[0] {
            MSG_TIMER_TICK => {
                let next_pid = sched.pick_next();
                syscall::sched_switch(next_pid);  // noreturn
            }
            MSG_TASK_YIELD => {
                sched.ready_queue.push_back(msg[1] as u32);
                // context switch...
            }
            _ => {}
        }
    }
}
```

---

## Key Files to Understand

### For IPC Understanding
- **Read First**: `kernel/src/ipc.rs` - Port and capability logic
- **Then**: `libgbsd/src/lib.rs` - Syscall wrappers
- **Reference**: `docs/ARCHITECTURE_GUIDE.md` § 5

### For Service Development
- **Template**: `servers/init_server/src/main.rs`
- **Spec**: `PHASE2_SPECIFICATION.md`
- **Guide**: `docs/IMPLEMENTATION_GUIDE.md` § 3

### For Syscall Details
- **Reference**: `kernel/src/syscall.rs` - All 10 syscalls
- **Spec**: `docs/ARCHITECTURE_GUIDE.md` § 3

---

## Development Workflow

### 1. Make a change
```bash
# Edit source file
vim servers/init_server/src/main.rs
```

### 2. Check it compiles
```bash
cargo check -p init_server
```

### 3. Build
```bash
cargo build -p init_server
```

### 4. Test in QEMU (Phase 2+)
```bash
qemu-system-x86_64 -kernel target/debug/kernel
```

### 5. Commit when ready
```bash
git add .
git commit -m "[service] description of changes"
```

---

## Important Syscall Patterns

### Allocate a Port
```rust
use libgbsd::x86_64_syscalls;

unsafe {
    let port = x86_64_syscalls::port_allocate();
    println!("Got port: {}", port);
}
```

### Send a Message
```rust
use libgbsd::{x86_64_syscalls, Message};

unsafe {
    let msg: Message = [1, 2, 3, 4, 5, 6, 7, 8];
    let result = x86_64_syscalls::port_send(port_id, &msg);
    if result != 0 {
        println!("Error: {:#x}", result);
    }
}
```

### Receive a Message
```rust
unsafe {
    let mut msg: Message = [0; 8];
    let result = x86_64_syscalls::port_receive(port_id, &mut msg);
    println!("Received: {:?}", msg);
}
```

### Get Current Time
```rust
unsafe {
    let now = x86_64_syscalls::sys_time();
    println!("Time: {}", now);
}
```

---

## Error Codes (from libgbsd)

```rust
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
```

---

## Testing

### Compilation Test
```bash
cargo check
# Should show: Finished
```

### Unit Tests
```bash
cargo test --lib
# Run all unit tests in kernel
```

### Build Test
```bash
cargo build --release
# Should produce binaries in target/release/
```

---

## Common Issues & Solutions

### "error: failed to read Cargo.toml"
**Cause**: Service not in workspace  
**Fix**: Add to root `Cargo.toml` members list

### "error: expected `;` found `,`"
**Cause**: Syntax error  
**Fix**: Check for missing semicolons

### "cannot find crate libgbsd"
**Cause**: Missing dependency in Cargo.toml  
**Fix**: Add `libgbsd` to dependencies

### Compilation hangs
**Cause**: Long linking time  
**Fix**: Use `cargo check` for quick syntax check only

---

## Documentation Map

| Need | Document |
|------|----------|
| Overview | `PHASE1_COMPLETE.md` |
| Phase 2 Tasks | `PHASE2_SPECIFICATION.md` |
| Architecture | `docs/ARCHITECTURE_GUIDE.md` |
| Implementation | `docs/IMPLEMENTATION_GUIDE.md` |
| Code Examples | `docs/SOLUTIONS_AND_RECOMMENDATIONS.md` |
| Progress Tracking | `IMPLEMENTATION_PROGRESS.md` |

---

## Team Checklist for Phase 2

### Before Starting
- [ ] Read `PHASE2_SPECIFICATION.md`
- [ ] Review `docs/IMPLEMENTATION_GUIDE.md` § 3-4
- [ ] Understand IPC from `docs/ARCHITECTURE_GUIDE.md` § 5
- [ ] Set up development environment

### During Development
- [ ] Make commits frequently
- [ ] Test with `cargo check` after each change
- [ ] Build with `cargo build` periodically
- [ ] Update progress in `IMPLEMENTATION_PROGRESS.md`
- [ ] Document any design decisions

### Before Submission
- [ ] All code compiles
- [ ] No warnings
- [ ] IPC tests pass
- [ ] Services boot without panic
- [ ] Update progress document

---

## Getting Help

### Code Questions
- Check: `docs/ARCHITECTURE_GUIDE.md` (design)
- Check: `docs/IMPLEMENTATION_GUIDE.md` (implementation)
- Check: `docs/SOLUTIONS_AND_RECOMMENDATIONS.md` (code examples)

### Syscall Questions
- Reference: `kernel/src/syscall.rs` (implementation)
- Reference: `libgbsd/src/lib.rs` (wrappers)
- Reference: `docs/ARCHITECTURE_GUIDE.md` § 3 (specification)

### Design Questions
- Reference: `docs/ANALYSIS_AND_SOLUTIONS.md` (strategic)
- Reference: `docs/ARCHITECTURE_GUIDE.md` (technical)
- Reference: `PHASE2_SPECIFICATION.md` (for Phase 2)

---

## Quick Reference

### Syscall Numbers (from libgbsd)
```rust
SYS_PORT_ALLOCATE = 1
SYS_PORT_SEND = 2
SYS_PORT_RECEIVE = 3
SYS_VM_ALLOCATE = 4
SYS_VM_DEALLOCATE = 5
SYS_CAP_MOVE = 6
SYS_SCHED_SPAWN = 7
SYS_SCHED_YIELD = 8
SYS_SCHED_SWITCH = 9
SYS_TIME = 10
```

### Capability Rights (from libgbsd)
```rust
CAP_SEND = 1 << 0
CAP_RECEIVE = 1 << 1
CAP_DESTROY = 1 << 2
CAP_DERIVE = 1 << 3
CAP_READ = 1 << 4
CAP_WRITE = 1 << 5
CAP_EXECUTE = 1 << 6
```

### Message Format
```rust
pub type Message = [u64; 8];  // 64 bytes total
```

---

## Ready to Start? 

1. Read: `PHASE2_SPECIFICATION.md`
2. Review: Code in `servers/init_server/src/main.rs`
3. Expand: Service stubs per specification
4. Test: With `cargo build`
5. Commit: Frequently with clear messages

**Next Phase**: Ready for immediate development!

---

**Status**: 🟢 Phase 1 Complete - Phase 2 Ready  
**Updated**: November 25, 2025  
**Next Review**: After Phase 2 services are implemented


