# jjpr Documentation

Complete documentation for jjpr - a utility for stacked pull requests with Jujutsu and GitHub.

## Getting Started

New to jjpr? Start here:

1. **[Overview](overview.md)** - Understand what jjpr is and how it works
2. **[Installation](installation.md)** - Set up jjpr on your system
3. **[GitHub Setup](github-setup.md)** - Configure authentication (SSH + Token)
4. **[Workflows](workflows.md)** - Learn common usage patterns

## Core Documentation

### Commands

- **[Command Reference](commands.md)** - Complete command documentation
  - `jj-pr mail` - Create/update PRs
  - `jj-pr sync` - Synchronize with remote
  - Flags, options, and examples

### Deep Dives

- **[Sync Deep Dive](sync.md)** - Understand sync behavior
  - Bidirectional synchronization
  - Conflict resolution
  - Divergence detection
  - Status mode

- **[Merge Scenarios](merge-scenarios.md)** - Understand merge workflows
  - Remote changes merged into PR branch
  - PRs merged into target branch
  - Stack merging strategies
  - Visual gitgraph diagrams

### Workflows

- **[Common Workflows](workflows.md)** - Real-world usage patterns
  - Creating stacks
  - Updating after review
  - Handling Copilot suggestions
  - Reordering commits
  - Collaborating on PRs

## Quick Reference

### Installation

```bash
cargo install --path .
```

### Authentication

```bash
# SSH keys (for Git)
ssh-keygen -t ed25519 -C "you@example.com"
# Add to GitHub → Settings → SSH keys

# Token (for API)
export GITHUB_TOKEN="github_pat_YOUR_TOKEN"
# Create at GitHub → Settings → Developer settings → Personal access tokens
```

### Basic Usage

```bash
# Create PRs for your commits
jj-pr mail

# Check sync status
jj-pr sync --status

# Sync with remote
jj-pr sync
```

## Document Structure

```
docs/
├── README.md              # This file - documentation index
├── overview.md            # What is jjpr and how it works
├── installation.md        # Setup and installation guide
├── github-setup.md        # GitHub authentication (SSH + Token)
├── commands.md            # Complete command reference
├── sync.md                # Deep dive into sync functionality
├── merge-scenarios.md     # Merge workflows with diagrams
└── workflows.md           # Common usage patterns
```

## Topics by Use Case

### "I'm brand new to jjpr"

1. Read [Overview](overview.md) - 10 minutes
2. Follow [Installation](installation.md) - 15 minutes
3. Complete [GitHub Setup](github-setup.md) - 10 minutes
4. Try [Basic Workflow](workflows.md#basic-workflow) - 5 minutes

Total: ~40 minutes to get started

### "I want to create my first stack"

1. [Creating a Stack](workflows.md#creating-a-stack)
2. [`jj-pr mail` command](commands.md#jstack-mail)
3. [Viewing the Stack](workflows.md#viewing-the-stack)

### "I need to sync changes from GitHub"

1. [Sync Deep Dive](sync.md) - Understanding sync
2. [`jj-pr sync` command](commands.md#jstack-sync)
3. [Handling Copilot Suggestions](workflows.md#handling-copilot-suggestions)

### "I have merge conflicts"

1. [Conflicts During Merge](merge-scenarios.md#conflicts-during-merge)
2. [Conflict Resolution](sync.md#conflict-resolution)
3. [Recovering from Errors](workflows.md#recovering-from-errors)

### "I'm merging PRs"

1. [PR Merged into Target Branch](merge-scenarios.md#pr-merged-into-target-branch)
2. [Stack Merging Strategy](merge-scenarios.md#stack-merging-strategy)
3. [Merging PRs](workflows.md#merging-prs)

### "I'm setting up authentication"

1. [SSH Keys](github-setup.md#part-1-ssh-keys)
2. [Personal Access Token](github-setup.md#part-2-personal-access-token)
3. [Storing Tokens](github-setup.md#storing-tokens)
4. [Troubleshooting](github-setup.md#troubleshooting)

### "I need to fix an error"

1. [Recovering from Errors](workflows.md#recovering-from-errors)
2. [Troubleshooting - Commands](commands.md#common-errors)
3. [Troubleshooting - GitHub Setup](github-setup.md#troubleshooting)
4. [Troubleshooting - Sync](sync.md#troubleshooting)

## Key Concepts

### Stacked PRs

Pull requests that build on each other:
```
main → PR A → PR B → PR C
```

Each PR can be reviewed and merged independently, but in order.

### Change IDs

JJ's stable identifiers for changes. jjpr uses these to:
- Track PRs across rebases
- Create consistent branch names
- Maintain stack structure

### Bidirectional Sync

jjpr syncs in both directions:
- **Remote → Local**: Pull Copilot suggestions, collaborator changes
- **Local → Remote**: Push your amendments and updates

### State Tracking

PR numbers stored in commit messages:
```
Add authentication

Implements OAuth2 flow.

Pull Request: #42
```

## Diagrams and Examples

All documents include:
- ✅ Real command examples with output
- ✅ Mermaid gitgraph diagrams (in merge-scenarios.md)
- ✅ Step-by-step workflows
- ✅ Troubleshooting sections

## Contributing to Docs

Found an issue or want to improve the docs?

1. Documentation lives in `docs/`
2. Use clear, concise language
3. Include examples for every concept
4. Add mermaid diagrams for git operations
5. Test all commands before documenting

## External Resources

- [Jujutsu Documentation](https://jj-vcs.github.io/jj/) - Learn JJ
- [GitHub Pull Requests](https://docs.github.com/en/pull-requests) - PR basics
- [GitHub API](https://docs.github.com/en/rest) - API reference

## Version

These docs are for **jjpr v0.1.0**.

For the latest version, see the [main repository](https://github.com/yourusername/jjpr).

## Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/jjpr/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/jjpr/discussions)
- **Email**: your-email@example.com

---

## Quick Links

- [Overview](overview.md) | [Installation](installation.md) | [Commands](commands.md)
- [Sync](sync.md) | [Merge Scenarios](merge-scenarios.md) | [Workflows](workflows.md)
- [GitHub Setup](github-setup.md)
