# GBSD Project Issues & Backlog

**Last Updated**: November 26, 2025  
**Status**: Ready for GitHub Project Import  
**Total Issues**: 23 (5 Critical, 8 High, 6 Medium, 4 Low)

---

## Critical Issues 🔴 (Must Fix Before Phase 4)

### Issue 1: Build Failure - Yanked Dependency

**Title**: Fix yanked multiboot2 dependency causing build failure  
**Type**: Build  
**Priority**: CRITICAL  
**Status**: TODO  
**Assignee**: DevOps  

**Description**:
```
error: failed to select a version for the requirement `multiboot2 = "^0.15"`
  version 0.15.0 is yanked
  version 0.15.1 is yanked
```

**Root Cause**: multiboot2 versions 0.15.0 and 0.15.1 are yanked from crates.io. Project cannot build.

**Acceptance Criteria**:
- [x] Identify working multiboot2 version
- [ ] Update kernel/Cargo.toml with valid version
- [ ] Verify `cargo build` completes successfully
- [ ] Zero warnings/errors reported

**Affected Components**: kernel, all services

**Labels**: bug, build, blocker

---

### Issue 2: Integration Testing Missing

**Title**: Implement integration test suite for Phase 2-3 services  
**Type**: Testing  
**Priority**: CRITICAL  
**Status**: TODO  
**Assignee**: QA Team  

**Description**:
Services (init_server, log_server, scheduler_server, vfs_server, ext4_server, netstack_server) have zero integration tests. Only unit test stubs exist in kernel.

**Current State**:
- kernel/src/error_tests.rs: Exists ✅
- kernel/src/ipc_tests.rs: Exists ✅
- Service integration tests: Missing ❌
- Boot sequence tests: Missing ❌
- IPC communication tests: Missing ❌

**Acceptance Criteria**:
- [ ] Create integration test framework for service startup
- [ ] Test init_server → log_server communication
- [ ] Test init_server → scheduler_server communication
- [ ] Test service port allocation and message passing
- [ ] Test complete boot sequence
- [ ] Document test results in TESTING_DOCUMENTATION_INDEX.md

**Affected Components**: init_server, log_server, scheduler_server, vfs_server, ext4_server, netstack_server

**Labels**: testing, integration-tests, critical

---

### Issue 3: Incomplete Syscall Implementations

**Title**: Complete remaining TODO syscall implementations  
**Type**: Implementation  
**Priority**: CRITICAL  
**Status**: TODO  
**Assignee**: Kernel Team  

**Description**:
Multiple syscall handlers contain incomplete implementations marked with TODO comments:
- `sys_sched_yield()` (kernel/src/syscall.rs:114) - Returns E_OK without actual yield
- `sys_sched_switch()` (kernel/src/syscall.rs:129) - Context switch not implemented
- `sys_time()` (kernel/src/syscall.rs:135) - Returns dummy 0x1000 instead of TSC

**Code Review Findings**:
```rust
fn sys_sched_yield() -> u64 {
    E_OK  // TODO: Implement proper yield
}

fn sys_sched_switch(target_pid: u32) -> u64 {
    // TODO: Actual context switch
    E_OK
}

fn sys_time() -> u64 {
    0x1000  // Dummy value
}
```

**Impact**: Scheduler cannot function correctly. Services will hang waiting for task switches.

**Acceptance Criteria**:
- [ ] Implement sys_sched_yield() to trigger scheduler
- [ ] Implement sys_sched_switch() for context switching
- [ ] Implement sys_time() to read TSC on x86_64
- [ ] Add ARM64 alternatives for time reading
- [ ] Test all three syscalls in isolation
- [ ] Verify scheduler_server responds to yield/switch

**Affected Components**: kernel/src/syscall.rs, scheduler_server

**Labels**: bug, kernel, critical

---

### Issue 4: Memory Management Stubs

**Title**: Implement proper virtual memory management  
**Type**: Implementation  
**Priority**: CRITICAL  
**Status**: TODO  
**Assignee**: Memory Team  

**Description**:
Memory management syscalls (vm_allocate, vm_deallocate) are stubs:

```rust
fn sys_vm_allocate(hint: u64, size: u64, flags: u32) -> u64 {
    if size == 0 || size > 0x40000000 {
        return E_INVAL;
    }
    // For now, just return a valid address in user space
    let addr = 0x1000 + hint;
    // Real implementation would allocate physical pages and map them
    addr
}
```

