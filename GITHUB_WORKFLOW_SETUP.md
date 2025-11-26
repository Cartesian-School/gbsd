# GitHub Kanban & Workflow Setup Guide

**For**: Cartesian School Organization  
**Project**: GuardBSD (GBSD)  
**Date**: November 26, 2025

---

## Quick Start

1. **Import Issues**: Use ISSUES.md to create GitHub issues (23 total)
2. **Setup Board**: Configure Kanban columns in GitHub Projects
3. **Assign Labels**: Use provided label taxonomy
4. **Setup Workflows**: Create CI/CD pipeline (Issue #7)

---

## GitHub Issues Taxonomy

### Priority Labels
```
🔴 critical    - Must fix before Phase 4 (5 issues)
🟠 high        - Should complete Phase 4 (8 issues)
🟡 medium      - Nice to have Phase 4 (6 issues)
🟢 low         - Future enhancement (4 issues)
```

### Issue Type Labels
```
🐛 bug                    - Defect in code
✨ feature                - New functionality
📚 documentation          - Docs, comments, guides
🔍 testing                - Testing and QA
🔧 build                  - Build system issues
🔐 security               - Security concerns
⚡ performance            - Performance optimization
♿ accessibility          - Accessibility
```

### Category Labels
```
kernel              - Kernel-related
services            - Services (any)
memory-management   - Virtual memory subsystem
ipc                 - Inter-process communication
scheduler           - Task scheduling
vfs                 - Virtual filesystem
networking          - Network stack
phase-1             - Phase 1 work (complete)
phase-2             - Phase 2 work (mostly complete)
phase-3             - Phase 3 work (in progress)
phase-4             - Phase 4 planning
```

### Status Labels
```
blocked             - Waiting on external factor/other issue
in-progress         - Currently being worked
needs-review        - Pull request/code review needed
dependencies        - Has dependency on other issue
```

---

## Issue Template

Use this format when creating issues on GitHub:

```markdown
**Title**: [Issue Title]

**Type**: [Bug/Feature/Documentation/Testing/Build/Security/Performance]

**Priority**: [Critical/High/Medium/Low]

**Description**:
[Detailed description from ISSUES.md]

**Root Cause** (if applicable):
[Technical explanation]

**Acceptance Criteria**:
- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Criterion 3

**Affected Components**:
[List files/modules affected]

**Estimated Effort**: [X days]

**Labels**:
- priority: [critical/high/medium/low]
- type: [bug/feature/etc]
- area: [kernel/services/etc]

**Related Issues**:
[Link to related issues]
```

---

## Kanban Board Setup

### Project Name
```
GBSD - Microkernel OS Development
```

### Columns (Recommended Setup)
```
1. Backlog
   └─ Description: Prioritized list of all work items
   └─ Automation: New issues default here

2. Ready
   └─ Description: Issues approved for development
   └─ Automation: Manual move when ready

3. In Progress
   └─ Description: Currently being worked
   └─ Automation: Move when PR created

4. In Review
   └─ Description: Pull request submitted
   └─ Automation: Auto-move when PR created

5. Done
   └─ Description: Completed and merged
   └─ Automation: Auto-move when PR merged
```

### Column Configuration (GitHub Projects v2)
```
Backlog:
  - Auto-add: All new issues
  - Status: Todo

Ready:
  - Status: Todo (ready for dev)
  - Manual moves

In Progress:
  - Status: In Progress
  - Auto-trigger: When linked PR created

In Review:
  - Status: In Review
  - Auto-trigger: When PR created

Done:
  - Status: Done
  - Auto-trigger: When PR merged to main
```

---

## GitHub Workflows Configuration

### File: `.github/workflows/build.yml`

```yaml
name: Build & Test

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

jobs:
  build:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
    
    - name: Install dependencies
      run: |
        sudo apt-get update
        sudo apt-get install -y nasm grub-common xorriso qemu-system-x86
    
    - name: Check
      run: cargo check --all
    
    - name: Build (Debug)
      run: cargo build --all
    
    - name: Build (Release)
      run: cargo build --all --release
    
    - name: Lint with Clippy
      run: cargo clippy --all -- -D warnings
    
    - name: Format check
      run: cargo fmt --all -- --check
    
    - name: Run tests
      run: cargo test --all
    
    - name: Upload artifacts
      if: success()
      uses: actions/upload-artifact@v3
      with:
        name: build-artifacts
        path: target/release/
```

### File: `.github/workflows/security.yml`

```yaml
name: Security Audit

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sunday

jobs:
  audit:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
    
    - name: Run security audit
      uses: rustsec/audit-check-action@v1
      with:
        token: ${{ secrets.GITHUB_TOKEN }}
```

### File: `.github/workflows/docs.yml`

```yaml
name: Documentation

on:
  push:
    branches: [ main, develop ]

jobs:
  docs:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
    
    - name: Build docs
      run: cargo doc --all --no-deps
    
    - name: Deploy to GitHub Pages
      if: github.ref == 'refs/heads/main'
      uses: peaceiris/actions-gh-pages@v3
      with:
        github_token: ${{ secrets.GITHUB_TOKEN }}
        publish_dir: ./target/doc
```

---

## Branch Protection Rules

Apply to `main` and `develop` branches:

```
Required status checks:
  ✓ Build & Test workflow
  ✓ Security Audit workflow
  ✓ Code review (1 approval minimum)

Require branches to be up to date: Yes
Require code review: Yes
Require status checks to pass: Yes
```

---

## Issue Dependency Linking

### Critical Path (Must complete in order):
```
Issue #1 (Build)
  ↓
Issue #3 (Scheduler syscalls)
  ↓
Issue #4 (Memory management)
  ↓
Issue #2 (Integration tests)
  ↓
Issue #5 (Phase 3 services)
```

### On GitHub:
Link using "Linked issues" feature in each issue:
- Issue #3 depends on Issue #1
- Issue #4 depends on Issue #3
- Issue #2 depends on Issue #4
- Issue #5 depends on Issue #2

This creates a visible dependency graph.

---

## Milestone Configuration

### Suggested Milestones

**Phase 3 - Blocker Fixes** (Target: 2 weeks)
- Issue #1, #3, #4 (and supporting issues)
- Milestone goal: Project builds and boots

**Phase 3 - Integration & Testing** (Target: 1 week)
- Issue #2, #5, #13
- Milestone goal: Phase 3 services functional

**Phase 3 - Quality & Security** (Target: 1 week)
- Issue #7, #8, #9, #11, #12
- Milestone goal: Production-ready Phase 3

**Phase 4 - Planning** (Target: 2 weeks)
- Issue #10, #17, #20-23
- Milestone goal: Phase 4 specification ready

---

## Review Process

### Code Review Checklist

Before approving PR, verify:
- [ ] Closes issue (link issue)
- [ ] Builds successfully
- [ ] Tests pass (all pass including new tests)
- [ ] No clippy warnings
- [ ] Code formatted (cargo fmt)
- [ ] Unsafe code justified (SAFETY comments)
- [ ] Error handling complete
- [ ] Documentation updated
- [ ] Performance acceptable (no regressions)

### PR Template

Create `.github/pull_request_template.md`:

```markdown
## Description
[Description of changes]

## Fixes Issue
Closes #[issue number]

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Security fix

## Testing
- [ ] Unit tests added
- [ ] Integration tests added
- [ ] Tested in QEMU
- [ ] Performance verified

## Checklist
- [ ] Code follows style guidelines
- [ ] New code includes comments/documentation
- [ ] No new warnings generated
- [ ] All tests pass
- [ ] Unsafe code justified

## Screenshots (if applicable)
[Add screenshots or console output]
```

---

## Metrics to Track

### Velocity
- Track issues completed per week
- Ideal: 5-7 issues/week for Phase 3 work

### Burndown
- Plot remaining work vs. time
- Target: Linear decrease to zero at milestone end

### Quality
- Build success rate (target: 100%)
- Test pass rate (target: 100%)
- Code review approval rate

### Coverage
- Test coverage (target: >80% for kernel)
- Documentation coverage (all public APIs)

---

## Regular Ceremonies

### Daily (Async)
- Update issue status in PR comments
- Link blockers/dependencies

### Weekly (Sync)
- Sprint planning (Monday)
- Sprint review (Friday)
- Retrospective (Friday)

### Bi-Weekly
- Demo of completed work
- Stakeholder update

---

## Issue Labels (Create in GitHub)

```
## Severity
🔴 critical
🟠 high
🟡 medium
🟢 low

## Type
🐛 bug
✨ feature
📚 documentation
🔍 testing
🔧 build
🔐 security
⚡ performance

## Component
📦 kernel
🖥️ services
💾 memory-management
🔗 ipc
⏰ scheduler
📂 vfs
🌐 networking

## Phase
1️⃣ phase-1
2️⃣ phase-2
3️⃣ phase-3
4️⃣ phase-4

## Status
⛔ blocked
▶️ in-progress
👀 needs-review
⛓️ dependencies
🆙 ready

## Duration
👶 trivial (< 1 day)
📌 small (1-2 days)
📋 medium (2-5 days)
📊 large (5+ days)
```

---

## Success Criteria

### Phase 3 Completion
- ✅ Issue #1 resolved (build works)
- ✅ Issue #3 resolved (scheduler functional)
- ✅ Issue #4 resolved (memory management)
- ✅ Issue #2 resolved (integration tests)
- ✅ Issue #5 resolved (Phase 3 services)
- ✅ All tests passing
- ✅ Zero build warnings
- ✅ Ready for Phase 4

**Estimated Timeline**: 3-4 weeks from start

### Phase 4 Entry Criteria
- Phase 3 completion checklist ✅
- Security audit complete
- Documentation complete
- Team trained on codebase
- Phase 4 specification approved

---

## Support & Troubleshooting

### Common Issues

**Q: How do I link a PR to an issue?**
A: In PR description, use "Closes #123" or "Fixes #123"

**Q: How do I mark an issue as blocked?**
A: Add `blocked` label and link the blocking issue

**Q: How do I add a PR to the project board?**
A: GitHub automatically adds PRs that close issues. Alternatively, manually add to board.

**Q: How do I see the critical path?**
A: View linked issues in Issue #1 (build)

---

## Next Steps

1. Create the 23 issues in GitHub (use ISSUES.md)
2. Add all provided labels
3. Create milestones for Phase 3 work
4. Setup branch protection on main/develop
5. Create .github/workflows/ files
6. Assign team members
7. Start with Issue #1 (build fix)
8. Track progress on board

---

## Resources

- **Project Board**: https://github.com/orgs/Cartesian-School/projects/1
- **Issues List**: ISSUES.md (this repository)
- **Review Summary**: REVIEW_SUMMARY.md
- **Architecture**: docs/ARCHITECTURE_GUIDE.md

