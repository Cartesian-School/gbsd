// kernel/src/ipc.rs
// Inter-Process Communication (IPC) - Port and Capability management

use crate::error::*;
use crate::globals::*;
use alloc::vec::Vec;

/// Allocate a new port for the current process
pub fn port_allocate() -> u64 {
    let mut state = kernel_state_mut();
    let mut next_id = NEXT_PORT_ID.lock();

    let port_id = *next_id;
    *next_id += 1;

    let current_pid = state.current_process_id;
    let port = Port::new(port_id, current_pid);

    state.ports.push(port);

    // Allocate capability for this port
    let mut cap_id = NEXT_CAP_ID.lock();
    let capability = Capability::new(
        *cap_id,
        current_pid,
        port_id,
        CAP_SEND | CAP_RECEIVE | CAP_DESTROY,
    );
    *cap_id += 1;

    state.capabilities.push(capability);

    port_id as u64
}

/// Send a message to a port
pub fn port_send(port_id: u32, msg_ptr: *const u64, len: usize) -> u64 {
    if len != 8 {
        return E_INVAL;
    }

    // Read message from user space (assume valid for now)
    let msg = unsafe {
        [
            *msg_ptr,
            *msg_ptr.add(1),
            *msg_ptr.add(2),
            *msg_ptr.add(3),
            *msg_ptr.add(4),
            *msg_ptr.add(5),
            *msg_ptr.add(6),
            *msg_ptr.add(7),
        ]
    };

    let mut state = kernel_state_mut();
    let current_pid = state.current_process_id;

    // Find port
    let port = match state.ports.iter_mut().find(|p| p.id == port_id) {
        Some(p) => p,
        None => return E_PORT_INVALID,
    };

    // Check if sender has capability with SEND right
    if !has_capability(current_pid, port_id, CAP_SEND, &state) {
        return E_NO_RIGHTS;
    }

    // Check if port queue is full
    if port.is_full() {
        return E_PORT_FULL;
    }

    // Push message
    if port.push_message(&msg) {
        E_OK
    } else {
        E_PORT_FULL
    }
}

/// Receive a message from a port
pub fn port_receive(port_id: u32, buf_ptr: *mut u64, len: usize) -> u64 {
    if len != 8 {
        return E_INVAL;
    }

    let mut state = kernel_state_mut();
    let current_pid = state.current_process_id;

    // Find port
    let port = match state.ports.iter_mut().find(|p| p.id == port_id) {
        Some(p) => p,
        None => return E_PORT_INVALID,
    };

    // Check if receiver has capability with RECEIVE right
    if !has_capability(current_pid, port_id, CAP_RECEIVE, &state) {
        return E_NO_RIGHTS;
    }

    // Pop message from queue
    match port.pop_message() {
        Some(msg) => {
            // Write message to user space (assume valid for now)
            unsafe {
                *buf_ptr = msg[0];
                *buf_ptr.add(1) = msg[1];
                *buf_ptr.add(2) = msg[2];
                *buf_ptr.add(3) = msg[3];
                *buf_ptr.add(4) = msg[4];
                *buf_ptr.add(5) = msg[5];
                *buf_ptr.add(6) = msg[6];
                *buf_ptr.add(7) = msg[7];
            }
            8  // 8 u64s received
        }
        None => {
            // No message available - for now return error (future: block)
            E_PORT_INVALID  // TODO: Block process
        }
    }
}

/// Move (transfer) a capability from one process to another
pub fn cap_move(src_cap_id: u32, dst_pid: u32, rights: u32) -> u64 {
    let mut state = kernel_state_mut();
    let current_pid = state.current_process_id;

    // Find source capability
    let src_cap = match state.capabilities.iter().find(|c| c.id == src_cap_id) {
        Some(c) => *c,
        None => return E_CAP_INVALID,
    };

    // Verify current process owns the capability
    if src_cap.owner_pid != current_pid {
        return E_NO_RIGHTS;
    }

    // Verify capability is not revoked
    if src_cap.revoked {
        return E_CAP_INVALID;
    }

    // Verify requested rights are subset of current rights
    if (rights & src_cap.rights) != rights {
        return E_NO_RIGHTS;
    }

    // Verify destination process exists
    if !state.processes.iter().any(|p| p.id == dst_pid) {
        return E_PROCESS_NOT_FOUND;
    }

    // Create new capability for destination process
    let mut cap_id = NEXT_CAP_ID.lock();
    let new_cap = Capability::new(*cap_id, dst_pid, src_cap.target_id, rights);
    *cap_id += 1;

    state.capabilities.push(new_cap);

    E_OK
}

/// Revoke a capability
pub fn cap_revoke(cap_id: u32) -> u64 {
    let mut state = kernel_state_mut();
    let current_pid = state.current_process_id;

    // Find capability
    let cap = match state.capabilities.iter_mut().find(|c| c.id == cap_id) {
        Some(c) => c,
        None => return E_CAP_INVALID,
    };

    // Verify current process owns the capability
    if cap.owner_pid != current_pid {
        return E_NO_RIGHTS;
    }

    cap.revoke();
    E_OK
}

/// Check if a process has a specific capability with required rights
fn has_capability(
    pid: u32,
    target_id: u32,
    required_right: u32,
    state: &KernelState,
) -> bool {
    state
        .capabilities
        .iter()
        .any(|c| {
            c.owner_pid == pid
                && c.target_id == target_id
                && !c.revoked
                && (c.rights & required_right) != 0
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_allocate() {
        // This would require mocking the global state
        // Actual tests would be in integration tests
    }
}

