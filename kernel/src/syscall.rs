use crate::error::*;
use crate::ipc::{port_allocate, port_send, port_receive, cap_move, cap_revoke};
use crate::globals::*;

/// Main syscall handler - dispatches to appropriate syscall
pub fn handle_syscall(num: u64, args: [u64; 6]) -> u64 {
    match num {
        SYS_PORT_ALLOCATE => sys_port_allocate(),
        SYS_PORT_SEND => sys_port_send(args[0] as u32, args[1] as *const u64, args[2] as usize),
        SYS_PORT_RECEIVE => sys_port_receive(args[0] as u32, args[1] as *mut u64, args[2] as usize),
        SYS_VM_ALLOCATE => sys_vm_allocate(args[0], args[1], args[2] as u32),
        SYS_VM_DEALLOCATE => sys_vm_deallocate(args[0], args[1]),
        SYS_CAP_MOVE => sys_cap_move(args[0] as u32, args[1] as u32, args[2] as u32),
        SYS_SCHED_SPAWN => sys_sched_spawn(args[0], args[1], args[2] as *const u8),
        SYS_SCHED_YIELD => sys_sched_yield(),
        SYS_SCHED_SWITCH => sys_sched_switch(args[0] as u32),
        SYS_TIME => sys_time(),
        _ => E_INVALID_SYSCALL,
    }
}

/// 1. Allocate a new port for IPC
fn sys_port_allocate() -> u64 {
    port_allocate()
}

/// 2. Send a message to a port
fn sys_port_send(port_id: u32, msg_ptr: *const u64, len: usize) -> u64 {
    port_send(port_id, msg_ptr, len)
}

/// 3. Receive a message from a port
fn sys_port_receive(port_id: u32, buf_ptr: *mut u64, len: usize) -> u64 {
    port_receive(port_id, buf_ptr, len)
}

/// 4. Allocate user virtual memory
fn sys_vm_allocate(hint: u64, size: u64, flags: u32) -> u64 {
    if size == 0 || size > 0x40000000 {  // Max 1 GB
        return E_INVAL;
    }

    if (hint & 0xFFF) != 0 {
        return E_ALIGN;  // Must be 4 KB aligned
    }

    // For now, just return a valid address in user space
    // Real implementation would allocate physical pages and map them
    let addr = 0x1000 + hint;
    if addr < 0x800000000000 {
        // Assuming user space goes up to this
        addr
    } else {
        E_NOMEM
    }
}

/// 5. Deallocate user virtual memory
fn sys_vm_deallocate(addr: u64, size: u64) -> u64 {
    if size == 0 {
        return E_INVAL;
    }

    if (addr & 0xFFF) != 0 {
        return E_ALIGN;
    }

    // For now, just acknowledge
    E_OK
}

/// 6. Move (transfer) a capability
fn sys_cap_move(src_cap: u32, dst_pid: u32, rights: u32) -> u64 {
    cap_move(src_cap, dst_pid, rights)
}

/// 7. Spawn a new task (process)
fn sys_sched_spawn(entry: u64, stack: u64, name_ptr: *const u8) -> u64 {
    if entry == 0 || stack == 0 {
        return E_INVAL;
    }

    // Verify entry and stack are in user space
    if entry >= 0x800000000000 || stack >= 0x800000000000 {
        return E_INVAL;
    }

    let mut state = kernel_state_mut();
    let mut pid_counter = NEXT_PROCESS_ID.lock();

    let new_pid = *pid_counter;
    *pid_counter += 1;

    let proc = ProcessDescriptor {
        id: new_pid,
        name: [0u8; 32],
        memory_start: 0x1000,
        memory_end: 0x800000000000,
        page_table_root: 0,  // Would be set by MMU
        state: ProcessState::Ready,
        stack_pointer: stack,
        instruction_pointer: entry,
    };

    state.processes.push(proc);

    new_pid as u64
}

/// 8. Yield CPU to scheduler
fn sys_sched_yield() -> u64 {
    // Signal scheduler that this task is yielding
    // The scheduler will decide what to do next
    E_OK  // TODO: Implement proper yield
}

/// 9. Switch to a different task (scheduler-only)
fn sys_sched_switch(target_pid: u32) -> u64 {
    let mut state = kernel_state_mut();

    // Only scheduler_server should call this
    // For now, just verify the PID exists
    if !state.processes.iter().any(|p| p.id == target_pid) {
        return E_PROCESS_NOT_FOUND;
    }

    state.current_process_id = target_pid;

    // TODO: Actual context switch
    E_OK
}

/// 10. Get monotonic time
fn sys_time() -> u64 {
    // Read TSC (Time Stamp Counter) on x86_64
    // For now, return a dummy value
    // Real implementation: read rdmsr(0x10)
    0x1000
}