**Missing**:
- Physical page allocation
- Page table creation and management
- Memory mapping (MMU setup)
- Address space layout randomization
- Memory protection enforcement

**Impact**: Services cannot allocate dynamic memory. Only stack memory available.

**Acceptance Criteria**:
- [ ] Implement physical page frame allocator
- [ ] Implement page table management (x86_64)
- [ ] Implement virtual address space allocation
- [ ] Add memory protection (execute, read, write flags)
- [ ] Test memory allocation/deallocation cycle
- [ ] Implement ARM64 page table support

**Affected Components**: kernel/src/memory.rs, kernel/src/syscall.rs

**Labels**: feature, memory, critical

---

### Issue 5: Phase 3 Services - Incomplete Implementation & Testing

**Title**: Complete Phase 3 services (vfs_server, ext4_server, netstack_server)  
**Type**: Implementation  
**Priority**: CRITICAL  
**Status**: IN PROGRESS  
**Assignee**: Services Team  

**Description**:
Phase 3 services have stubs but are not fully functional:

**vfs_server (servers/vfs_server/src/main.rs)**:
- tmpfs filesystem declared but never integrated
- File operations (create_file, write_file) have partial implementations
- Message handlers not complete
- No error handling for edge cases

**ext4_server (servers/ext4_server/src/main.rs)**:
- Block cache implemented but never used
- ext4 structures defined but no actual parsing
- No real block device I/O
- No journal support

**netstack_server (servers/netstack_server/src/main.rs)**:
- Socket table declared but operations incomplete
- State machine for sockets exists but not enforced
- No actual TCP/UDP implementation
- No network interface support

**Acceptance Criteria**:
- [ ] vfs_server: Complete tmpfs filesystem implementation
- [ ] vfs_server: Implement all 10 VFS operations (open, close, read, write, seek, stat, mkdir, rmdir, readdir, unlink)
- [ ] vfs_server: Add directory traversal and path resolution
- [ ] ext4_server: Parse ext4 superblock and block groups
- [ ] ext4_server: Implement inode reading/writing
- [ ] ext4_server: Add block cache management with LRU eviction
- [ ] netstack_server: Implement TCP state machine
- [ ] netstack_server: Add UDP socket support
- [ ] netstack_server: Test all socket operations
- [ ] Integration: Test services communicating with vfs/ext4/netstack

**Affected Components**: servers/vfs_server, servers/ext4_server, servers/netstack_server

**Labels**: implementation, phase-3, critical

---

## High Priority Issues 🟠 (Should Fix in Phase 4)

### Issue 6: Dependency on Yanked Bootloader Version

**Title**: Evaluate and update bootloader dependency  
**Type**: Build/Dependencies  
**Priority**: HIGH  
**Status**: TODO  
**Assignee**: Build Team  

**Description**:
kernel/Cargo.toml uses bootloader v0.9, which may have security issues or lack modern features.

**Current**:
```toml
bootloader = { version = "0.9", features = ["map-physical-memory"] }
```

**Risk**: Older bootloader version may lack:
- UEFI support
- Secure boot compatibility
- Modern memory layout handling
- KASLR support

**Acceptance Criteria**:
- [ ] Evaluate bootloader 0.11+ compatibility
- [ ] Test with latest bootloader version
- [ ] Verify memory layout still works
- [ ] Document bootloader choice rationale

**Affected Components**: kernel/Cargo.toml, linker.ld

**Labels**: dependencies, build, high-priority

---

### Issue 7: No CI/CD Pipeline

**Title**: Implement GitHub Actions CI/CD workflow  
**Type**: DevOps  
**Priority**: HIGH  
**Status**: TODO  
**Assignee**: DevOps  

**Description**:
Project has no automated build/test pipeline. Manual builds required.

**Missing Components**:
- Automated build on push
- Compilation error detection
- Warning as errors enforcement
- Automated test execution
- Code quality checks
- Release builds

**Acceptance Criteria**:
- [ ] Create .github/workflows/build.yml
- [ ] Run cargo build on every push
- [ ] Run cargo test for kernel unit tests
- [ ] Enforce zero warnings policy
- [ ] Generate build artifacts
- [ ] Document CI/CD setup

