# GBSD Microkernel Architecture Guide

**GuardBSD - Security-First Microkernel Design**

---

## Table of Contents

1. [Microkernel Philosophy](#1-microkernel-philosophy)
2. [Kernel Design](#2-kernel-design)
3. [System Call Interface](#3-system-call-interface)
4. [Capability-Based Security](#4-capability-based-security)
5. [Inter-Process Communication (IPC)](#5-inter-process-communication-ipc)
6. [Memory Management](#6-memory-management)
7. [Task Scheduling](#7-task-scheduling)
8. [Fault Isolation](#8-fault-isolation)
9. [Architecture Specifics (x86_64, ARM64)](#9-architecture-specifics-x86_64-arm64)
10. [Common Patterns & Best Practices](#10-common-patterns--best-practices)

---

## 1. Microkernel Philosophy

### Why Microkernel?

Traditional **monolithic kernels** (Linux, Windows) include:
- Filesystems
- Device drivers
- Network stack
- Graphics subsystem

**Result**: Thousands of lines of kernel code → huge TCB → security vulnerabilities

**GBSD Philosophy**: Minimize privilege, maximize isolation

```
┌──────────────────────────────────────────┐
│  Monolithic Kernel (Million Lines)       │
│  • Filesystems                           │
│  • Drivers                               │
│  • Network                               │
│  • Graphics                              │
│  • Security gap: bug anywhere = full    │
│    system compromise                    │
└──────────────────────────────────────────┘

vs.

┌──────────────────────────────────────────┐
│  GBSD Microkernel (< 8 KB)              │
│  • Message passing                       │
│  • Memory protection                     │
│  • Task switching                        │
│  • Interrupt routing                     │
│  • Security: Each service isolated      │
└──────────────────────────────────────────┘
         ↓ (capability-based IPC)
┌──────────────────────────────────────────┐
│  Userspace Servers                       │
│  • vfs_server (filesystem)              │
│  • netstack_server (TCP/IP)             │
│  • drm_server (GPU)                     │
│  • Any service can crash without        │
│    affecting others                     │
└──────────────────────────────────────────┘
```

### Principles

1. **Least Privilege**: Services run with minimal permissions
2. **Fault Isolation**: Crash of one service ≠ system crash
3. **Capability-Based**: All access mediated by capabilities (not ACLs)
4. **Message Passing**: No shared memory by default
5. **Transparency**: All inter-service communication visible

---

## 2. Kernel Design

### 2.1 Kernel Memory Layout (x86_64)

```
0xFFFFFFFF_FFFFFFFF ─────────────────────┐
                                         │
  Kernel Memory (1 GB)                   │
  ├─ Kernel Code & Data  0xFFFFFFFF_80000000  ← _start
  ├─ Kernel Heap        0xFFFFFFFF_90000000
  ├─ Kernel Stack       0xFFFFFFFF_FFF00000
  └─ Direct Map         0xFFFFFFFF_00000000  (all physical RAM)
                                         │
0xFFFFFFFF_00000000 ─────────────────────┘

0x00000000_00000000 ─────────────────────┐
                                         │
  User Memory (per-process)              │
  ├─ Code Segment       0x00000000_00001000
  ├─ Data Segment       0x00000000_00010000
  ├─ BSS                0x00000000_00020000
  ├─ Heap               0x00000000_10000000
  └─ Stack              0x00000000_7FFFFFFF  (grows down)
                                         │
0x00000000_00000000 ─────────────────────┘
```

### 2.2 Global Kernel State

```rust
// kernel/src/globals.rs
pub static KERNEL_STATE: SpinLock<KernelState> = SpinLock::new(KernelState {
    capability_table: Vec::new(),     // All capabilities
    port_table: Vec::new(),           // All ports (queues)
    process_table: Vec::new(),        // All processes
    current_process_id: 0,            // Currently running PID
    interrupt_handlers: [None; 256],  // IDT
});

pub struct KernelState {
    // Capability management
    capability_table: Vec<Capability>,
    
    // Port management (for IPC)
    port_table: Vec<Port>,
    
    // Process management
    process_table: Vec<Process>,
    current_process_id: ProcessID,
    
    // Interrupt/Exception handlers
    interrupt_handlers: [Option<fn()>; 256],
}

pub struct Capability {
    id: CapabilityID,
    owner_pid: ProcessID,
    rights_mask: u32,  // CAP_SEND | CAP_RECEIVE | CAP_DESTROY | ...
    revoked: bool,
}

pub struct Port {
    id: PortID,
    owner_pid: ProcessID,
    queue: VecDeque<Message>,  // Up to 128 messages
    max_size: usize,
    senders: Vec<ProcessID>,   // Who can send to this port?
}

pub struct Process {
    pid: ProcessID,
    memory_start: usize,
    memory_end: usize,
    page_table: PageTableRoot,
    context: TaskContext,
    state: ProcessState,      // Running, Ready, Sleeping, Dead
    capabilities: Vec<CapabilityID>,
}

pub enum ProcessState {
    Running,
    Ready,
    Sleeping { until: u64 },
    Dead,
}
```

### 2.3 Kernel Entry Points

1. **Bootstrap**: `_start()` (bootloader → kernel)
2. **Exception Handler**: `exception_handler()` (CPU exception)
3. **Interrupt Handler**: `interrupt_handler()` (external interrupt)
4. **Syscall Handler**: `syscall_handler()` (user syscall)

```rust
// kernel/src/main.rs
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 1. Initialize hardware
    arch::early_init();
    
    // 2. Set up paging
    memory::init_paging();
    
    // 3. Set up interrupts (IDT)
    arch::init_idt();
    
    // 4. Initialize allocators
    memory::init_allocators();
    
    // 5. Load init_server from boot modules
    process::load_init_server();
    
    // 6. Enter scheduler (never returns)
    schedule::kernel_loop()
}
```

---

## 3. System Call Interface

### 3.1 Syscall Convention (x86_64)

**Syscall Number**: `rax`  
**Arguments**: `rdi`, `rsi`, `rdx`, `rcx`, `r8`, `r9`  
**Return**: `rax` (0 = success, or error code)  

```asm
; User code (glibc wrapper)
mov rax, 1           ; syscall number: port_allocate
syscall
cmp rax, 0
je success
; handle error
```

### 3.2 Syscall Table

| # | Name | Args | Returns | Error |
|---|------|------|---------|-------|
| 1 | `port_allocate()` | — | u32 (port_id) | 0xFFFFFFFF |
| 2 | `port_send(port, msg, len)` | rdi, rsi, rdx | 0 | E_PORT_INVALID, E_PORT_FULL, E_NO_RIGHTS |
| 3 | `port_receive(port, buf)` | rdi, rsi | bytes_read | E_PORT_INVALID, E_NO_RIGHTS |
| 4 | `vm_allocate(hint, size, flags)` | rdi, rsi, rdx | u64 (addr) | E_NOMEM, E_INVAL |
| 5 | `vm_deallocate(addr, size)` | rdi, rsi | 0 | E_INVAL, E_NOT_OWNER |
| 6 | `cap_move(src, dst, rights)` | rdi, rsi, rdx | 0 | E_CAP_INVALID, E_NO_RIGHTS |
| 7 | `sched_spawn(entry, stack, name)` | rdi, rsi, rdx | u32 (pid) | E_NOMEM, E_INVAL |
| 8 | `sched_yield()` | — | 0 | — |
| 9 | `sched_switch(target_pid)` | rdi | — | E_INVAL, E_NO_RIGHTS |

### 3.3 Syscall Dispatcher

```rust
// kernel/src/syscall.rs
pub extern "C" fn syscall_entry(frame: &mut InterruptFrame) {
    match frame.rax {
        1 => frame.rax = sys_port_allocate(frame),
        2 => frame.rax = sys_port_send(frame.rdi, frame.rsi, frame.rdx),
        3 => frame.rax = sys_port_receive(frame.rdi, frame.rsi),
        4 => frame.rax = sys_vm_allocate(frame.rdi, frame.rsi, frame.rdx),
        5 => frame.rax = sys_vm_deallocate(frame.rdi, frame.rsi),
        6 => frame.rax = sys_cap_move(frame.rdi, frame.rsi, frame.rdx),
        7 => frame.rax = sys_sched_spawn(frame.rdi, frame.rsi, frame.rdx),
        8 => frame.rax = sys_sched_yield(frame),
        9 => frame.rax = sys_sched_switch(frame.rdi, frame),
        _ => frame.rax = ERROR_INVALID_SYSCALL,
    }
}
```

---

## 4. Capability-Based Security

### 4.1 What is a Capability?

A **capability** is an unforgeable token that grants rights to an object:

```
┌─────────────────────────────────────────┐
│  Capability = Handle + Rights            │
├─────────────────────────────────────────┤
│  CapabilityID (u32)                    │
│  + Rights Bitmask (u32)                 │
│     ├─ CAP_SEND       (bit 0)           │
│     ├─ CAP_RECEIVE    (bit 1)           │
│     ├─ CAP_DESTROY    (bit 2)           │
│     ├─ CAP_DERIVE     (bit 3)           │
│     └─ CAP_READ/WRITE (bits 4-5)       │
│  + Owner (ProcessID)                   │
│  + Revoked? (bool)                     │
└─────────────────────────────────────────┘
```

### 4.2 Capability Operations

#### Create
```rust
fn port_allocate() -> CapabilityID {
    let cap = Capability {
        id: next_cap_id(),
        owner_pid: current_pid,
        rights_mask: CAP_SEND | CAP_RECEIVE | CAP_DESTROY,
        revoked: false,
    };
    KERNEL_STATE.capability_table.push(cap);
    cap.id
}
```

#### Transfer
```rust
fn cap_move(src_cap: CapabilityID, dst_process: ProcessID, rights: u32) -> u64 {
    let mut cap = KERNEL_STATE.capability_table[src_cap];
    
    // Check if source process has this capability
    if current_pid != cap.owner_pid {
        return ERROR_NO_RIGHTS;
    }
    
    // Check if requested rights are a subset of current rights
    if (rights & cap.rights_mask) != rights {
        return ERROR_NO_RIGHTS;
    }
    
    // Transfer to destination
    let dst_process = &mut KERNEL_STATE.process_table[dst_process];
    dst_process.capabilities.push(src_cap);
    
    0  // success
}
```

#### Revoke
```rust
fn cap_revoke(cap: CapabilityID) {
    KERNEL_STATE.capability_table[cap].revoked = true;
}
```

### 4.3 Rights Enforcement

Before any operation, kernel checks rights:

```rust
fn sys_port_send(port_id: u32, msg: *const u64, len: usize) -> u64 {
    // 1. Find capability for this port
    let cap = find_capability(current_pid, port_id);
    if cap.is_none() {
        return ERROR_PORT_INVALID;
    }
    
    // 2. Check if CAP_SEND right is present
    if !cap.unwrap().has_right(CAP_SEND) {
        return ERROR_NO_RIGHTS;
    }
    
    // 3. Check if capability is revoked
    if cap.unwrap().revoked {
        return ERROR_CAP_INVALID;
    }
    
    // 4. Perform operation
    let port = &mut KERNEL_STATE.port_table[port_id];
    port.queue.push_back(Message::from_ptr(msg, len))?;
    
    0
}
```

### 4.4 Capability Quotas (Optional)

```rust
pub struct Process {
    // ...
    max_capabilities: usize,  // 128 per process
    capability_count: usize,
}

fn port_allocate() -> u64 {
    let proc = &mut KERNEL_STATE.process_table[current_pid];
    
    if proc.capability_count >= proc.max_capabilities {
        return ERROR_NO_RIGHTS;  // or new E_CAP_QUOTA_EXCEEDED
    }
    
    proc.capability_count += 1;
    // ... create capability
}
```

---

## 5. Inter-Process Communication (IPC)

### 5.1 Port-Based Messaging

**Port** = Message queue + capability control

```
┌─ Process A ────────────────────────────┐
│  cap_id=7 (CAP_SEND)                   │
│                                        │
│  port_send(7, [msg], 8)                │
└──────────────────┬─────────────────────┘
                   │
                   ↓ (kernel route)
┌─ Kernel ─────────┼──────────────────────┐
│  Port 7:                                │
│  ├─ queue: [msg]                        │
│  ├─ owner: Process B                    │
│  └─ rights: SEND | RECEIVE | DESTROY   │
└──────────────────┼──────────────────────┘
                   │
                   ↓
┌─ Process B ────────────────────────────┐
│  cap_id=7 (CAP_RECEIVE)                │
│                                        │
│  [msg] = port_receive(7, buf)          │
└────────────────────────────────────────┘
```

### 5.2 Message Format

```rust
pub type Message = [u64; 8];

// Example: File read request
let msg = [
    OP_READ,           // msg[0]: Operation code
    3,                 // msg[1]: File descriptor
    0x1000,            // msg[2]: Buffer address
    4096,              // msg[3]: Read size
    reply_port,        // msg[4]: Reply port capability
    0,                 // msg[5]: Reserved
    0,                 // msg[6]: Reserved
    0,                 // msg[7]: Reserved
];

port_send(vfs_port, msg, 8)?;
let response = port_receive(reply_port)?;
// response[0] = bytes_read
// response[1] = error code
```

### 5.3 Port Queue Management

```rust
pub struct Port {
    queue: VecDeque<Message>,
    max_size: usize,  // Default 128 messages
    waiting_threads: Vec<ProcessID>,
}

fn sys_port_receive(port_id: u32, buf: *mut u64) -> u64 {
    let port = &mut KERNEL_STATE.port_table[port_id];
    
    loop {
        // If message available, dequeue and return
        if let Some(msg) = port.queue.pop_front() {
            unsafe {
                core::ptr::copy_nonoverlapping(msg.as_ptr(), buf, 8);
            }
            return 8;
        }
        
        // Otherwise, block current process
        let current = current_pid();
        port.waiting_threads.push(current);
        KERNEL_STATE.process_table[current].state = ProcessState::Sleeping;
        
        // Yield to scheduler
        sys_sched_yield();
        // Wake up here when message arrives
    }
}

fn sys_port_send(port_id: u32, msg: &Message, len: usize) -> u64 {
    let port = &mut KERNEL_STATE.port_table[port_id];
    
    // Check if queue is full
    if port.queue.len() >= port.max_size {
        return ERROR_PORT_FULL;
    }
    
    port.queue.push_back(*msg);
    
    // Wake one waiting thread
    if let Some(waiter) = port.waiting_threads.pop() {
        KERNEL_STATE.process_table[waiter].state = ProcessState::Ready;
    }
    
    0
}
```

### 5.4 IPC Patterns

#### Request/Reply (Synchronous)

```rust
// Client
let my_reply_port = port_allocate()?;
let request = [OP_READ, fd, buf, len, my_reply_port, 0, 0, 0];
port_send(server_port, request, 8)?;

// Wait for reply (blocking)
let reply = port_receive(my_reply_port)?;
let bytes_read = reply[0] as usize;

// Server
loop {
    let req = port_receive(server_port)?;
    let bytes = vfs_read(req[1], req[2], req[3]);
    port_send(req[4], [bytes, 0, 0, 0, 0, 0, 0, 0], 8)?;
}
```

#### Fire & Forget (Asynchronous)

```rust
// Client
let log_msg = [LOG_WRITE, level, timestamp, pid, 0, 0, 0, 0];
port_send(log_server_port, log_msg, 8)?;
// No reply expected

// Server
loop {
    let msg = port_receive(log_port)?;
    write_log_entry(msg);
}
```

#### Broadcast (Multicast)

```rust
// Kernel broadcasts events to subscribed services
let subscribers = get_subscribers("network.device_up");
for sub_pid in subscribers {
    let event = [EVENT_NETWORK_UP, dev_id, 0, 0, 0, 0, 0, 0];
    port_send(sub_pid.notify_port, event, 8)?;
}
```

---

## 6. Memory Management

### 6.1 Virtual Memory Layout (per-process)

```
0x0000_0000_0000_0000 ──────────┐
                                  │
  Guard Page                       │ (4 KB)
  ↓                               │
  Code Segment         0x0000_1000 ├──────
  ↓                               │
  Data Segment         0x0010_0000 ├──────
  ↓                               │
  BSS                  0x0020_0000 ├──────
  ↓                               │
  Heap (grows ↑)       0x1000_0000 ├──────
  ...                             │
  Stack (grows ↓)      0x7FFF_0000 ├──────
  ↓                               │
  Guard Page           0x7FFF_F000 ├──────
  ↓                               │
0x7FFF_FFFF_FFFF_FFFF ──────────┘

Kernel Memory (0xFFFF_8000_0000_0000 and above)
```

### 6.2 Page Allocator

```rust
// kernel/src/memory/page_alloc.rs
pub struct PageAllocator {
    free_frames: Vec<PhysicalFrame>,
    allocated: BTreeMap<u64, usize>,  // addr → size
}

impl PageAllocator {
    pub fn allocate(&mut self, page_count: usize) -> Option<PhysicalFrame> {
        if page_count <= self.free_frames.len() {
            let frame = self.free_frames.pop()?;
            self.allocated.insert(frame.address, page_count * 4096);
            Some(frame)
        } else {
            None
        }
    }
    
    pub fn deallocate(&mut self, addr: u64) {
        if let Some(size) = self.allocated.remove(&addr) {
            let frame_count = size / 4096;
            for i in 0..frame_count {
                self.free_frames.push(PhysicalFrame {
                    address: addr + (i as u64 * 4096),
                });
            }
        }
    }
}
```

### 6.3 Page Table Management

```rust
// kernel/src/memory/page_table.rs
pub struct PageTableEntry {
    pub present: bool,
    pub writable: bool,
    pub user_accessible: bool,
    pub write_through: bool,
    pub cache_disable: bool,
    pub accessed: bool,
    pub dirty: bool,
    pub physical_addr: u64,
}

pub struct PageTable {
    level: u8,  // 4 (PML4), 3 (PDPT), 2 (PDT), 1 (PT)
    entries: [PageTableEntry; 512],
}

pub fn map_page(
    page_table: &mut PageTable,
    virt_addr: u64,
    phys_addr: u64,
    flags: MapFlags,
) {
    let pml4_idx = (virt_addr >> 39) & 0x1FF;
    let pdpt_idx = (virt_addr >> 30) & 0x1FF;
    let pdt_idx = (virt_addr >> 21) & 0x1FF;
    let pt_idx = (virt_addr >> 12) & 0x1FF;
    
    // Navigate through page table hierarchy
    let mut table = page_table;
    for &idx in &[pml4_idx as usize, pdpt_idx as usize, pdt_idx as usize] {
        if !table.entries[idx].present {
            table.entries[idx].physical_addr = allocate_page_table();
            table.entries[idx].present = true;
        }
        table = unsafe { &mut *(table.entries[idx].physical_addr as *mut PageTable) };
    }
    
    // Map final page
    table.entries[pt_idx as usize] = PageTableEntry {
        present: true,
        writable: flags.writable,
        user_accessible: flags.user_accessible,
        physical_addr: phys_addr,
        ..Default::default()
    };
}
```

---

## 7. Task Scheduling

### 7.1 Scheduler Design (Userspace Policy)

**Key Insight**: Kernel only performs context switching. Scheduling policy is userspace.

```
┌─ Kernel ──────────────────────────┐
│ Timer interrupt (100 Hz)          │
│ → Send message to scheduler_server│
│ → Wait for sched_switch() call    │
└───────────────────────────────────┘

       ↓

┌─ Scheduler_server ────────────────┐
│ Maintain ready queue              │
│ Decide: which task runs next?     │
│ Call sched_switch(next_pid)       │
└───────────────────────────────────┘

       ↓

┌─ Kernel ──────────────────────────┐
│ Context switch: old_task → next   │
│ Update CR3 (page tables)          │
│ IRET to userspace                 │
└───────────────────────────────────┘
```

### 7.2 Scheduler Implementation (Pseudocode)

```rust
// servers/scheduler_server/src/main.rs
fn main() {
    let scheduler_port = port_allocate()?;
    let mut ready_queue: VecDeque<ProcessID> = VecDeque::new();
    let mut sleeping: BTreeMap<ProcessID, Time> = BTreeMap::new();
    let mut running: Option<ProcessID> = None;
    
    loop {
        // Receive messages (timer, yield, etc.)
        let msg = port_receive(scheduler_port)?;
        
        match msg[0] {
            MSG_TIMER_TICK => {
                // Wake sleeping tasks whose time has expired
                let now = sys_time();
                while let Some((&pid, &wake_time)) = sleeping.iter().next() {
                    if wake_time <= now {
                        ready_queue.push_back(pid);
                        sleeping.remove(&pid);
                    } else {
                        break;
                    }
                }
                
                // Pick next task
                if let Some(current) = running {
                    ready_queue.push_back(current);  // Put back if still runnable
                }
                
                if let Some(next) = ready_queue.pop_front() {
                    running = Some(next);
                    sys_sched_switch(next)?;  // noreturn
                }
            }
            MSG_TASK_YIELD => {
                // Task called sched_yield()
                if let Some(current) = running {
                    ready_queue.push_back(current);
                }
                
                if let Some(next) = ready_queue.pop_front() {
                    running = Some(next);
                    sys_sched_switch(next)?;
                }
            }
            MSG_TASK_SLEEP(pid, duration) => {
                sleeping.insert(pid, sys_time() + duration);
                
                if let Some(next) = ready_queue.pop_front() {
                    running = Some(next);
                    sys_sched_switch(next)?;
                }
            }
            _ => {}
        }
    }
}
```

### 7.3 Context Switching (x86_64 Assembly)

```asm
; kernel/src/arch/x86_64/context_switch.S
; void context_switch(old_rsp: *mut u64, new_rsp: u64, new_cr3: u64)

global context_switch
context_switch:
    ; rdi = old_rsp (pointer to stack location)
    ; rsi = new_rsp (new stack pointer)
    ; rdx = new_cr3 (new page table root)
    
    ; Save old registers to kernel stack
    push rbp
    push rbx
    push r12
    push r13
    push r14
    push r15
    
    ; Save old RSP
    mov [rdi], rsp
    
    ; Switch page tables (TLB flush)
    mov cr3, rdx
    
    ; Load new RSP
    mov rsp, rsi
    
    ; Restore new registers
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp
    
    ; Return to userspace via IRET
    ret
```

---

## 8. Fault Isolation

### 8.1 Isolation Mechanisms

| Mechanism | How It Works | Effect |
|-----------|------------|--------|
| **Page Tables** | Each process has separate CR3 | Memory isolation |
| **Rings** | User code in ring 3, kernel in ring 0 | Privilege separation |
| **Capabilities** | No ambient authority | No accidental access |
| **Process Death** | Kernel reclaims process resources | No zombie state |

### 8.2 Exception Handling

When a service crashes:

```rust
// kernel/src/exception.rs
pub extern "C" fn general_protection_fault(frame: &mut InterruptFrame) {
    let pid = current_pid();
    eprintln!("GPF in process {}", pid);
    
    // Kill the process
    let proc = &mut KERNEL_STATE.process_table[pid];
    proc.state = ProcessState::Dead;
    
    // Reclaim resources
    deallocate_process_memory(pid);
    revoke_capabilities(pid);
    
    // Notify init_server
    notify_init_server(InitMessage::ProcessDied(pid));
    
    // Switch to next ready process
    sys_sched_yield();
}
```

### 8.3 Recovery & Restart

```rust
// servers/init_server/src/main.rs
fn on_service_crash(pid: ProcessID) {
    match SERVICES.get(&pid) {
        Some(service) => {
            eprintln!("Service {} crashed, restarting...", service.name);
            
            // Wait a bit to avoid crash loop
            thread::sleep(Duration::from_secs(1));
            
            // Restart service
            match spawn_service(&service) {
                Ok(new_pid) => println!("Service restarted as PID {}", new_pid),
                Err(e) => eprintln!("Failed to restart: {:?}", e),
            }
        }
        None => {}
    }
}
```

---

## 9. Architecture Specifics (x86_64, ARM64)

### 9.1 x86_64 Details

#### GDT (Global Descriptor Table)

```rust
// kernel/src/arch/x86_64/gdt.rs
pub struct GlobalDescriptorTable {
    table: [u64; 8],
}

impl GlobalDescriptorTable {
    pub fn new() -> Self {
        Self {
            table: [
                0,  // Null descriptor
                descriptor(0, 0xFFFFFFFF, GDT_CODE | GDT_PRESENT),  // Kernel code
                descriptor(0, 0xFFFFFFFF, GDT_DATA | GDT_PRESENT),  // Kernel data
                descriptor(0, 0xFFFFFFFF, GDT_CODE | GDT_USER | GDT_PRESENT),  // User code
                descriptor(0, 0xFFFFFFFF, GDT_DATA | GDT_USER | GDT_PRESENT),  // User data
                0,  // Reserved
                0,  // TSS low
                0,  // TSS high
            ]
        }
    }
}
```

#### IDT (Interrupt Descriptor Table)

```rust
// kernel/src/arch/x86_64/idt.rs
pub struct InterruptDescriptorTable {
    table: [IDTEntry; 256],
}

#[repr(C)]
pub struct IDTEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    attributes: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

pub fn init_idt() {
    let mut idt = InterruptDescriptorTable::new();
    
    // Exceptions
    idt.set_handler(0, exception_divide_by_zero);      // #DE
    idt.set_handler(1, exception_debug);               // #DB
    idt.set_handler(3, exception_breakpoint);          // #BP
    idt.set_handler(6, exception_invalid_opcode);      // #UD
    idt.set_handler(8, exception_double_fault);        // #DF
    idt.set_handler(14, exception_page_fault);         // #PF
    
    // Hardware interrupts (IRQ 0-15)
    idt.set_handler(32, interrupt_timer);              // PIT/APIC timer
    idt.set_handler(33, interrupt_keyboard);           // PS/2 keyboard
    idt.set_handler(34, interrupt_cascade);
    idt.set_handler(35, interrupt_com2);
    idt.set_handler(36, interrupt_com1);
    // ...
    
    // Syscall (int 0x80)
    idt.set_handler(0x80, syscall_entry);
    
    idt.load();
}
```

#### Paging (4-Level Page Tables)

```
Virtual Address (48-bit):
┌──────┬──────┬──────┬──────┬──────────┐
│ Sign │ L4   │ L3   │ L2   │ L1 + Offset│
│ Ext  │ 9-bit│ 9-bit│ 9-bit│ 9-bit  + 12│
└──────┴──────┴──────┴──────┴──────────┘
   (47)  (39-30) (30-21) (21-12)   (11-0)

PML4 (Page Map Level 4) ──┐
   ↓                        │ CR3 points here
PDPT (Page Directory Pointer Table)
   ↓
PDT (Page Directory Table)
   ↓
PT (Page Table)
   ↓
Physical Page (4 KB)
```

### 9.2 ARM64 Details

#### Exception Vectors

```asm
; kernel/src/arch/arm64/vectors.S
.align 12
exception_vectors:
    ; Synchronous (from same EL)
    .align 7
    b sync_same_el
    
    ; IRQ (from same EL)
    .align 7
    b irq_same_el
    
    ; FIQ (from same EL)
    .align 7
    b fiq_same_el
    
    ; SError (from same EL)
    .align 7
    b serror_same_el
    
    ; Similar blocks for lower EL (EL0)
```

#### Translation Table

```rust
// kernel/src/arch/arm64/mmu.rs
pub struct TranslationTableBase {
    ttbr0: u64,  // Lower VA space (0x0000...)
    ttbr1: u64,  // Upper VA space (0xFFFF...)
}

pub struct TranslationTable {
    entries: [u64; 512],  // 4-level walk
}
```

---

## 10. Common Patterns & Best Practices

### 10.1 Safe Error Handling

```rust
// ✗ BAD: Can panic
let port = port_allocate().unwrap();

// ✓ GOOD: Explicit error handling
match port_allocate() {
    Ok(port) => { /* use port */ },
    Err(e) => { eprintln!("Failed: {:?}", e); return; }
}

// ✓ BEST: Use ? operator
let port = port_allocate()?;
```

### 10.2 Capability Patterns

```rust
// Pattern 1: Capability delegation
let vfs_cap = port_allocate()?;
let read_only_cap = cap_move(vfs_cap, process_b, CAP_SEND)?;

// Pattern 2: Capability revocation
cap_revoke(read_only_cap)?;

// Pattern 3: Capability verification
if !has_right(cap, CAP_RECEIVE) {
    return Err(Error::NoRights);
}
```

### 10.3 IPC Message Passing

```rust
// Pattern 1: Synchronous request/reply
let msg = [OP_READ, fd, buf, len, reply_port, 0, 0, 0];
port_send(server_port, &msg, 8)?;
let reply = port_receive(reply_port)?;

// Pattern 2: Asynchronous notification
let notify = [EVENT_DONE, result, 0, 0, 0, 0, 0, 0];
port_send(notify_port, &notify, 8)?;

// Pattern 3: Bulk data transfer (with references)
struct BulkData {
    addr: u64,
    len: usize,
}

let msg = [OP_SEND_BULK, bulk.addr as u64, bulk.len as u64, ...];
port_send(server_port, &msg, 8)?;
```

### 10.4 Memory Safety

```rust
// ✗ BAD: Unsafe without verification
let data = unsafe { *(user_ptr as *const u64) };

// ✓ GOOD: Verify address is valid first
fn read_user_memory(addr: u64) -> Result<u64> {
    if !is_user_address(addr) {
        return Err(Error::InvalidAddress);
    }
    Ok(unsafe { *(addr as *const u64) })
}

// ✓ BEST: Use kernel copy routines
fn copy_from_user(src: u64, dst: &mut [u8]) -> Result<()> {
    if src + dst.len() as u64 > USER_SPACE_END {
        return Err(Error::InvalidAddress);
    }
    unsafe {
        core::ptr::copy_nonoverlapping(src as *const u8, dst.as_mut_ptr(), dst.len());
    }
    Ok(())
}
```

### 10.5 No-Panic Policy

```rust
// ✗ BAD: Can panic
assert_eq!(a, b);
vec![1, 2, 3][10];

// ✓ GOOD: Return errors instead
if a != b {
    return Err(Error::Mismatch);
}

match safe_get_index(vec, 10) {
    Some(v) => { /* use v */ },
    None => return Err(Error::OutOfBounds),
}
```

---

## Conclusion

GBSD's microkernel design prioritizes:

1. **Minimal TCB** – Less code = fewer bugs = better security
2. **Fault Isolation** – Services fail independently
3. **Capability-Based Security** – Fine-grained, unforgeable access control
4. **Userspace Policy** – Scheduling, service management in userspace
5. **Type-Safe Implementation** – Rust prevents entire classes of bugs

This architecture enables GBSD to run complex services (graphics, networking, filesystems) safely and efficiently, while keeping the trusted computing base under 8 KB.


