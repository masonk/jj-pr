# GitHub Setup Guide

## Overview

jjpr requires two types of GitHub authentication:
1. **SSH Keys** - For Git operations (push, pull, fetch)
2. **Personal Access Token** - For GitHub API operations (creating/updating PRs)

## Why Two Authentication Methods?

| Operation | Method | Why |
|-----------|--------|-----|
| `git push` / `git fetch` | SSH Keys | Secure, no password needed, standard Git practice |
| Create PR | API Token | GitHub API requires token authentication |
| Update PR base | API Token | GitHub API operation |
| Get PR info | API Token | GitHub API operation |

You **cannot** use only one method - both are required.

---

## Part 1: SSH Keys

### Check Existing Keys

```bash
ls -la ~/.ssh/id_*.pub
```

If you see files like `id_ed25519.pub` or `id_rsa.pub`, you already have keys.

### Generate New Keys (if needed)

```bash
ssh-keygen -t ed25519 -C "your-email@example.com"
```

When prompted:
- **File location**: Press Enter (uses default `~/.ssh/id_ed25519`)
- **Passphrase**: Optional but recommended

### Add Key to SSH Agent

**macOS/Linux**:
```bash
eval "$(ssh-agent -s)"
ssh-add ~/.ssh/id_ed25519
```

**macOS** - Persist across reboots:
```bash
# Add to ~/.ssh/config
cat >> ~/.ssh/config << 'EOF'
Host github.com
  AddKeysToAgent yes
  UseKeychain yes
  IdentityFile ~/.ssh/id_ed25519
EOF
```

**Windows** (PowerShell):
```powershell
Start-Service ssh-agent
ssh-add ~\.ssh\id_ed25519
```

### Add Public Key to GitHub

1. Copy your public key:
   ```bash
   cat ~/.ssh/id_ed25519.pub
   ```