**Affected Components**: Repository configuration

**Labels**: devops, automation, high-priority

---

### Issue 8: Incomplete Error Handling

**Title**: Add comprehensive error handling to service message handlers  
**Type**: Code Quality  
**Priority**: HIGH  
**Status**: TODO  
**Assignee**: Services Team  

**Description**:
Service message handlers have minimal error handling. No validation of incoming messages or state checks before operations.

**Examples**:
- vfs_server file operations: No path validation
- netstack_server socket operations: No state machine enforcement
- ext4_server block operations: No corruption detection
- Services: No capability checking before operations

**Impact**: Services may crash on malformed input or state violations.

**Acceptance Criteria**:
- [ ] Add input validation to all message handlers
- [ ] Implement state machine enforcement
- [ ] Add capability checks before operations
- [ ] Return proper error codes for all failure cases
- [ ] Log errors to log_server
- [ ] Test with invalid inputs

**Affected Components**: All services

**Labels**: quality, error-handling, high-priority

---

### Issue 9: Missing Memory Bounds Checking

**Title**: Add memory bounds validation to kernel syscalls  
**Type**: Security  
**Priority**: HIGH  
**Status**: TODO  
**Assignee**: Kernel Security Team  

**Description**:
Syscalls accept user-provided pointers without thorough bounds checking:

```rust
fn sys_port_send(port_id: u32, msg_ptr: *const u64, len: usize) -> u64 {
    // No validation of msg_ptr
    let msg = unsafe {
        core::ptr::read(msg_ptr as *const [u64; 8])
    };
    // ...
}
```

**Risk**: Invalid pointer dereference, kernel panic, potential security vulnerability.

**Acceptance Criteria**:
- [ ] Add pointer validation to all syscalls accepting pointers
- [ ] Implement page-fault-safe pointer access
- [ ] Add bounds checking for buffer operations
- [ ] Document pointer safety guarantees
- [ ] Add security tests for invalid pointers

**Affected Components**: kernel/src/syscall.rs, kernel/src/ipc.rs

**Labels**: security, memory-safety, high-priority

---

### Issue 10: Incomplete Documentation of Message Protocols

**Title**: Document all service IPC message formats and state machines  
**Type**: Documentation  
**Priority**: HIGH  
**Status**: TODO  
**Assignee**: Documentation Team  

**Description**:
Message protocol definitions are scattered across service files. No centralized documentation of:
- Message format specifications
- Protocol state machines
- Error response formats
- Message version/compatibility

**Missing Documentation**:
- vfs_server protocol specification (10 message types)
- ext4_server protocol specification
- netstack_server protocol specification
- State machine diagrams
- Protocol versioning strategy

**Acceptance Criteria**:
- [ ] Create docs/IPC_PROTOCOLS.md
- [ ] Document all message types per service
- [ ] Add state machine diagrams
- [ ] Document error response formats
- [ ] Version protocol specifications
- [ ] Add protocol versioning strategy

**Affected Components**: docs/, all services

**Labels**: documentation, api, high-priority

---

### Issue 11: No Runtime Crash Analysis

**Title**: Implement crash dump and analysis mechanism  
**Type**: Debugging  
**Priority**: HIGH  
**Status**: TODO  
**Assignee**: Debug Team  

**Description**:
When services crash (panic), no crash dump or analysis is provided. Difficult to debug production issues.

**Missing**:
- Panic handler that logs stack trace
- Register dump on panic
- Crash report generation
- Memory dump capture
- Service restart on crash

**Acceptance Criteria**:
- [ ] Enhance panic handlers in all services
- [ ] Log panic information to log_server
- [ ] Implement stack trace printing
- [ ] Capture register state on panic
- [ ] Generate crash reports
- [ ] Document crash analysis process

**Affected Components**: All services, kernel panic handler

**Labels**: debugging, robustness, high-priority

---

### Issue 12: Unsafe Code Not Documented

**Title**: Document all unsafe code blocks and safety invariants  
**Type**: Code Quality  
**Priority**: HIGH  
**Status**: TODO  
**Assignee**: Security Review Team  

**Description**:
Project contains ~50+ unsafe blocks with minimal or no documentation of safety invariants.

