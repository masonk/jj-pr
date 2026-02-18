# Common Workflows

This guide covers typical jjpr workflows for different scenarios.

## Table of Contents

1. [Basic Workflow](#basic-workflow)
2. [Creating a Stack](#creating-a-stack)
3. [Updating After Review](#updating-after-review)
4. [Handling Copilot Suggestions](#handling-copilot-suggestions)
5. [Reordering Commits](#reordering-commits)
6. [Splitting a Commit](#splitting-a-commit)
7. [Merging PRs](#merging-prs)
8. [Collaborating on a PR](#collaborating-on-a-pr)
9. [Recovering from Errors](#recovering-from-errors)

---

## Basic Workflow

### Creating and Mailing a Single Change

```bash
# Start with a clean working copy
jj status

# Make changes
echo "new feature" > feature.rs
jj describe -m "Add new feature

This implements the new feature X."

# Create PR
jj-pr mail
# Output: Created PR #42

# Continue working...
```

### Syncing After Reviews

```bash
# Check what's changed
jj-pr sync --status
# Output: Remote has 1 new commit (Copilot fix)

# Sync changes
jj-pr sync
# Output: Synced successfully

# Verify
jj log
# Output shows updated commit
```

---

## Creating a Stack

### Building a Feature Stack

**Scenario**: Implement authentication in three stages.

```bash
# Stage 1: Database schema
jj new
echo "CREATE TABLE users..." > schema.sql
jj describe -m "Add user schema"

# Stage 2: Auth logic
jj new
cat > auth.rs << 'EOF'
fn authenticate() { }
EOF
jj describe -m "Add authentication logic"

# Stage 3: API endpoints
jj new
cat > api.rs << 'EOF'
fn login_endpoint() { }
EOF
jj describe -m "Add login endpoints"

# Mail entire stack
jj-pr mail
```

**Output**:
```
Created PR #101: Add user schema (base: main)
Created PR #102: Add authentication logic (base: spr/user/schema-change-id)
Created PR #103: Add login endpoints (base: spr/user/auth-change-id)
```

**Result**:
- PR #101 can be reviewed/merged independently
- PR #102 builds on #101
- PR #103 builds on #102

### Viewing the Stack

```bash
# View all commits
jj log -r 'origin/main..@'

# With graph
jj log -r 'origin/main..@' --graph
```

---

## Updating After Review

### Amending Based on Feedback

**Scenario**: Reviewer asks you to improve error handling in commit B.

```bash
# Check current state
jj log

# Find the commit (e.g., change_id = bbb456)
jj log -r bbb456

# Edit that commit
jj edit bbb456

# Make changes
vim auth.rs  # Add better error handling

# Update description (optional)
jj describe -m "Add authentication logic

Improved error handling based on review feedback"

# Return to head
jj edit @

# Update PRs
jj-pr mail
# Output: Updated PR #102

# Push changes
jj-pr sync
```

**What Happens**:
- Commit B is updated
- JJ automatically rebases commit C on updated B
- PR #102 is updated with new changes
- PR #103 is updated (if C changed)

---

## Handling Copilot Suggestions

### Accepting Copilot Fixes

**Scenario**: GitHub Copilot suggests a fix to PR #42.

```bash
# You see notification: "Copilot suggested changes to PR #42"

# Check status
jj-pr sync --status
# Output:
#   Commit 1/3
#   PR: #42
#   ⬇️  Remote has 1 new commit
#      Run 'jj-pr sync' to pull changes

# Pull changes
jj-pr sync
# Output:
#   Merging remote changes...
#   ✅ Successfully merged
#   ✅ Synced successfully

# Verify the changes
jj diff -r <change-id>

# If changes look good, continue
# If not, amend and re-mail:
jj edit <change-id>
# Make corrections
jj describe -m "Add feature

Fixed Copilot suggestion"
jj-pr mail
```

### Multiple Copilot Suggestions

```bash
# Copilot suggested changes to multiple PRs

# Check all at once
jj-pr sync --status
# Output:
#   Commit 1/3 (PR #42): ⬇️  Remote has 1 new commit
#   Commit 2/3 (PR #43): ⬇️  Remote has 2 new commits
#   Commit 3/3 (PR #44): ✅ In sync

# Sync all
jj-pr sync
# Output: Synced 2 commit(s)

# Review all changes
jj log -r 'origin/main..@' --patch
```

---

## Reordering Commits

### Moving a Commit Earlier in Stack

**Scenario**: You have A ← B ← C but realize C should come before B.

```bash
# Current state
jj log -r 'origin/main..@'
# Output:
#   @  ccc789 C
#   ○  bbb456 B
#   ○  aaa123 A

# Reorder: Insert C after A
jj rebase -r ccc789 -d aaa123

# New state
jj log -r 'origin/main..@'
# Output:
#   @  bbb456 B (updated)
#   ○  ccc789 C
#   ○  aaa123 A

# Update PRs with new bases
jj-pr mail
# Output:
#   Updated PR #44 (C) base: A's branch
#   Updated PR #43 (B) base: C's branch

# Sync
jj-pr sync
```

### Moving Multiple Commits

```bash
# Move commits B and C together
jj rebase -s bbb456 -d some_other_commit

# Update PRs
jj-pr mail && jj-pr sync
```

---

## Splitting a Commit

### Breaking One Commit Into Two

**Scenario**: Commit A does too much. Split into A1 (schema) and A2 (logic).

```bash
# Current state: A ← B ← C
# Want: A1 ← A2 ← B ← C

# Edit the commit
jj edit aaa123

# Split interactively
jj split
# Output: Interactive editor shows changes
# Move schema changes to first commit, logic to second

# Or split by path
jj split schema.sql
# Creates two commits:
#   - Commit with schema.sql
#   - Commit with everything else

# Describe the splits
jj describe -r @- -m "Add user schema"
jj describe -r @ -m "Add authentication logic"

# Return to head
jj edit @~  # or use change ID

# Update PRs
jj-pr mail
# Output:
#   Created PR #105 for new commit (A1)
#   Updated PR #102 (was A, now A2)
#   Updated PR #103 (B) base: A2's branch

# Sync all
jj-pr sync
```

---

## Merging PRs

### Bottom-Up Merge Strategy

**Recommended**: Merge PRs from bottom to top of stack.

```bash
# Stack: A (#101) ← B (#102) ← C (#103)

# 1. Merge A first
# On GitHub: Merge PR #101 into main

# 2. Update local
git fetch origin main
jj git import

# 3. Update stack bases
jj-pr mail
# Output:
#   Updated PR #102 base: main (was A's branch)
#   Updated PR #103 base: B's branch (unchanged)

# 4. Merge B
# On GitHub: Merge PR #102 into main

# 5. Update again
git fetch origin main
jj git import
jj-pr mail
# Output:
#   Updated PR #103 base: main

# 6. Merge C
# On GitHub: Merge PR #103 into main

# 7. Clean up local
jj abandon aaa123 bbb456 ccc789
# Or keep them for history
```

### Handling Merge Conflicts

```bash
# PR #101 merged, but #102 now has conflicts with main

# Update local main
git fetch origin main
jj git import

# Rebase B onto new main
jj rebase -r bbb456 -d origin/main

# Resolve conflicts if any
# JJ will show conflict markers
vim conflicted_file.rs

# Update PR
jj-pr mail

# PR #102 now targets main with conflicts resolved
```

---

## Collaborating on a PR

### Your Teammate Pushes to Your PR

**Scenario**: You asked teammate to help with PR #42. They pushed commits.

```bash
# Teammate pushes to spr/you/aaa123

# Check status
jj-pr sync --status
# Output:
#   Commit 1/3 (PR #42): ⬇️  Remote has 3 new commits

# Pull their changes
jj-pr sync
# Output:
#   Merging remote changes...
#   ✅ Merged successfully

# Review what they changed
jj diff -r aaa123

# Make additional changes
jj edit aaa123
vim feature.rs
jj describe -m "Add feature

Updated with teammate's feedback"

# Push back
jj-pr mail && jj-pr sync
```

### Pushing to Someone Else's PR

```bash
# Your teammate has PR #99 you want to help with
# Branch: spr/teammate/zzz999

# Fetch their branch
git fetch origin spr/teammate/zzz999

# Import to JJ
jj git import

# Create new commit on top
jj new zzz999
# Make changes
vim file.rs
jj describe -m "Help with feature X"

# Push to their branch (NOT as a new PR)
git push origin HEAD:spr/teammate/zzz999

# Tell them to sync
# They run: jj-pr sync
```

---

## Recovering from Errors

### Accidentally Closed a PR

```bash
# You accidentally closed PR #42

# Reopen on GitHub
# Go to PR → Reopen

# Sync to confirm
jj-pr sync --status
# Output: PR #42 is open

# Continue working
```

### Force-Pushed Over Your Stack

**Scenario**: Someone force-pushed to main, and your stack is now out of sync.

```bash
# Update main
git fetch origin main
jj git import

# Rebase entire stack
jj rebase -r 'origin/main..@' -d origin/main

# Update all PRs
jj-pr mail

# Force-push all
jj-pr sync
```

### Lost PR Number in Commit

**Scenario**: Commit message lost "Pull Request: #42" line.

```bash
# Find the commit
jj log -r aaa123

# Re-add PR number
jj describe -r aaa123 -m "$(jj log -r aaa123 -T description)

Pull Request: #42"

# Verify
jj log -r aaa123

# Re-sync
jj-pr sync
```

### Conflicts After Sync

```bash
# Sync detected conflicts
jj-pr sync
# Output: ❌ MERGE CONFLICT DETECTED

# Check conflicts
jj status
# Output: Working copy has conflicts

# View conflicts
cat conflicted_file.rs
# Output: Shows conflict markers

# Resolve
vim conflicted_file.rs
# Remove markers, choose correct version

# Finalize merge
jj squash --from <remote-commit> --into <local-change-id>

# Complete sync
jj-pr sync
# Output: ✅ Synced successfully
```

---

## Advanced Patterns

### Daily Workflow

**Morning**:
```bash
# Check overnight changes
jj-pr sync --status

# Sync if needed
jj-pr sync

# Check what you're working on
jj log
```

**During Development**:
```bash
# Make changes
jj new
vim code.rs
jj describe -m "Add feature"

# Create PR
jj-pr mail

# Continue with next feature
jj new
# ...
```

**Before Leaving**:
```bash
# Ensure everything is pushed
jj-pr sync --status

# Push if needed
jj-pr sync

# Check stack
jj log -r 'origin/main..@'
```

### CI/CD Integration

**Before pushing**:
```bash
# Run tests locally
cargo test

# Mail PRs
jj-pr mail

# Wait for CI
# GitHub Actions run on PRs

# If CI fails, fix and update
jj edit <failing-commit>
vim fix.rs
jj-pr mail
jj-pr sync
```

### Large Stack Management

```bash
# Create epic branch for tracking
jj branch create epic/authentication

# Build stack
jj new
# ... create multiple commits ...

# Mail as stack
jj-pr mail

# Track progress with labels
# Use GitHub labels: "epic:authentication"

# Merge progressively
# Merge PRs one by one from bottom
```

---

## Tips and Tricks

### Quick Status Check

```bash
# Alias in ~/.bashrc
alias jss="jj-pr sync --status"
alias jm="jj-pr mail"
alias js="jj-pr sync"

# Use
jss  # Quick status
jm   # Mail
js   # Sync
```

### Pre-Mail Checks

```bash
# Before mailing, ensure quality
cargo fmt
cargo clippy
cargo test

# Then mail
jj-pr mail
```

### Reviewing Your Stack

```bash
# See all changes
jj diff -r 'origin/main..@'

# See per-commit
jj log -r 'origin/main..@' --patch

# Summary
jj log -r 'origin/main..@' --summary
```

---

## Next Steps

- [Sync Deep Dive](sync.md) - Understand sync behavior
- [Commands](commands.md) - Full command reference
- [Overview](overview.md) - How jjpr works
