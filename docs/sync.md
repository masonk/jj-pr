# Sync Deep Dive

## Overview

The `jj-pr sync` command is the heart of jjpr's bidirectional workflow. It ensures your local commits stay synchronized with their corresponding GitHub PR branches, handling changes from multiple sources.

## Why Sync Matters

In modern development workflows, PR branches can change from multiple sources:

### Sources of Remote Changes

1. **GitHub Copilot**: Suggested fixes applied directly to PRs
2. **Collaborators**: Team members pushing to your PR branch
3. **Review Bots**: Automated formatting, linting, or security fixes
4. **Manual GitHub Edits**: Quick fixes made through the GitHub UI
5. **Merge Commits**: Merges from base branch to resolve conflicts

### Sources of Local Changes

1. **Amendments**: You update commits based on review feedback
2. **Rebases**: You reorder or restructure your stack
3. **Conflict Resolution**: You resolve conflicts locally
4. **New Changes**: You add new functionality to existing commits

## The Sync Algorithm

### Phase 1: Discovery

```
For each commit in the stack:
  1. Check if commit has a PR number
  2. If no PR → skip (not mailed yet)
  3. If has PR → construct branch name (spr/{owner}/{change-id})
  4. Check if branch exists on remote
  5. If no remote branch → skip
  6. Fetch remote branch
```

### Phase 2: Divergence Detection

```
For each fetched branch:
  1. Get remote commit SHA
  2. Get local commit SHA
  3. If SHAs match → skip (already in sync)
  4. Check commits between local and remote
  5. Check commits between remote and local
  6. Classify divergence state
```

### Divergence States

| Remote Changes | Local Changes | State | Symbol |
|----------------|---------------|-------|--------|
| No | No | In sync | ✅ |
| Yes | No | Remote ahead | ⬇️ |
| No | Yes | Local ahead | ⬆️ |
| Yes | Yes | Both diverged | ⚠️ |

### Phase 3: Synchronization

```
Based on divergence state:

If Remote Ahead (⬇️):
  1. Squash remote changes onto local commit
  2. If conflict → abort with guidance
  3. If success → push to remote

If Local Ahead (⬆️):
  1. Push local changes to remote

If Both Diverged (⚠️):
  1. Squash remote changes onto local commit
  2. If conflict → abort with guidance
  3. If success → push merged result
```

## Detailed Examples

### Example 1: Remote Changes Only (Copilot Fix)

**Initial State:**
```
Local:  A---B---C  (your commits)
         \
Remote:  A---X   (Copilot added fix X to PR)
```

**What Happens:**

1. Fetch detects remote has commit X
2. JJ squashes X onto local commit A
3. Result pushed back to remote

**Final State:**
```
Local:  A'--B'--C'  (A' includes Copilot's fix)
         \
Remote:  A'         (matches local A')
```

Note: JJ automatically propagates the change to B' and C' through its conflict resolution.

**Command Output:**
```bash
$ jj-pr sync
🔄 Syncing commits...

📝 Syncing commit 1/3
   PR: #42
   ⬇️  Remote has 1 new commit(s)
   🔄 Merging remote changes into local commit...
   ✅ Successfully merged remote changes
   ⬆️  Pushing to remote...
   ✅ Synced successfully
```

### Example 2: Local Changes Only (Amendment)

**Initial State:**
```
Local:  A---B---C  (you amended A)
         \
Remote:  A         (old version)
```

**What Happens:**

1. Fetch shows remote matches old version of A
2. Local has diverged (different tree)
3. Push updated A to remote

**Final State:**
```
Local:  A---B---C  (same)
         \
Remote:  A         (updated to match local)
```

**Command Output:**
```bash
$ jj-pr sync
🔄 Syncing commits...

📝 Syncing commit 1/3
   PR: #42
   ⬆️  Local has 1 new commit(s)
   ⬆️  Pushing to remote...
   ✅ Synced successfully
```

### Example 3: Both Changed (Non-Conflicting)

**Initial State:**
```
Local:  Added feature.rs
Remote: Added tests.rs (same base)
```

Different files modified - no conflict.

**What Happens:**

1. JJ squashes remote changes onto local
2. No conflicts (different files)
3. Merged result has both changes
4. Push to remote

**Final State:**
```
Merged: Has both feature.rs and tests.rs
```