2. Go to [GitHub → Settings → SSH and GPG keys](https://github.com/settings/keys)

3. Click **New SSH key**

4. Fill in:
   - **Title**: "jjpr CLI" (or your computer name)
   - **Key type**: Authentication Key
   - **Key**: Paste your public key

5. Click **Add SSH key**

### Test SSH Connection

```bash
ssh -T git@github.com
```

Expected output:
```
Hi username! You've successfully authenticated, but GitHub does not provide shell access.
```

If you get "Permission denied", check:
- Key was added to ssh-agent: `ssh-add -l`
- Key was added to GitHub
- Using correct key: `ssh -vT git@github.com` (verbose mode)

---

## Part 2: Personal Access Token

### Option A: Fine-Grained Tokens (Recommended)

Fine-grained tokens are more secure with granular permissions.

#### 1. Create Token

1. Go to [GitHub → Settings → Developer settings → Personal access tokens → Fine-grained tokens](https://github.com/settings/tokens?type=beta)

2. Click **Generate new token**

#### 2. Configure Token

**Token name**: `jj-pr-cli`

**Expiration**:
- Development: 90 days
- Personal use: 1 year
- Team use: Custom (discuss with team)

**Description**: "jjpr CLI for stacked PRs"

**Repository access**:
- **Only select repositories**: Choose specific repos you'll use jjpr with
- **All repositories**: If you want to use jjpr across all your repos

#### 3. Set Permissions

Under **Repository permissions**:

| Permission | Access | Why |
|------------|--------|-----|
| Pull requests | **Read and write** | Create and update PRs |
| Contents | **Read-only** | Access repository content |
| Metadata | **Read-only** | Automatically included |

#### 4. Generate and Copy

1. Click **Generate token**
2. **Copy the token immediately** - you won't see it again
3. Store safely (see [Storing Tokens](#storing-tokens) below)

### Option B: Classic Tokens (Legacy)

Classic tokens have broader permissions but are simpler to set up.

#### 1. Create Token

1. Go to [GitHub → Settings → Developer settings → Personal access tokens → Tokens (classic)](https://github.com/settings/tokens)

2. Click **Generate new token** → **Generate new token (classic)**

#### 2. Configure Token

**Note**: `jj-pr-cli`

**Expiration**: 90 days (or your preference)

**Select scopes**:
- ✅ **repo** (Full control of private repositories)

This grants:
- repo:status
- repo_deployment
- public_repo
- repo:invite
- security_events

#### 3. Generate and Copy

1. Click **Generate token**
2. Copy the token immediately
3. Store safely

---

## Storing Tokens

### Environment Variable (Recommended)

**Bash/Zsh** (`~/.bashrc` or `~/.zshrc`):
```bash
export GITHUB_TOKEN="github_pat_YOUR_TOKEN_HERE"
```

**Fish** (`~/.config/fish/config.fish`):
```fish
set -x GITHUB_TOKEN "github_pat_YOUR_TOKEN_HERE"
```

**Windows** (PowerShell profile):
```powershell
$env:GITHUB_TOKEN = "github_pat_YOUR_TOKEN_HERE"
```

Reload shell:
```bash
source ~/.bashrc  # or restart terminal
```

### Alternative: .env File (Not Recommended for Security)

```bash
# In your project directory
echo 'GITHUB_TOKEN=github_pat_YOUR_TOKEN_HERE' > .env

# Add to .gitignore (CRITICAL!)
echo '.env' >> .gitignore
```

Load before using jj-pr:
```bash
source .env
jj-pr mail
```

⚠️ **Warning**: Easy to accidentally commit. Use environment variable instead.

### Verification

```bash
echo $GITHUB_TOKEN
# Should print your token
```

If empty:
- Check you added to correct shell config file
- Reloaded shell (or restarted terminal)
- Used correct syntax for your shell

---

## Token Security

### Best Practices

1. ✅ **Use fine-grained tokens** when possible
2. ✅ **Set expiration dates** (max 1 year)
3. ✅ **Limit repository access** to only what you need
4. ✅ **Regenerate tokens** periodically
5. ✅ **Never commit tokens** to Git
6. ✅ **Use different tokens** for different tools/machines

### If Token is Compromised

1. **Immediately revoke**:
   - Go to GitHub Settings → Developer settings → Personal access tokens
   - Find the token → Delete

2. **Check recent activity**:
   - GitHub → Settings → Security log
   - Look for suspicious activity

3. **Generate new token** following steps above

4. **Update environment variable** with new token

### Token Permissions Explained

#### Pull Requests: Read and Write

**Allows**:
- Create new pull requests (`POST /repos/{owner}/{repo}/pulls`)
- Update PR title, body (`PATCH /repos/{owner}/{repo}/pulls/{number}`)
- Change PR base branch (`PATCH /repos/{owner}/{repo}/pulls/{number}`)
- List PRs (`GET /repos/{owner}/{repo}/pulls`)

**Does NOT allow**:
- Merge PRs (requires separate permission)
- Close PRs (requires separate permission)
- Delete branches

#### Contents: Read

**Allows**:
- List repository contents
- Read file contents
- Access commit information

**Does NOT allow**:
- Push commits (this is done via SSH, not API)
- Create/delete branches via API
- Modify repository files via API

#### Metadata: Read

**Automatically included**, allows:
- Access repository metadata
- List branches
- Get repository info

---

## Troubleshooting

### "GITHUB_TOKEN or GH_TOKEN environment variable required"

**Problem**: Token not found.

**Solutions**:
```bash
# 1. Check if set
echo $GITHUB_TOKEN

# 2. If empty, check shell config
cat ~/.bashrc | grep GITHUB_TOKEN

# 3. Reload shell
source ~/.bashrc

# 4. Or restart terminal
```

### "Bad credentials" or "401 Unauthorized"

**Problem**: Token is invalid or expired.

**Solutions**:
1. Check token on GitHub (Settings → Developer settings)
2. Regenerate if expired
3. Update environment variable
4. Verify with:
   ```bash
   curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/user
   ```

### "Resource not accessible by personal access token"

**Problem**: Token lacks required permissions.

**Solutions**:
1. Go to token settings on GitHub
2. Add missing permissions:
   - Pull requests: Read and write
   - Contents: Read
3. Or create new token with correct permissions

### "Permission denied (publickey)"

**Problem**: SSH key not configured.

**Solutions**:
1. Check key exists: `ls ~/.ssh/id_*.pub`
2. Add to ssh-agent: `ssh-add ~/.ssh/id_ed25519`
3. Add to GitHub (Settings → SSH keys)
4. Test: `ssh -T git@github.com`

### "Failed to parse GitHub owner/repo"

**Problem**: Git remote not configured correctly.

**Check**:
```bash
git remote -v
```

**Should show**:
```
origin  git@github.com:username/repo.git (fetch)
origin  git@github.com:username/repo.git (push)
```

**Fix**:
```bash
# Add if missing
git remote add origin git@github.com:username/repo.git

# Or update if wrong
git remote set-url origin git@github.com:username/repo.git
```

---

## Multiple Accounts

### Different Tokens Per Repository

**Use repository-specific tokens**:

```bash
# In repo A
export GITHUB_TOKEN="token_for_account_a"
jj-pr mail

# In repo B
export GITHUB_TOKEN="token_for_account_b"
jj-pr mail
```

### Different SSH Keys Per Account

**~/.ssh/config**:
```
Host github-work
  HostName github.com
  User git
  IdentityFile ~/.ssh/id_ed25519_work

Host github-personal
  HostName github.com
  User git
  IdentityFile ~/.ssh/id_ed25519_personal
```

**Use in remotes**:
```bash
# Work repo
git remote add origin git@github-work:company/repo.git

# Personal repo
git remote add origin git@github-personal:username/repo.git
```

---

## CI/CD Integration

### GitHub Actions

**Don't use personal tokens in Actions** - use `GITHUB_TOKEN` secret instead:

```yaml
name: jjpr
on: [push]

jobs:
  mail:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Install jjpr
        run: cargo install --git https://github.com/username/jjpr

      - name: Mail PRs
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: jj-pr mail
```

The `GITHUB_TOKEN` secret is automatically provided by GitHub Actions.

---

## Next Steps

- [Installation](installation.md) - Complete setup guide
- [Overview](overview.md) - Learn how jjpr works
- [Commands](commands.md) - Command reference
- [Workflows](workflows.md) - Start using jjpr
