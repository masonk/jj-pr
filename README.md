# jjpr: A Rust utility for stacked PRs with Jujutsu

A utility that facilitates stacked pull requests on Github when using the Jujutsu version control system locally.

## Overview

GitHub has a branch-centric workflow for pull requests. Every pull request must be its own named branch.

There's a well-posed argument against branch-based workflows in [SpaceDentist's SPR repo](https://github.com/spacedentist/spr/blob/master/docs/README.md)

Commit-based code review is a better model because both commits and PRs need to be *atomic*.

Atomicity means that each individual commit should "have a coherent thesis and be a complete change in and of itself". A "complete change" means that the commit leaves the repo in a state where it has a new piece of functionality, the software works, does not break CI workflows or cause other inconveniences for other developers. Meanwhile, a code reviewer should *also* should be presented with changes which "have a coherent thesis" and "are a complete change in of itself". After all, some of a code reviewer's most important jobs are to determine whether a changeset's thesis makes sense, whether the changeset accomplishes its thesis, and whether the changest breaks anything or has unintended consequences. How could a reviewer do these jobs if not presented with a complete change which - at least ostensibly - has a coherent thesis? These remarks imply that an ideal unit of code review precisely matches the definition of an ideal commit.

Atomicity also implies that a commit should be indivisible - if it's possible to divide a commit into two subcommits which each stand on their own with a coherent thesis, then that should be done. Once again, this property is precisely the one we desire in code review, since the difficulity of code reviews scales superlinearly with the size of the changeset under review. Two 150 line changests that have all the properties of atomicity are much easier to review than a 300 line changest (or even a 200 line changeset) which has two theses. 

## Installation

Pull source and install using cargo.

```sh
jj git clone git@github.com:masonk/jj-pr.git
cd jj-spr
cargo install --path jj-spr
```

## Commands

### `jj-pr mail`

Creates PR branches for all nonempty commits between `@` (current commit) and the base branch.

- Automatically creates branch names based on change IDs (format: `spr/<username>/<change_id>`)
- Creates stacked PRs where each PR's base is the previous commit's branch
- Updates existing PRs if they already exist
- Handles proper parent-child relationships for stacked PRs

**Examples:**
```bash
# Auto-detect base branch (tries origin/master, then origin/main)
jj-pr mail

# Explicitly specify base branch
jj-pr mail --base origin/main

# Short form
jj-pr mail -b origin/main
```

### `jj-pr sync`

Pulls down any new changes from PR branches and synchronizes them with local commits.

- Fetches remote changes for each tracked commit
- Squashes remote-only changes onto local commits if necessary
- Resets PR branches to point at the latest version of local commits
- Two-way propagation: handles changes made on GitHub (e.g., by AI review bots)

**Examples:**
```bash
# Use saved or auto-detected base branch
jj-pr sync

# Explicitly specify base branch
jj-pr sync --base origin/main
```

## Installation

### Prerequisites

- Rust toolchain (install from [rustup.rs](https://rustup.rs))
- Jujutsu VCS (`jj`) installed and in PATH
- Git repository colocated with Jujutsu
- GitHub personal access token

### Build from source

```bash
cargo build --release
```

The binary will be available at `target/release/jj-pr`.

### Configuration

#### GitHub Token

Set your GitHub token as an environment variable:

```bash
export GITHUB_TOKEN=ghp_your_token_here
# or
export GH_TOKEN=ghp_your_token_here
```

You can also add this to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.).

#### Base Branch

The base branch is the upstream branch to compare against (typically `origin/master` or `origin/main`).

**Auto-detection:** On first run, `jj-pr` will:
1. Try `origin/master`
2. Fall back to `origin/main`
3. Prompt you if neither exists

**Manual configuration:** You can explicitly set the base branch:

```bash
# Via CLI flag (also saves for future use)
jj-pr mail --base origin/main

# Or set it directly in config
jj config set --repo jjpr.baseBranch origin/main

# View current setting
jj config get jjpr.baseBranch
```

The base branch choice is saved in your repository's `.jj/repo/config.toml` and remembered for future commands.

## Testing

### Unit and Integration Tests

Run the basic integration tests (no GitHub API required):

```bash
cargo test
```

The test suite includes:
- Basic repository setup and JJ integration
- Stacked PR creation (A targets master, B targets A)
- PR number tracking in commit messages
- Branch management and Git operations
- Local amendment and change propagation
- Remote change detection

### GitHub Integration Tests

These tests create **real PRs** on GitHub to verify the complete workflow.

**Prerequisites:**
1. **SSH keys configured** for GitHub (for git operations)
   ```bash
   # Test SSH connection
   ssh -T git@github.com
   # Should show: "Hi username! You've successfully authenticated..."
   ```

2. **GitHub personal access token** (for API calls)
   - Go to: GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
   - Create token with **repo** scope (full control of private repositories)

3. **Dedicated test repository** with write access

**Running the tests:**
```bash
# Set up test environment
export GITHUB_TOKEN=ghp_your_token_here
export JJPR_TEST_REPO=owner/test-repo  # Use a dedicated test repo!

# Run GitHub integration tests
cargo test --test github_integration_test -- --ignored
```

**What these tests verify:**
- Real PR creation via GitHub API
- **Stacked PR base branch targeting** (PR for B targets PR branch for A)
- PR updates after local amendments
- PR number persistence across operations
- Complete end-to-end workflow

**⚠️ Important:**
- Use a **dedicated test repository**! These tests create real PRs and branches.
- Tests use **SSH for git operations** and **token for GitHub API**.
- The test repo will have test branches and PRs created (cleaned up automatically).

### Test Categories

| Test File | Type | Requires | What It Tests |
|-----------|------|----------|---------------|
| `tests/integration_test.rs` | Local | JJ + Git | Core logic without GitHub API |
| `tests/github_integration_test.rs` | GitHub | Token + Test Repo | Real PR creation and stacking |

## Usage

1. Make sure you're in a Git repository that's colocated with Jujutsu
2. Create commits in Jujutsu as normal
3. Run `jj-pr mail` to create PR branches and PRs on GitHub
4. Make changes on GitHub or locally
5. Run `jj-pr sync` to synchronize changes between local and remote

## How it works

### State Tracking via Commit Messages

`jj-pr` tracks the association between commits and PRs by storing the PR number directly in commit messages:

```
Add user authentication

Implement JWT-based authentication with refresh tokens.

Pull Request: #123
```

This approach has several benefits:
- **No external state files** - state travels with commits
- **Visible in `jj log`** - you can always see which commits have PRs
- **Survives rebases** - JJ's change IDs are stable across amendments
- **No API calls needed** - we know the PR number without querying GitHub

### Automatically creating PR branches from jj commits

`jj-pr mail` creates PR branches that are 1:1 with jj commits:

1. Determines the base branch (auto-detect, use saved config, or CLI flag)
2. Finds all nonempty commits between `@` and the base branch
3. For each commit, checks if it already has a PR number in the message
4. If yes: updates the PR (e.g., base branch) if needed
5. If no: creates a new PR and adds the PR number to the commit message
6. Stacks PRs properly: each PR's base is the previous commit's branch

Branch naming: `spr/<owner>/<change_id>`

When you amend commits locally, JJ automatically propagates changes to downstream commits. Running `jj-pr mail` again will push the updated commits to their existing PR branches.

### Two-way propagation

Especially in the era of AI code review bots, it's common to first alter a PR branch *on the origin* and need those changes to come down to local.

`jj-pr sync`:
1. Finds all commits with PR numbers (mailed commits)
2. Fetches the corresponding PR branch from origin
3. If the remote has new commits, squashes them onto the local commit using `jj squash`
4. Force-pushes the updated commit back to the PR branch

## Project Structure

- `src/main.rs` - CLI entry point
- `src/config.rs` - Configuration management (base branch detection and storage)
- `src/jj.rs` - Jujutsu command execution and parsing
- `src/git.rs` - Git operations wrapper using libgit2
- `src/github.rs` - GitHub API client using Octocrab
- `src/message.rs` - Commit message parsing and PR number tracking
- `src/commands/mail.rs` - Implementation of `mail` command
- `src/commands/sync.rs` - Implementation of `sync` command

## License

See LICENSE file for details.
