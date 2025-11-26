# GBSD Phase 2 - Testing & Integration Plan

**Date**: November 25, 2025  
**Phase**: 2 (Bootstrap Services)  
**Status**: Implementation Complete - Testing Ready

---

## Testing Strategy

### Unit Testing (Per Service)

#### init_server Unit Tests
```rust
#[test]
fn test_port_allocation() {
    // Test: Verify port allocation succeeds
    // Expected: get_port() returns valid u32 > 0
}

#[test]
fn test_service_registry() {
    // Test: Verify service registration
    // Expected: SERVICES array updated correctly
}

#[test]
fn test_process_spawn() {
    // Test: Verify process spawn syscall
    // Expected: spawn_process() returns PID > 0
}
```

#### log_server Unit Tests
```rust
#[test]
fn test_ring_buffer_write() {
    // Test: Write to ring buffer
    // Expected: Entry stored at head position
}

#[test]
fn test_ring_buffer_wrap() {
    // Test: Ring buffer wraps at 16384
    // Expected: head and tail wrap correctly
}

#[test]
fn test_log_entry_format() {
    // Test: Log entry structure
    // Expected: Correct sizes and alignment
}
```

#### scheduler_server Unit Tests
```rust
#[test]
fn test_queue_enqueue() {
    // Test: Enqueue to ready queue
    // Expected: PID added to queue
}

#[test]
fn test_queue_dequeue() {
    // Test: Dequeue from ready queue
    // Expected: Correct FIFO order
}

#[test]
fn test_sleeping_tasks() {
    // Test: Sleeping tasks tracking
    // Expected: Tasks wake when time reached
}
```

---

## Integration Testing

### IPC Communication Flow

**Test 1: init_server → log_server**
```
Setup:
1. Start init_server
2. init_server allocates port
3. init_server spawns log_server
4. log_server allocates own port

Expected:
- Both services have valid ports
- Services can exchange messages
- Messages delivered correctly
```

**Test 2: init_server → scheduler_server**
```
Setup:
1. init_server starts
2. init_server spawns scheduler_server
3. Services initialize

Expected:
- Scheduler port allocated
- Scheduler ready queue empty
- Scheduler waiting for events
```

**Test 3: Any Service → log_server**
```
Setup:
1. All services running
2. Service sends LOG_WRITE message
3. log_server receives and processes

Expected:
- Message received in log_server
- Entry added to ring buffer
- Output sent to serial console
```

---

## System Testing

### Boot Sequence Test

**Expected Output**:
```
[kernel] IDT initialized
[init] init_server started (PID 1)
[init] Allocated port for init_server
[init] Starting log_server...
[init] log_server started (PID 2)
[log] log_server started (PID 2)
[log] Allocated port for logging
[log] Ready for log messages
[init] Starting scheduler_server...
[init] scheduler_server started (PID 3)
[scheduler] scheduler_server started (PID 3)
[scheduler] Allocated port for scheduling
[scheduler] Ready queue: empty
[scheduler] Waiting for events...
[init] All bootstrap services started
[init] Waiting for events...
```

**Test Steps**:
1. Build kernel
2. Build services
3. Boot in QEMU with `-serial mon:stdio`
4. Capture serial output
5. Compare with expected output

**Success Criteria**:
- [ ] All three services start
- [ ] No panics or crashes
- [ ] Serial output matches expected
- [ ] Services enter main loops

---

## QEMU Boot Testing

### Command
```bash
qemu-system-x86_64 \
  -kernel target/release/kernel \
  -initrd initrd.img \
  -serial mon:stdio \
  -m 512M \
  -nographic
```

### Monitoring
```bash
# In separate terminal
cat /tmp/qemu.log | grep -E "\[init\]|\[log\]|\[scheduler\]"
```

### Expected Result
Services boot in order: init → log → scheduler

---

## Error Scenarios

### Scenario 1: Port Allocation Fails
```
Symptom: E_NOMEM returned from port_allocate()
Expected Handling: Service exits gracefully
Test: Verify error code propagated
```

### Scenario 2: Message Queue Full
```
Symptom: E_PORT_FULL returned from port_send()
Expected Handling: Retry or log error
Test: Verify backpressure handling
```

### Scenario 3: Service Crashes
```
Symptom: Child process dies
Expected Handling: init_server detects CMD_SERVICE_DIED
Test: Verify service recovery
```

### Scenario 4: Receive on Empty Port
```
Symptom: E_PORT_INVALID returned
Expected Handling: Process handles gracefully
Test: Verify non-blocking behavior
```

---

## Performance Benchmarks (Target for Phase 3)

| Operation | Target | Metric |
|-----------|--------|--------|
| Port allocation | < 1 µs | Latency |
| Port send | < 5 µs | Latency |
| Port receive | < 5 µs | Latency |
| Context switch | < 500 ns | Latency |
| Log write | < 10 µs | Throughput |
| Ring buffer | > 10K msg/s | Throughput |

---

## Debugging Tools

### Serial Console Monitoring
```bash
# Monitor serial output in real-time
screen /dev/ttyUSB0 115200

# Or with socat
socat -d -d pty,raw,echo=0 pty,raw,echo=0
```