**Command Output:**
```bash
$ jj-pr sync
🔄 Syncing commits...

📝 Syncing commit 1/3
   PR: #42
   ⚠️  Both local and remote have changes (1 remote, 1 local)
   🔄 Merging remote changes into local commit...
   ✅ Successfully merged remote changes
   ⬆️  Pushing to remote...
   ✅ Synced successfully
```

### Example 4: Both Changed (Conflicting)

**Initial State:**
```
Local:  Modified line 10 of auth.rs → "local version"
Remote: Modified line 10 of auth.rs → "remote version"
```

Same line modified - conflict!

**What Happens:**

1. JJ attempts to squash
2. JJ detects conflict
3. JJ creates conflict markers
4. Sync aborts with guidance

**Conflict Markers:**
```rust
// auth.rs
fn authenticate(user: &str) -> Result<Token> {
    <<<<<<< Conflict 1 of 1
    %%%%%%% Changes from abc123 (local)
    let token = create_token(user, "local version");
    +++++++ Contents of xyz789 (remote)
    let token = create_token(user, "remote version");
    >>>>>>> Conflict 1 of 1 ends
    Ok(token)
}
```

**Command Output:**
```bash
$ jj-pr sync
🔄 Syncing commits...

📝 Syncing commit 1/3
   PR: #42
   ⚠️  Both local and remote have changes (1 remote, 1 local)
   🔄 Merging remote changes into local commit...
   ❌ MERGE CONFLICT DETECTED
      JJ has created conflict markers in your working copy.
      Please:
        1. Resolve the conflicts manually
        2. Run 'jj squash' to finalize the merge
        3. Re-run 'jj-pr sync' to complete synchronization

   Error details: Merge conflict in auth.rs

⚠️  Sync completed with 1 error(s)
   0 commit(s) synced successfully
   1 commit(s) need conflict resolution
```

**Resolution Steps:**

1. **Edit the file:**
   ```rust
   // auth.rs
   fn authenticate(user: &str) -> Result<Token> {
       // Keep the version you want:
       let token = create_token(user, "resolved version");
       Ok(token)
   }
   ```

2. **Finalize the merge:**
   ```bash
   jj squash --from xyz789 --into abc123
   ```

3. **Re-run sync:**
   ```bash
   jj-pr sync
   ```

4. **Success:**
   ```bash
   📝 Syncing commit 1/3
      PR: #42
      ⬆️  Pushing to remote...
      ✅ Synced successfully
   ```

## Status Mode

Use `--status` to preview sync operations without making changes.

### When to Use Status

- **Before syncing**: See what's out of sync
- **After PR reviews**: Check if Copilot made changes
- **Before leaving**: Ensure everything is pushed
- **Debugging**: Understand sync state

### Status Output

```bash
$ jj-pr sync --status
📊 Checking sync status...
   Base branch: origin/main

Found 3 commit(s) to check

📝 Commit 1/3
   Change ID: abc123
   Message: Add authentication
   PR: #42
   ✅ In sync

📝 Commit 2/3
   Change ID: def456
   Message: Add validation
   PR: #43
   ⬇️  Remote has 2 new commit(s)
      Run 'jj-pr sync' to pull changes

📝 Commit 3/3
   Change ID: ghi789
   Message: Add tests
   PR: #44
   ⬆️  Local has 1 new commit(s)
      Run 'jj-pr sync' to push changes

📊 Status Summary:
   1 commit(s) in sync
   2 commit(s) need syncing
```

## Advanced Scenarios

### Scenario 1: Stacked Changes Propagation

When you sync commit A in a stack, changes automatically propagate:

**Before:**
```
A (synced) ← B ← C
```

**After syncing A with remote changes:**
```
A' (merged) ← B' (auto-updated) ← C' (auto-updated)
```

JJ's conflict resolution automatically rebases B and C on top of A'.

### Scenario 2: Multiple Remote Commits

Remote can have multiple commits since your last sync:

```
Local:  A
         \
Remote:  A---X---Y---Z  (3 new commits)
```

Sync squashes all three (X, Y, Z) onto local A in one operation.

### Scenario 3: Base Branch Updates

If someone updates the base branch of your PR on GitHub:

```
Original:  PR #43 base: main
Updated:   PR #43 base: spr/owner/abc123
```

Next `jj-pr mail` will detect the change and update the PR base to match your local stack structure.

### Scenario 4: Deleted PR Branches

If a PR branch is deleted (e.g., after merging):

```bash
$ jj-pr sync
📝 Syncing commit 1/3
   PR: #42
   ⏭️  No remote branch found

📝 Syncing commit 2/3
   PR: #43
   ✅ Synced successfully
```

