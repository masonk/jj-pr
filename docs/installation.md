# Installation Guide

## Prerequisites

### 1. Jujutsu (jj)

jjpr requires Jujutsu version 0.8.0 or later.

**macOS** (via Homebrew):
```bash
brew install jj
```

**Linux**:
```bash
cargo install --git https://github.com/martinvonz/jj.git jj-cli
```

**Windows**:
```bash
cargo install --git https://github.com/martinvonz/jj.git jj-cli
```

Verify installation:
```bash
jj --version
```

### 2. Git

jjpr uses Git for push/pull operations. Most systems have Git pre-installed.

Verify installation:
```bash
git --version
```

### 3. Rust (for building from source)

If building jjpr from source:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Installing jjpr

### Option 1: From Source (Current)

```bash
# Clone the repository
git clone https://github.com/yourusername/jjpr.git
cd jjpr

# Build in release mode
cargo build --release

# Install to ~/.cargo/bin
cargo install --path .
```

Verify installation:
```bash
jj-pr --help
```

### Option 2: Pre-built Binaries (Coming Soon)

Pre-built binaries for major platforms will be available in future releases.

## Initial Setup

### 1. Configure Jujutsu

Set your identity (if not already done):

```bash
jj config set --user user.name "Your Name"
jj config set --user user.email "you@example.com"
```

### 2. Set Up GitHub Authentication

jjpr requires two types of authentication:

#### A. SSH Keys (for Git operations)

1. Generate SSH key (if you don't have one):
   ```bash
   ssh-keygen -t ed25519 -C "you@example.com"
   ```

2. Add to ssh-agent:
   ```bash
   eval "$(ssh-agent -s)"
   ssh-add ~/.ssh/id_ed25519
   ```

3. Add public key to GitHub:
   - Go to GitHub Settings → SSH and GPG keys
   - Click "New SSH key"
   - Paste contents of `~/.ssh/id_ed25519.pub`

4. Test SSH connection:
   ```bash
   ssh -T git@github.com
   ```

#### B. Personal Access Token (for GitHub API)

jjpr needs a GitHub token to create and update PRs.

##### For Fine-Grained Tokens (Recommended):

1. Go to: **GitHub Settings → Developer settings → Personal access tokens → Fine-grained tokens**

2. Click **Generate new token**

3. Configure:
   - **Token name**: `jj-pr`
   - **Expiration**: Choose appropriate duration (90 days, 1 year, etc.)
   - **Repository access**: Select repositories or "All repositories"

4. **Permissions** (Repository permissions):
   - **Pull requests**: Read and write ✅
   - **Contents**: Read only ✅
   - **Metadata**: Read only ✅ (auto-included)

5. Generate token and copy it

##### For Classic Tokens (Legacy):

1. Go to: **GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)**

2. Click **Generate new token**

3. Select scopes:
   - **repo** (Full control of private repositories) ✅

4. Generate token and copy it

#### C. Configure Token

Add the token to your environment:

**Bash/Zsh** (`~/.bashrc` or `~/.zshrc`):
```bash
export GITHUB_TOKEN="github_pat_YOUR_TOKEN_HERE"
```

**Fish** (`~/.config/fish/config.fish`):
```fish
set -x GITHUB_TOKEN "github_pat_YOUR_TOKEN_HERE"
```

Reload your shell:
```bash
source ~/.bashrc  # or ~/.zshrc or restart terminal
```

Verify:
```bash
echo $GITHUB_TOKEN
```

### 3. Initialize Repository

In your project directory:

```bash
# If starting fresh
jj git init --colocate

# If you have existing Git repo
cd your-project
jj git init --colocate

# Verify
jj status
```

The `--colocate` flag makes JJ work alongside Git in the same directory.

### 4. Configure Base Branch (Optional)

jjpr auto-detects your base branch, but you can set it explicitly:

```bash
jj config set --repo jjpr.baseBranch origin/main
```

Or specify per-command:
```bash
jj-pr mail --base origin/develop
```

## Verification

Test your setup:

```bash
# Create a test commit
echo "# Test" > test.md
jj describe -m "Test commit"

# Try creating a PR (won't actually create if you haven't pushed anything)
jj-pr mail --help
```

If everything is configured correctly, you should see the help output.

## Troubleshooting

### "jj binary not found in PATH"

jjpr can't find Jujutsu. Make sure `jj` is installed and in your PATH:

```bash
which jj
```

If not found, reinstall Jujutsu or add it to PATH.

### "GITHUB_TOKEN or GH_TOKEN environment variable required"

Your GitHub token isn't set. Make sure you:
1. Created a token (see step 2B above)
2. Added it to your shell config file
3. Reloaded your shell or restarted your terminal

### "Permission denied (publickey)"

Your SSH keys aren't configured. Follow step 2A above.

Test your SSH connection:
```bash
ssh -T git@github.com
```

You should see:
```
Hi username! You've successfully authenticated, but GitHub does not provide shell access.
```

### "Failed to parse GitHub owner/repo from remote"

Your repository doesn't have a GitHub remote configured:

```bash
git remote -v
```

Add the origin:
```bash
git remote add origin git@github.com:username/repo.git
```

### "Could not auto-detect base branch"

jjpr couldn't find `origin/master` or `origin/main`. Either:

1. Push to create the branch:
   ```bash
   git push -u origin main
   ```

2. Specify explicitly:
   ```bash
   jj-pr mail --base origin/develop
   ```

## Upgrading

### From Source

```bash
cd jjpr
git pull
cargo install --path .
```

### Via Cargo (if installed that way)

```bash
cargo install --path . --force
```

## Uninstalling

```bash
cargo uninstall jj-pr
```

## Next Steps

- [Overview](overview.md) - Learn how jjpr works
- [Commands](commands.md) - Command reference
- [Workflows](workflows.md) - Start using jjpr