**Examples**:
- kernel/src/ipc.rs:43 - Unsafe pointer read without justification
- kernel/src/arch/x86_64/idt.rs:9 - Unsafe IDT initialization
- servers/init_server/src/main.rs:76 - Unsafe syscall invocation

**Impact**: Security audit risk. Difficult to verify safety.

**Acceptance Criteria**:
- [ ] Add // SAFETY comments to all unsafe blocks
- [ ] Document invariants being relied upon
- [ ] Add SAFETY.md with unsafe justification matrix
- [ ] Create audit checklist for unsafe code
- [ ] Review and approve all unsafe code

**Affected Components**: kernel, all services

**Labels**: security, code-review, high-priority

---

### Issue 13: Testing Documentation Incomplete

**Title**: Complete PHASE2_TESTING_PLAN.md implementation and results  
**Type**: Testing/Documentation  
**Priority**: HIGH  
**Status**: TODO  
**Assignee**: QA Team  

**Description**:
PHASE2_TESTING_PLAN.md exists with test specifications but shows:
- No actual test implementations
- No test results documented
- No test run output captured
- No failure analysis

**Current State**:
- Unit test stubs: Defined but not implemented
- Integration tests: Outlined but not executed
- System tests: Listed but untested
- Boot sequence: Never verified

**Acceptance Criteria**:
- [ ] Implement all Phase 2 unit tests
- [ ] Execute integration test suite
- [ ] Capture actual boot sequence output
- [ ] Document test results and coverage
- [ ] Fix any test failures
- [ ] Update testing plan with actual results

**Affected Components**: tests/, PHASE2_TESTING_PLAN.md

**Labels**: testing, documentation, high-priority

---

## Medium Priority Issues 🟡 (Nice to Have, Plan for Phase 4)

### Issue 14: Memory Leak Detection

**Title**: Implement memory leak detection for services  
**Type**: Testing/Quality  
**Priority**: MEDIUM  
**Status**: TODO  
**Assignee**: QA Team  

**Description**:
Services allocate memory but no tracking of leaks. In #[no_std] environment, malloc/free patterns need auditing.

**Acceptance Criteria**:
- [ ] Audit all service memory allocations
- [ ] Implement allocation tracking
- [ ] Add leak detection tests
- [ ] Document memory limits per service
- [ ] Add statistics to log_server

**Affected Components**: All services, memory.rs

**Labels**: quality, memory, medium-priority

---

### Issue 15: Hardcoded Service Addresses

**Title**: Replace hardcoded binary addresses with bootloader-provided addresses  
**Type**: Implementation  
**Priority**: MEDIUM  
**Status**: TODO  
**Assignee**: Boot Team  

**Description**:
init_server contains hardcoded service binary addresses:

```rust
const LOG_SERVER_ADDR: u64 = 0x100000;
const SCHEDULER_SERVER_ADDR: u64 = 0x200000;
```

**Problem**: Addresses conflict with actual binary layout. Services won't load at correct addresses.

**Acceptance Criteria**:
- [ ] Get binary addresses from multiboot2 header
- [ ] Pass addresses through kernel to init_server
- [ ] Remove hardcoded addresses
- [ ] Verify services load at correct locations

**Affected Components**: servers/init_server, kernel

**Labels**: implementation, boot, medium-priority

---

### Issue 16: No Performance Benchmarking

**Title**: Add performance benchmarks for critical paths  
**Type**: Performance  
**Priority**: MEDIUM  
**Status**: TODO  
**Assignee**: Performance Team  

**Description**:
No performance metrics captured for:
- Syscall overhead
- IPC latency
- Scheduler context switch time
- File operation throughput
- Socket throughput

**Acceptance Criteria**:
- [ ] Create benchmark suite
- [ ] Measure syscall overhead
- [ ] Measure IPC latency
- [ ] Measure context switch time
- [ ] Document baseline performance
- [ ] Identify optimization opportunities

**Affected Components**: All

**Labels**: performance, testing, medium-priority

---

### Issue 17: Incomplete API Documentation

**Title**: Generate and publish API documentation  
**Type**: Documentation  
**Priority**: MEDIUM  
**Status**: TODO  
**Assignee**: Documentation Team  

**Description**:
No rustdoc comments on public APIs. No generated documentation.