Sync skips deleted branches. You can clean up the PR number:

```bash
jj describe -m "$(jj log -r abc123 -T description | sed '/Pull Request:/d')"
```

## Sync Guarantees

### What Sync Ensures

1. ✅ **Local ↔ Remote consistency**: After successful sync, local and remote match
2. ✅ **Stack integrity**: Downstream commits are updated automatically
3. ✅ **No data loss**: Conflict detection prevents silent overwrites
4. ✅ **Idempotency**: Running sync multiple times is safe

### What Sync Doesn't Do

1. ❌ **Auto-resolve conflicts**: You must resolve conflicts manually
2. ❌ **Rebase on base**: Doesn't update stack against new base branch commits
3. ❌ **Delete branches**: Doesn't clean up merged PR branches
4. ❌ **Update PR metadata**: Doesn't change PR title, description, etc.

## Performance

### Parallelization

Currently, sync operates **sequentially** on each commit. Future versions may parallelize independent syncs.

### Network Usage

Sync makes these network calls per commit:
- 1 fetch (git)
- 1 push (git, if changes exist)

Total: ~2 round-trips per commit with changes.

### Large Stacks

For stacks with 10+ commits:
- Consider using `--status` first
- Sync may take 30-60 seconds
- Most time is network I/O

## Error Handling

### Transient Errors

Network errors are reported but don't affect subsequent commits:

```bash
📝 Syncing commit 2/3
   PR: #43
   ⚠️  Failed to fetch: network error
   Skipping...
```

Re-run sync to retry.

### Fatal Errors

Some errors stop sync entirely:

- Invalid token
- Repository not found
- Permission denied

Fix the underlying issue and re-run.

### Partial Sync

If sync fails partway through:

```bash
⚠️  Sync completed with 1 error(s)
   2 commit(s) synced successfully
   1 commit(s) need conflict resolution
```

The successfully synced commits are pushed. Fix the failing commit and re-run.

## Best Practices

### 1. Sync Often

Sync after:
- Reviewing PRs on GitHub
- Receiving notifications about changes
- Before starting new work
- Before mailing updates

### 2. Use Status Mode

Check status before syncing:

```bash
jj-pr sync --status  # Check first
jj-pr sync           # Then sync
```

### 3. Resolve Conflicts Immediately

Don't let conflicts accumulate:

```bash
# Conflict detected
jj-pr sync
# → Resolve immediately

# Edit files
vim auth.rs

# Finalize
jj squash --from <remote> --into <local>

# Complete sync
jj-pr sync
```

### 4. Keep Stacks Small

Smaller stacks = faster syncs = less conflict potential:
- 3-5 commits per stack: optimal
- 10+ commits: consider splitting

### 5. Communicate Changes

If you force-push to a collaborator's PR branch, let them know:

```bash
# After syncing a collaborator's PR
jj-pr sync

# Send them a note
echo "Synced your PR #43 with my changes"
```

## Troubleshooting

### "Failed to squash remote changes"

**Cause**: Merge conflict or JJ error.

**Solution**:
1. Check JJ status: `jj status`
2. Look for conflict markers
3. Resolve conflicts manually
4. Run `jj squash` to finalize
5. Re-run `jj-pr sync`

### "Remote branch commit doesn't match expected"

**Cause**: Branch was force-pushed between fetch and push.

**Solution**: Re-run sync. It will fetch the latest state.

### "Both have changes but squash succeeded unexpectedly"

**Cause**: Changes were in different parts of the code.

**Verification**: Review the merged result to ensure correctness.

### Sync Seems Stuck

**Symptoms**: No output for a long time.

**Causes**:
- Large file fetch
- Slow network
- GitHub API rate limiting

**Solution**:
1. Wait a few minutes
2. Check network connectivity
3. Cancel (Ctrl+C) and retry
4. Check GitHub API rate limit: `curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/rate_limit`

## Future Enhancements

Planned improvements:

1. **Parallel sync**: Sync independent commits concurrently
2. **Interactive mode**: Preview and choose which commits to sync
3. **Auto-rebase**: Optionally rebase stack on updated base branch
4. **Conflict resolution UI**: Interactive conflict resolution
5. **Partial sync**: Sync specific commits only

## Next Steps

- [Command Reference](commands.md) - Full command documentation
- [Workflows](workflows.md) - Common usage patterns
- [Overview](overview.md) - How jjpr works
