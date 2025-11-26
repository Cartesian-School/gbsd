// kernel/src/ipc_tests.rs
// Unit tests for IPC module

#[cfg(test)]
mod tests {
    use crate::globals::*;

    #[test]
    fn test_port_creation() {
        let port = Port::new(1, 1);
        assert_eq!(port.id, 1);
        assert_eq!(port.owner_pid, 1);
        assert_eq!(port.queue_size, 0);
        assert!(!port.is_full());
        assert!(port.is_empty());
    }

    #[test]
    fn test_port_message_queue() {
        let mut port = Port::new(1, 1);

        // Create a test message
        let msg = [1u64, 2, 3, 4, 5, 6, 7, 8];

        // Push message
        assert!(port.push_message(&msg), "Should push message successfully");
        assert_eq!(port.queue_size, 1);
        assert!(!port.is_empty());

        // Pop message
        let received = port.pop_message();
        assert!(received.is_some(), "Should pop message");
        let received_msg = received.unwrap();
        assert_eq!(received_msg[0], 1);
        assert_eq!(received_msg[7], 8);
        assert_eq!(port.queue_size, 0);
        assert!(port.is_empty());
    }

    #[test]
    fn test_port_ring_buffer_wrapping() {
        let mut port = Port::new(1, 1);
        let msg = [1u64, 2, 3, 4, 5, 6, 7, 8];

        // Fill the queue
        for i in 0..64 {
            assert!(
                port.push_message(&msg),
                "Should be able to push {} messages",
                i + 1
            );
        }

        assert!(port.is_full(), "Queue should be full at 64 messages");
        assert!(!port.push_message(&msg), "Should not push when full");

        // Pop some messages
        for _ in 0..10 {
            assert!(port.pop_message().is_some());
        }

        assert_eq!(port.queue_size, 54);

        // Should be able to push more
        assert!(port.push_message(&msg), "Should push after making space");
    }

    #[test]
    fn test_capability_creation() {
        let cap = Capability::new(1, 10, 100, 0x03);
        assert_eq!(cap.id, 1);
        assert_eq!(cap.owner_pid, 10);
        assert_eq!(cap.target_id, 100);
        assert_eq!(cap.rights, 0x03);
        assert!(!cap.revoked);
    }

    #[test]
    fn test_capability_rights_check() {
        let cap = Capability::new(1, 10, 100, 0x03); // Rights: bit 0 and 1

        // Check rights
        assert!(cap.has_right(0x01), "Should have right 0x01");
        assert!(cap.has_right(0x02), "Should have right 0x02");
        assert!(!cap.has_right(0x04), "Should not have right 0x04");
    }

    #[test]
    fn test_capability_revocation() {
        let mut cap = Capability::new(1, 10, 100, 0x03);
        assert!(!cap.revoked, "Capability should not be revoked initially");
        assert!(cap.has_right(0x01), "Should have rights before revocation");

        cap.revoke();
        assert!(cap.revoked, "Capability should be revoked");
        assert!(!cap.has_right(0x01), "Should not have rights after revocation");
    }

    #[test]
    fn test_process_descriptor_creation() {
        let proc = ProcessDescriptor {
            id: 1,
            name: [0; 32],
            memory_start: 0x1000,
            memory_end: 0x2000,
            page_table_root: 0,
            state: ProcessState::Ready,
            stack_pointer: 0x2000,
            instruction_pointer: 0x1000,
        };

        assert_eq!(proc.id, 1);
        assert_eq!(proc.memory_start, 0x1000);
        assert_eq!(proc.memory_end, 0x2000);
    }

    #[test]
    fn test_process_state_values() {
        // Verify process states
        assert_eq!(ProcessState::Ready as usize, 0);
        assert_eq!(ProcessState::Running as usize, 1);
        assert_eq!(ProcessState::Sleeping as usize, 2);
        assert_eq!(ProcessState::Dead as usize, 3);
    }
}