### QEMU Monitor
```bash
# Connect to QEMU monitor
(echo "info registers"; sleep 1) | nc localhost 5555

# View memory
x /16x 0x1000
```

### GDB Debugging
```bash
# Start QEMU with GDB server
qemu-system-x86_64 -s -S -kernel kernel

# Connect GDB
gdb
target remote localhost:1234
```

---

## Known Issues & Workarounds

### Issue 1: Context Switching Not Implemented
**Status**: Expected - Phase 3
**Workaround**: Services run sequentially for now
**Fix**: Implement x86_64 context switch assembly

### Issue 2: Process Blocking Not Working
**Status**: Expected - Phase 2
**Workaround**: Non-blocking polls in main loops
**Fix**: Implement blocked task list in kernel

### Issue 3: Message Content Limited
**Status**: Current - Phase 2
**Workaround**: Simple message format only
**Fix**: Add message parsing helpers

---

## Test Coverage Map

```
Code Under Test (Phase 2)
├── init_server (~240 LOC)
│   ├── Port allocation ✓
│   ├── Service spawning ✓
│   ├── Message sending ✓
│   ├── Event handling ✓
│   └── Error cases ⚠️ (needs testing)
│
├── log_server (~245 LOC)
│   ├── Ring buffer ✓
│   ├── Message receiving ✓
│   ├── Serial output ✓
│   └── Wrap-around ⚠️ (needs testing)
│
└── scheduler_server (~275 LOC)
    ├── Ready queue ✓
    ├── Sleeping tasks ✓
    ├── Round-robin ⚠️ (needs context switch)
    └── Task switching ⚠️ (needs context switch)
```

---

## Test Execution Plan

### Week 2 (This Week)
**Goal**: Verify boot sequence and IPC

**Tests**:
1. [ ] Compilation test (30 min)
2. [ ] Boot test in QEMU (1 hour)
3. [ ] Serial output verification (1 hour)
4. [ ] IPC message test (2 hours)
5. [ ] Error handling test (2 hours)

**Expected Outcome**: Services boot, communicate, no crashes

### Week 3
**Goal**: Fix issues, complete integration

**Tests**:
1. [ ] Service lifecycle test (2 hours)
2. [ ] Logging end-to-end test (2 hours)
3. [ ] Scheduler event test (2 hours)
4. [ ] Stress testing (3 hours)

**Expected Outcome**: All services stable, ready for Phase 3

### Week 4
**Goal**: Documentation, polish, handoff

**Tests**:
1. [ ] Final integration test (2 hours)
2. [ ] Documentation review (2 hours)
3. [ ] Code cleanup (2 hours)
4. [ ] Performance baseline (2 hours)

**Expected Outcome**: Phase 2 complete, ready for Phase 3

---

## Success Criteria

### Minimum Viable Product (MVP)
- [x] All three services compile
- [ ] Services boot without panic
- [ ] Serial output visible
- [ ] IPC messages delivered
- [ ] No infinite loops

### Production Ready
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Performance within targets
- [ ] Error handling comprehensive
- [ ] Documentation complete

### Phase 2 Complete
- [ ] MVP + Production ready
- [ ] 100+ manual tests passed
- [ ] Ready for Phase 3
- [ ] All tasks from Phase 2 spec done

---

## Regression Testing

### When Adding Features
1. Ensure existing tests still pass
2. Add new tests for features
3. Run full test suite
4. Check performance impact

### Before Release
1. Run all tests (unit + integration)
2. Boot in QEMU
3. Verify serial output
4. Check for memory leaks
5. Confirm error handling

---

## Test Report Template

```
TEST REPORT - Phase 2
Date: YYYY-MM-DD
Test Session: N
Tester: [Name]

Tests Run: XX
Tests Passed: XX
Tests Failed: XX
Tests Skipped: XX

Issues Found:
1. [Description]
   Status: [Open/In Progress/Fixed]
   
Pass Rate: XX%
Confidence: [Low/Medium/High]

Next Steps:
- [Task 1]
- [Task 2]
```

---

## Continuous Integration Setup (Future)

```yaml
# .github/workflows/build-test.yml
name: Build and Test
on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Build
        run: cargo build --release
      - name: Test
        run: cargo test --lib
      - name: Boot Test
        run: |
          qemu-system-x86_64 -kernel kernel -serial mon:stdio
```

---

## Debugging Checklist

When Something Breaks:
- [ ] Check compiler errors
- [ ] Check runtime panics
- [ ] Verify serial output
- [ ] Check syscall return values
- [ ] Trace message flow
- [ ] Check memory bounds
- [ ] Verify port IDs
- [ ] Check queue status

---

## Support Resources

**Documentation**:
- `PHASE2_SPECIFICATION.md` - Detailed requirements
- `QUICKSTART.md` - Developer guide
- `ARCHITECTURE_GUIDE.md` - Technical design

**Code References**:
- `kernel/src/syscall.rs` - Syscall implementations
- `libgbsd/src/lib.rs` - Syscall wrappers
- `servers/init_server/src/main.rs` - Reference implementation

---

**Status**: Ready for Testing  
**Effort**: 760 lines of code  
**Timeline**: Week 2 of 4 (Phase 2)  
**Next**: Boot and verify in QEMU


