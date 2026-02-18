# Command Reference

## Overview

jjpr has two main commands:
- `jj-pr mail` - Create/update PRs for your commits
- `jj-pr sync` - Synchronize local commits with remote PR branches

## jj-pr mail

Creates GitHub PR branches and pull requests for all non-empty commits between your working commit (`@`) and the base branch.

### Usage

```bash
jj-pr mail [OPTIONS]
```

### Options

| Option | Short | Description |
|--------|-------|-------------|
| `--base <BRANCH>` | `-b` | Base branch to compare against (e.g., `origin/master`) |

### Behavior

1. **Finds commits**: Gets all non-empty commits from base branch to `@`
2. **Creates branches**: Creates `spr/{owner}/{change-id}` for each commit
3. **Creates PRs**: Creates or updates GitHub pull requests
4. **Sets up stacking**: Each PR targets the previous commit's branch
5. **Tracks state**: Stores PR numbers in commit messages

### Examples

#### Basic Usage

```bash
jj-pr mail
```

Uses auto-detected base branch (tries `origin/master`, then `origin/main`).

#### Specify Base Branch

```bash
jj-pr mail --base origin/develop
```

Creates PRs against `origin/develop` instead of the default.

#### Short Flag

```bash
jj-pr mail -b origin/staging
```

Same as above using the short flag.

### Output

```
📬 Finding commits to mail...
   Base branch: origin/main

Found 3 commit(s) to mail

📝 Processing commit 1/3
   Change ID: abc123...
   Message: Add authentication

   🌿 Creating branch: spr/owner/abc123...
   ✅ Branch created

   📤 Creating pull request...
   🎉 Created PR #42: Add authentication
       https://github.com/owner/repo/pull/42
       Base: main

...
```

### What Gets Created

For a stack like:

```
origin/main
    ↓
 Commit A (change_id: aaa)
    ↓
 Commit B (change_id: bbb)
    ↓
 Commit C (change_id: ccc)
```

jjpr creates:

- **Branch A**: `spr/owner/aaa` → **PR #101** (base: `main`)
- **Branch B**: `spr/owner/bbb` → **PR #102** (base: `spr/owner/aaa`)
- **Branch C**: `spr/owner/ccc` → **PR #103** (base: `spr/owner/bbb`)

Each commit's message is updated:

```
Add authentication

Implements OAuth2 flow.

Pull Request: #42
```

### Edge Cases

#### Empty Commits

Empty commits (no file changes) are skipped:

```
Found 3 commit(s) to mail
Skipping commit 2/3 (empty commit)
```

#### Existing PRs

If a commit already has a PR number, jjpr updates that PR:

```
📝 Processing commit 1/3
   Change ID: abc123...
   Message: Add authentication
   PR: #42 (existing)

   📤 Updating pull request #42...
   ✅ Updated PR #42
```

#### No Commits

If there are no commits between base and `@`:

```
📬 Finding commits to mail...
✅ No commits to mail
```

### Common Errors

#### "Could not auto-detect base branch"

Neither `origin/master` nor `origin/main` exists. Solutions:

```bash
# Push to create main branch
git push -u origin main

# Or specify base explicitly
jj-pr mail --base origin/develop
```

#### "GITHUB_TOKEN or GH_TOKEN environment variable required"

GitHub token not set:

```bash
export GITHUB_TOKEN="github_pat_YOUR_TOKEN_HERE"
```

#### "Failed to parse GitHub owner/repo from remote"

No GitHub remote configured:

```bash
git remote add origin git@github.com:username/repo.git
```

---

## jj-pr sync

Synchronizes local commits with remote PR branches, merging changes from both directions.

### Usage

```bash
jj-pr sync [OPTIONS]
```

### Options

| Option | Short | Description |
|--------|-------|-------------|
| `--base <BRANCH>` | `-b` | Base branch to compare against |
| `--status` | `-s` | Show sync status without performing sync |

### Behavior

For each commit with a PR:

1. **Fetches remote**: Gets latest state of PR branch from GitHub
2. **Detects divergence**: Checks if local or remote (or both) have changed
3. **Merges changes**: Squashes remote changes onto local commit
4. **Handles conflicts**: Guides user through conflict resolution
5. **Pushes updates**: Force-pushes merged result back to GitHub

### Status Mode

With `--status` flag, shows what would be synced without making changes:

```bash
jj-pr sync --status
```

### Examples

#### Basic Sync

```bash
jj-pr sync
```

Syncs all commits with PRs.

#### Status Check

```bash
jj-pr sync --status
```

Shows sync status without syncing.

#### Sync with Different Base

```bash
jj-pr sync --base origin/develop
```

### Output Examples

#### All In Sync

```
🔄 Syncing commits...
   Base branch: origin/main

Found 3 commit(s) to sync

📝 Syncing commit 1/3
   Change ID: abc123...
   Message: Add authentication
   PR: #42
   ✅ In sync

📝 Syncing commit 2/3
   Change ID: bbb456...
   Message: Add validation
   PR: #43
   ✅ In sync

...

🎉 Successfully synced 3 commit(s)
```

#### Remote Has Changes

When Copilot or a collaborator pushes to your PR:

```
📝 Syncing commit 1/3
   Change ID: abc123...
   Message: Add authentication
   PR: #42
   ⬇️  Fetching remote branch... done
   ⬇️  Remote has 2 new commit(s)
   🔄 Merging remote changes into local commit...
   ✅ Successfully merged remote changes
   ⬆️  Pushing to remote...
   ✅ Synced successfully
```