**Acceptance Criteria**:
- [ ] Add rustdoc to all public functions
- [ ] Add examples in rustdoc
- [ ] Generate cargo doc output
- [ ] Publish docs online
- [ ] Add API reference to README

**Affected Components**: All Rust code

**Labels**: documentation, api, medium-priority

---

### Issue 18: Workspace Configuration Warnings

**Title**: Fix Cargo workspace configuration warnings  
**Type**: Build  
**Priority**: MEDIUM  
**Status**: TODO  
**Assignee**: Build Team  

**Description**:
Build produces warnings:
```
warning: profiles for the non root package will be ignored
warning: virtual workspace defaulting to `resolver = "1"`
```

**Acceptance Criteria**:
- [ ] Move profiles to workspace root
- [ ] Set resolver = "2" explicitly
- [ ] Verify zero warnings on build
- [ ] Document workspace layout

**Affected Components**: Cargo.toml files

**Labels**: build, warnings, medium-priority

---

### Issue 19: ARM64 Support Incomplete

**Title**: Complete ARM64 architecture support  
**Type**: Implementation  
**Priority**: MEDIUM  
**Status**: TODO  
**Assignee**: Architecture Team  

**Description**:
ARM64 support exists only as stubs. No actual implementation for:
- Exception handlers
- Context switching
- Memory management
- Syscall entry point

**Acceptance Criteria**:
- [ ] Implement ARM64 exception handlers
- [ ] Implement ARM64 context switching
- [ ] Implement ARM64 memory management
- [ ] Test on ARM64 hardware/QEMU
- [ ] Document ARM64 differences

**Affected Components**: kernel/src/arch/arm64/

**Labels**: feature, architecture, medium-priority

---

## Low Priority Issues 🟢 (Future Enhancement)

### Issue 20: Wayland/Graphics Stubs Not Created

**Title**: Prepare stubs for Phase 4 graphics services  
**Type**: Planning  
**Priority**: LOW  
**Status**: TODO  
**Assignee**: Graphics Team  

**Description**:
Phase 4 will require:
- drm_server (DRM/KMS driver)
- mesa_server (OpenGL/Vulkan)
- gwc (Wayland compositor)

Currently not stubbed out. Preparing stubs will help Phase 4 planning.

**Acceptance Criteria**:
- [ ] Create servers/drm_server stub
- [ ] Create servers/mesa_server stub
- [ ] Create servers/gwc stub
- [ ] Document graphics architecture
- [ ] Plan Phase 4 timeline

**Affected Components**: servers/

**Labels**: phase-4, graphics, low-priority

---

### Issue 21: SSH Server Stub

**Title**: Create SSH server stub for Phase 4  
**Type**: Planning  
**Priority**: LOW  
**Status**: TODO  
**Assignee**: Networking Team  

**Description**:
Phase 4 will include sshd. Creating stub now helps planning and integration.

**Acceptance Criteria**:
- [ ] Create servers/sshd stub
- [ ] Document SSH protocol integration points
- [ ] Plan authentication strategy

**Affected Components**: servers/

**Labels**: phase-4, networking, low-priority

---

### Issue 22: Shell Implementation Plan

**Title**: Document shell (gsh) implementation strategy  
**Type**: Planning  
**Priority**: LOW  
**Status**: TODO  
**Assignee**: Applications Team  

**Description**:
Phase 4 includes custom shell (gsh). Need implementation strategy for:
- Command parsing
- Builtin commands
- Job control
- Signal handling

**Acceptance Criteria**:
- [ ] Document shell architecture
- [ ] Plan command parsing strategy
- [ ] List required builtins
- [ ] Plan signal handling

**Affected Components**: Documentation

**Labels**: phase-4, applications, low-priority

---

### Issue 23: Code Style and Formatting

**Title**: Establish and enforce code style guidelines  
**Type**: Quality  
**Priority**: LOW  
**Status**: TODO  
**Assignee**: Development Team  

**Description**:
No enforced code style or rustfmt configuration.

**Acceptance Criteria**:
- [ ] Add rustfmt.toml
- [ ] Run rustfmt on all code
- [ ] Configure clippy lint levels
- [ ] Add pre-commit hooks
- [ ] Document style guide

**Affected Components**: Repository configuration

**Labels**: quality, style, low-priority

---

## Issue Summary by Category

