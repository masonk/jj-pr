# jjpr Overview

## What is jjpr?

jjpr is a command-line utility that enables efficient **stacked pull request workflows** with [Jujutsu (jj)](https://jj-vcs.github.io/jj/) version control and GitHub.

## What are Stacked PRs?

Stacked PRs allow you to create multiple dependent pull requests in a chain:

```
origin/main
    ↓
   PR A (feature foundation)
    ↓
   PR B (builds on A)
    ↓
   PR C (builds on B)
```

Benefits:
- **Faster reviews**: Smaller, focused PRs are easier to review
- **Parallel work**: Continue building while waiting for reviews
- **Logical separation**: Each PR represents one logical change
- **Independent merging**: PRs can be merged in order as they're approved

## Why Jujutsu?

Jujutsu's change-based model (vs Git's commit-based model) makes stacked PRs natural:

- **Stable change IDs**: Changes maintain identity across rebases
- **Automatic conflict resolution**: JJ propagates changes through the stack
- **Easy reordering**: Rearrange changes without complex rebasing
- **Colocated with Git**: Works alongside your Git workflow

## How jjpr Works

### State Tracking

jjpr tracks PR numbers directly in commit messages:

```
Add user authentication

Implements OAuth2 flow for user login.

Pull Request: #123
```

This approach:
- ✅ No external database needed
- ✅ Survives rebases and amendments
- ✅ Visible in both JJ and Git
- ✅ Simple and transparent

### Core Commands

#### `jj-pr mail`

Creates GitHub PR branches for commits between your working commit (`@`) and the base branch:

```bash
jj-pr mail
```

What it does:
1. Finds all non-empty commits between base and `@`
2. Creates a PR branch for each commit: `spr/{owner}/{change-id}`
3. Creates or updates GitHub PRs
4. Sets up correct base branches (stacking)
5. Stores PR numbers in commit messages

#### `jj-pr sync`

Synchronizes local commits with remote PR branches:

```bash
jj-pr sync
```

What it does:
1. Fetches all PR branches from GitHub
2. Detects divergence (local vs remote)
3. Merges remote changes into local commits
4. Pushes updated commits back to GitHub
5. Handles merge conflicts gracefully

#### `jj-pr sync --status`

Shows sync status without making changes:

```bash
jj-pr sync --status
```

Output shows which commits:
- ✅ Are in sync
- ⬇️ Have remote changes (from Copilot, collaborators, etc.)
- ⬆️ Have local changes
- ⚠️ Have both (may need conflict resolution)

## Key Features

### 1. Automatic Stacking

jjpr automatically sets up the correct base branch for each PR:

```
Commit A → PR targets main
Commit B (child of A) → PR targets A's branch
Commit C (child of B) → PR targets B's branch
```

### 2. Bidirectional Sync

Sync handles changes from multiple sources:

- **Remote changes**: Copilot suggestions, collaborator pushes, review bot fixes
- **Local changes**: Your amendments and updates
- **Both**: Merges changes automatically or prompts for conflict resolution

### 3. Smart Base Branch Detection

jjpr auto-detects your base branch:

1. Tries `origin/master`
2. Falls back to `origin/main`
3. Prompts if neither exists
4. Remembers your choice in `.jj/repo/config.toml`

Override with `--base` flag:

```bash
jj-pr mail --base origin/develop
```

### 4. Conflict Resolution

When sync encounters conflicts:

1. JJ creates conflict markers in your working copy
2. jjpr displays clear resolution steps
3. You resolve conflicts manually
4. Re-run `jj-pr sync` to complete

## Architecture

```
┌─────────────────────────────────────────────────┐
│ Your Working Directory                           │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐  │
│  │ Commit A │ ←  │ Commit B │ ←  │ Commit C │  │
│  │ (change) │    │ (change) │    │ (change) │  │
│  └──────────┘    └──────────┘    └──────────┘  │
│      ↓                ↓                ↓         │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐  │
│  │ PR #123  │    │ PR #124  │    │ PR #125  │  │
│  └──────────┘    └──────────┘    └──────────┘  │
└─────────────────────────────────────────────────┘
                      ↕
┌─────────────────────────────────────────────────┐
│ GitHub                                           │
│  ┌────────────────┐  ┌────────────────┐         │
│  │ PR #123        │  │ PR #124        │         │
│  │ base: main     │  │ base: spr/.../A│         │
│  │ head: spr/.../A│  │ head: spr/.../B│         │
│  └────────────────┘  └────────────────┘         │
└─────────────────────────────────────────────────┘
```

## Comparison to Other Tools

| Feature | jjpr | git-stack | gh-stack | Graphite |
|---------|--------|-----------|----------|----------|
| VCS | Jujutsu | Git | Git | Git |
| PR Creation | ✅ | ✅ | ✅ | ✅ |
| Bidirectional Sync | ✅ | ❌ | ❌ | ✅ |
| Change IDs | ✅ (native) | ❌ | ❌ | ✅ (metadata) |
| State Storage | Commit msgs | Git notes | External DB | External DB |
| Conflict Resolution | Interactive | Manual | Manual | Interactive |

## Next Steps

- [Installation Guide](installation.md) - Get started with jjpr
- [Command Reference](commands.md) - Detailed command documentation
- [Sync Guide](sync.md) - Deep dive into sync functionality
- [Workflows](workflows.md) - Common usage patterns
- [GitHub Setup](github-setup.md) - Authentication and permissions