#### Local Has Changes

When you amend a commit locally:

```
📝 Syncing commit 1/3
   Change ID: abc123...
   Message: Add authentication
   PR: #42
   ⬆️  Local has 1 new commit(s)
   ⬆️  Pushing to remote...
   ✅ Synced successfully
```

#### Both Have Changes (No Conflict)

```
📝 Syncing commit 1/3
   Change ID: abc123...
   Message: Add authentication
   PR: #42
   ⚠️  Both local and remote have changes (2 remote, 1 local)
   🔄 Merging remote changes into local commit...
   ✅ Successfully merged remote changes
   ⬆️  Pushing to remote...
   ✅ Synced successfully
```

#### Merge Conflict

```
📝 Syncing commit 1/3
   Change ID: abc123...
   Message: Add authentication
   PR: #42
   ⚠️  Both local and remote have changes (1 remote, 1 local)
   🔄 Merging remote changes into local commit...
   ❌ MERGE CONFLICT DETECTED
      JJ has created conflict markers in your working copy.
      Please:
        1. Resolve the conflicts manually
        2. Run 'jj squash' to finalize the merge
        3. Re-run 'jj-pr sync' to complete synchronization

   Error details: Merge conflict in file.txt

⚠️  Sync completed with 1 error(s)
   0 commit(s) synced successfully
   1 commit(s) need conflict resolution
```

#### Status Mode Output

```bash
jj-pr sync --status
```

```
📊 Checking sync status...
   Base branch: origin/main

Found 3 commit(s) to check

📝 Commit 1/3
   Change ID: abc123...
   Message: Add authentication
   PR: #42
   ✅ In sync

📝 Commit 2/3
   Change ID: bbb456...
   Message: Add validation
   PR: #43
   ⬇️  Remote has 2 new commit(s)
      Run 'jj-pr sync' to pull changes

📝 Commit 3/3
   Change ID: ccc789...
   Message: Add tests
   PR: #44
   ⚠️  Both local and remote have changes (1 remote, 1 local)
      Run 'jj-pr sync' to merge (may require conflict resolution)

📊 Status Summary:
   1 commit(s) in sync
   2 commit(s) need syncing
```

### Conflict Resolution

When sync encounters conflicts:

1. **JJ creates conflict markers** in your working copy:
   ```
   <<<<<<< Conflict 1 of 1
   %%%%%%% Changes from abc123...
   +local changes
   +++++++ Contents of bbb456...
   +remote changes
   >>>>>>> Conflict 1 of 1 ends
   ```

2. **Edit files** to resolve conflicts (remove markers, keep desired changes)

3. **Finalize the merge**:
   ```bash
   jj squash --from <remote-commit> --into <change-id>
   ```

4. **Re-run sync**:
   ```bash
   jj-pr sync
   ```

### Sync States

jjpr detects four sync states:

| State | Symbol | Meaning | Action |
|-------|--------|---------|--------|
| In sync | ✅ | Local and remote identical | None |
| Remote ahead | ⬇️ | Remote has new commits | Pull and merge |
| Local ahead | ⬆️ | Local has new commits | Push to remote |
| Both diverged | ⚠️ | Both have changes | Merge (may conflict) |

### Common Errors

#### "No remote branch found"

The PR branch doesn't exist on GitHub. This happens if:
- You deleted the branch
- The PR was closed and branch deleted
- You haven't run `jj-pr mail` yet

Solution: Run `jj-pr mail` to create the branches.

#### "Failed to squash remote changes"

JJ couldn't automatically merge. This usually means a conflict. Follow the conflict resolution steps above.

#### "Revision doesn't exist"

The commit or remote reference doesn't exist. Make sure:
- You're in a JJ repository: `jj status`
- The commit has a PR: `jj log`
- You've fetched recent changes: `git fetch`

---

## Global Flags

These flags work with all commands:

| Flag | Description |
|------|-------------|
| `--help` | Show help information |
| `--version` | Show version information |

## Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `GITHUB_TOKEN` | GitHub personal access token | Yes |
| `GH_TOKEN` | Alternative to GITHUB_TOKEN | No |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error occurred |

## Tips

### Dry Run

To see what would happen without making changes:

```bash
# See what would be mailed
jj log -r 'origin/main..@'

# See sync status
jj-pr sync --status
```

### Workflow Integration

Typical workflow:

```bash
# Make changes
jj describe -m "Add feature"

# Create/update PRs
jj-pr mail

# (Time passes, reviews happen)

# Sync changes from remote
jj-pr sync

# Make more changes
jj describe -m "Add feature

Address review comments"

# Update PRs
jj-pr mail

# Final sync
jj-pr sync
```

### Base Branch Persistence

Once you specify a base branch, jjpr remembers it:

```bash
# First time
jj-pr mail --base origin/develop

# Subsequent runs use same base
jj-pr mail
jj-pr sync
```

To change:

```bash
jj config set --repo jjpr.baseBranch origin/main
```

Or override per-command:

```bash
jj-pr mail --base origin/staging
```

## Next Steps

- [Sync Deep Dive](sync.md) - Detailed sync behavior
- [Workflows](workflows.md) - Common usage patterns
- [Overview](overview.md) - How jjpr works
