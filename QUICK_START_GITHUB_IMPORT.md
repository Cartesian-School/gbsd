# Quick Start: GitHub Project Import

**Created**: November 26, 2025  
**For**: Cartesian School Organization  
**Time to Complete**: ~4 hours

---

## 📋 Pre-Import Checklist

- [ ] Have admin access to https://github.com/orgs/Cartesian-School/projects/1
- [ ] Have contributor access to repository
- [ ] Team members assigned and ready
- [ ] GitHub workflow files location ready: `.github/workflows/`

---

## ⚡ 5-Minute Setup

### Step 1: Create GitHub Issues (30 min)
1. Go to https://github.com/orgs/Cartesian-School/repositories (select GBSD repo)
2. Click "Issues" → "New Issue"
3. For each issue in ISSUES.md:
   - Copy title from "**Title**:" field
   - Copy description from "**Description**:" section
   - Add acceptance criteria as checklist
   - Add labels from "**Labels**:" field
   - Set priority label
   - Click "Create issue"

**Shortcut**: Import 5 critical issues first (Issues #1-5), then batch-import others

### Step 2: Create Project Board (15 min)
1. Go to https://github.com/orgs/Cartesian-School/projects
2. Click "New project"
3. Name: `GBSD - Microkernel OS Development`
4. Template: Table (or Kanban)
5. Add custom fields:
   - Priority (dropdown: Critical, High, Medium, Low)
   - Effort (dropdown: 1d, 2-3d, 5d, 10d+)
   - Team (dropdown: DevOps, QA, Kernel, Services, Docs)

### Step 3: Add Columns (10 min)
Create 5 columns:
```
Backlog → Ready → In Progress → In Review → Done
```

### Step 4: Create Labels (15 min)
Run this in GitHub or use the UI:
```
Critical (red)
High (orange)
Medium (yellow)
Low (green)
Bug (red)
Feature (blue)
Documentation (purple)
Testing (orange)
Build (dark red)
Security (red)
Kernel (gray)
Services (blue)
Phase-1 (gray)
Phase-2 (gray)
Phase-3 (blue)
Phase-4 (light blue)
Blocked (red)
```

### Step 5: Add GitHub Workflows (30 min)
Create files in `.github/workflows/`:
1. `build.yml` - Compile, test, lint
2. `security.yml` - Dependency audit
3. `docs.yml` - Generate documentation

(See GITHUB_WORKFLOW_SETUP.md for file contents)

---

## 🎯 Critical Path Setup (2 hours)

### For Fast-Track (Just Critical Issues):

1. **Create Issues #1-5 only** (the blockers)
   - Issue #1: Build fix
   - Issue #2: Integration tests
   - Issue #3: Scheduler syscalls
   - Issue #4: Memory management
   - Issue #5: Phase 3 services

2. **Link as dependencies**:
   - Issue #1 → Independent
   - Issue #3 → Depends on #1
   - Issue #4 → Depends on #3
   - Issue #2 → Depends on #4
   - Issue #5 → Depends on #2

3. **Assign team**:
   - Issue #1: DevOps
   - Issue #3-4: Kernel Team
   - Issue #2: QA Team
   - Issue #5: Services Team

4. **Estimate**:
   - Week 1: Issues #1, #3, #4 (6 days)
   - Week 2: Issues #2, #5 (8 days)

---

## 📱 Using the Kanban Board

### Daily Workflow:
```
1. Check "In Progress" column
2. Update issue status in comments
3. Move to "In Review" when PR created
4. Move to "Done" when PR merged
```

### Tracking Progress:
- **Backlog**: All future work
- **Ready**: Approved, ready to start
- **In Progress**: Currently being coded
- **In Review**: PR submitted, awaiting review
- **Done**: Merged and deployed

### Example Flow:
```
Issue #1 (Build fix)
  Backlog → Ready → In Progress → In Review → Done
```

---

## 🔗 Issue Dependency Example

Create these links in GitHub:

```
Issue #1 (Build)
  ├─ Issue #3 depends on this
  │  ├─ Issue #4 depends on this
  │  │  ├─ Issue #2 depends on this
  │  │  │  └─ Issue #5 depends on this

Parallel Track:
Issue #7 (CI/CD) - Independent
Issue #8 (Error handling) - Independent
Issue #9 (Memory bounds) - Independent
```

In GitHub, use "Linked issues" section to set these relationships.

---

## 🏷️ Quick Label Guide

### For Issue #1 (Build):
```
Labels:
- critical
- bug
- build
- blocker
```

### For Issue #3 (Scheduler):
```
Labels:
- critical
- bug
- kernel
- phase-3
```

### For Issue #5 (Services):
```
Labels:
- critical
- implementation
- phase-3
- services
```

### For Issue #7 (CI/CD):
```
Labels:
- high
- feature
- build
- devops
```

---

## 📊 Board Automation Setup

### GitHub Projects Automations:
1. Set "In Progress" status when PR linked
2. Set "Done" status when PR merged
3. Auto-add new issues to Backlog

### Branch Protection Rules:
```
Settings → Branches → Add rule for main/develop:
- Require pull request reviews: 1
- Require status checks to pass: Yes
  ✓ build.yml
  ✓ security.yml
```

---

## 🚀 Go-Live Sequence

### Hour 1: Foundation
- [ ] Create project board
- [ ] Create labels
- [ ] Create columns

### Hour 2: Critical Issues
- [ ] Create Issues #1-5
- [ ] Set dependencies
- [ ] Assign team

### Hour 3: Infrastructure
- [ ] Create .github/workflows/
- [ ] Add build.yml
- [ ] Add security.yml
- [ ] Add branch protection

### Hour 4: Finish
- [ ] Create remaining issues (6-23)
- [ ] Add all issues to board
- [ ] Test workflows
- [ ] Team training

---

## 🎓 Team Training Checklist

Share with team:
- [ ] REVIEW_SUMMARY.md (5 min read)
- [ ] ISSUES.md (10 min overview)
- [ ] GITHUB_WORKFLOW_SETUP.md (technical review)
- [ ] Board walkthrough (10 min demo)
- [ ] Workflow explanation (5 min)

**Total training time**: ~30 minutes

---

## ✅ Verification Checklist

After setup:
- [ ] All 23 issues created
- [ ] Board has 5 columns
- [ ] All labels present
- [ ] Critical path issues linked
- [ ] Team assigned
- [ ] Workflows created and passing
- [ ] Branch protection active
- [ ] First issue (Build) in "Ready"
- [ ] Team notified and trained

---

## 🆘 Troubleshooting

### Issue creation slow?
**Solution**: Batch create 5-10 at a time, don't do all 23 at once

### Workflow not triggering?
**Solution**: Verify `.github/workflows/` path is correct, commit to main/develop

### Board not showing issues?
**Solution**: Manually add via "Add issues" button, or wait 5 mins for sync

### Can't link PRs to issues?
**Solution**: Use "Closes #123" in PR description

---

## 📞 Support

**For questions:**
- Board setup: See GITHUB_WORKFLOW_SETUP.md
- Issues detail: See ISSUES.md
- Executive info: See REVIEW_SUMMARY.md
- Navigation: See PROFESSIONAL_REVIEW_INDEX.md

---

## 🎯 Success Criteria

Setup is complete when:
1. ✅ All 23 issues visible on board
2. ✅ Board organized by priority
3. ✅ Workflows passing
4. ✅ Team can navigate board
5. ✅ Issue #1 assigned and ready to start

**Estimated time to Phase 4 start**: 3-4 weeks after Issue #1 starts

---

**Next**: Start with Issue #1 (Build fix) - 1 day effort

