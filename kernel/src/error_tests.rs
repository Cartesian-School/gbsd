// kernel/src/error_tests.rs
// Unit tests for error module

#[cfg(test)]
mod tests {
    use crate::error::*;

    #[test]
    fn test_error_codes_are_unique() {
        // Verify all error codes are distinct
        let errors = [
            E_OK,
            E_PORT_INVALID,
            E_PORT_FULL,
            E_NO_RIGHTS,
            E_INVAL,
            E_NOMEM,
            E_CAP_INVALID,
            E_PROCESS_NOT_FOUND,
            E_NOT_OWNER,
            E_ALIGN,
            E_INVALID_SYSCALL,
        ];

        // Check no duplicates
        for i in 0..errors.len() {
            for j in (i + 1)..errors.len() {
                assert_ne!(
                    errors[i], errors[j],
                    "Error codes must be unique: {:x} == {:x}",
                    errors[i], errors[j]
                );
            }
        }
    }

    #[test]
    fn test_error_codes_are_non_zero() {
        // E_OK is 0, others should be non-zero
        assert_eq!(E_OK, 0, "E_OK must be 0");

        let errors = [
            E_PORT_INVALID,
            E_PORT_FULL,
            E_NO_RIGHTS,
            E_INVAL,
            E_NOMEM,
            E_CAP_INVALID,
            E_PROCESS_NOT_FOUND,
            E_NOT_OWNER,
            E_ALIGN,
            E_INVALID_SYSCALL,
        ];

        for error in errors.iter() {
            assert_ne!(*error, 0, "Error code must be non-zero: {:x}", *error);
        }
    }

    #[test]
    fn test_error_format() {
        // Verify error codes follow pattern 0xFFFFFFFF_XXXXXXXX
        let errors = [
            E_PORT_INVALID,
            E_PORT_FULL,
            E_NO_RIGHTS,
            E_INVAL,
            E_NOMEM,
            E_CAP_INVALID,
            E_PROCESS_NOT_FOUND,
            E_NOT_OWNER,
            E_ALIGN,
            E_INVALID_SYSCALL,
        ];

        for error in errors.iter() {
            let upper = (error >> 32) as u32;
            assert_eq!(
                upper, 0xFFFFFFFF,
                "Error code upper bits must be 0xFFFFFFFF: {:x}",
                error
            );
        }
    }

    #[test]
    fn test_syscall_numbers() {
        // Verify all syscall numbers are defined
        assert_eq!(SYS_PORT_ALLOCATE, 1);
        assert_eq!(SYS_PORT_SEND, 2);
        assert_eq!(SYS_PORT_RECEIVE, 3);
        assert_eq!(SYS_VM_ALLOCATE, 4);
        assert_eq!(SYS_VM_DEALLOCATE, 5);
        assert_eq!(SYS_CAP_MOVE, 6);
        assert_eq!(SYS_SCHED_SPAWN, 7);
        assert_eq!(SYS_SCHED_YIELD, 8);
        assert_eq!(SYS_SCHED_SWITCH, 9);
        assert_eq!(SYS_TIME, 10);
    }

    #[test]
    fn test_capability_rights() {
        // Verify capability rights are bit flags
        assert_eq!(CAP_SEND & CAP_RECEIVE, 0, "CAP rights must not overlap");
        assert_eq!(CAP_SEND & CAP_DESTROY, 0);
        assert_eq!(CAP_RECEIVE & CAP_DESTROY, 0);

        // All should be powers of 2 or combinations
        let rights = [
            CAP_SEND, CAP_RECEIVE, CAP_DESTROY, CAP_DERIVE, CAP_READ, CAP_WRITE, CAP_EXECUTE,
        ];

        for right in rights.iter() {
            assert!(right.count_ones() <= 1 || *right == 0, "Each right should be a single bit");
        }
    }
}