### Build Issues (3)
- Issue 1: Yanked multiboot2 dependency ❌ BLOCKING
- Issue 6: Bootloader dependency evaluation
- Issue 18: Workspace configuration warnings

### Testing Issues (5)
- Issue 2: Integration testing missing ❌ BLOCKING
- Issue 13: Testing documentation incomplete
- Issue 14: Memory leak detection
- Issue 16: Performance benchmarking
- Issue 19: ARM64 support incomplete

### Implementation Issues (7)
- Issue 3: Incomplete syscall implementations ❌ BLOCKING
- Issue 4: Memory management stubs ❌ BLOCKING
- Issue 5: Phase 3 services incomplete ❌ BLOCKING
- Issue 15: Hardcoded service addresses
- Issue 20: Wayland/Graphics stubs
- Issue 21: SSH server stub
- Issue 22: Shell implementation plan

### Documentation Issues (4)
- Issue 10: IPC protocol documentation
- Issue 12: Unsafe code documentation
- Issue 17: API documentation
- Issue 23: Code style guidelines

### Security/Quality Issues (4)
- Issue 8: Incomplete error handling
- Issue 9: Memory bounds checking
- Issue 11: Runtime crash analysis
- Issue 12: Unsafe code documentation

---

## Recommended Resolution Order

### Phase 3.1 - Blocker Fixes (1-2 weeks)
1. **Issue 1** - Fix yanked dependencies (1 day)
2. **Issue 3** - Implement remaining syscalls (3 days)
3. **Issue 4** - Implement memory management (2 days)
4. Rebuild and verify

### Phase 3.2 - Integration & Testing (2-3 weeks)
5. **Issue 2** - Integration test suite (2-3 days)
6. **Issue 5** - Complete Phase 3 services (5 days)
7. **Issue 13** - Implement Phase 2 testing (2 days)
8. Test complete boot sequence

### Phase 3.3 - Quality & Security (1-2 weeks)
9. **Issue 7** - Setup CI/CD pipeline (2 days)
10. **Issue 8** - Error handling audit (2 days)
11. **Issue 9** - Memory bounds validation (2 days)
12. **Issue 12** - Unsafe code review (2 days)

### Phase 4 - Documentation & Enhancement (Parallel)
13. **Issue 10** - Message protocol docs (1 day)
14. **Issue 11** - Crash analysis (1 day)
15. **Issue 17** - API documentation (1 day)
16. Planning issues (20-23)

---

## GitHub Labels Recommended

```
- blocker: Release/milestone blocking
- build: Build system, compilation
- bug: Defect in code
- critical: Must fix before Phase 4
- documentation: Docs, comments, guides
- error-handling: Error/exception handling
- feature: New feature request
- high-priority: Should complete Phase 4
- implementation: Code development
- integration-tests: Integration testing
- memory-safety: Memory safety issues
- phase-3: Phase 3 work
- phase-4: Phase 4 planning
- performance: Performance optimization
- quality: Code quality improvements
- security: Security related
- testing: Testing and QA
- unsafe-code: Unsafe code review
```

---

## Workflow Recommendations

### For Kanban Board (`https://github.com/orgs/Cartesian-School/projects/1`)

**Columns**:
1. **Backlog** - All issues, prioritized
2. **Ready** - Issues approved for development
3. **In Progress** - Currently being worked
4. **In Review** - Pull request submitted
5. **Done** - Completed and merged

**Critical Path** (Issue 1 → 3 → 4 → 2 → 5):
- Must complete in order
- Total estimated time: 3-4 weeks
- Blocks Phase 4 until complete

**Parallel Tracks** (After blockers):
- Issue 7 (CI/CD) - DevOps
- Issue 8-9 (Quality) - Code Review
- Issue 10-12 (Documentation) - Tech Writers

---

## Notes for Development Team

1. **Build system is blocking** - Issue 1 must be resolved first
2. **Scheduler is incomplete** - Issue 3 prevents proper task management
3. **Memory management is stubbed** - Issue 4 limits service capabilities
4. **No integration tests exist** - Issue 2 makes Phase 3 verification impossible
5. **Phase 3 services are incomplete** - Issue 5 leaves core functionality missing

**Recommendation**: Focus on Critical issues first to achieve working Phase 3 system, then move to High-priority for Phase 4 readiness.

