use anyhow::{Context, Result};
use git2::{BranchType, Cred, FetchOptions, Oid, PushOptions, RemoteCallbacks, Repository};
use std::path::{Path, PathBuf};
// PathBuf used in remote_callbacks() closure for key file construction

pub struct Git {
    repo: Repository,
}

impl Git {
    pub fn new(repo_path: impl AsRef<Path>) -> Result<Self> {
        let repo = Repository::open(repo_path).context("Failed to open git repository")?;
        Ok(Self { repo })
    }

    /// Build RemoteCallbacks with SSH agent + key-file fallback.
    fn remote_callbacks() -> RemoteCallbacks<'static> {
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username, allowed| {
            let username = username.unwrap_or("git");

            // 1. Try SSH agent (works when ssh-agent / macOS Keychain is running)
            if allowed.contains(git2::CredentialType::SSH_KEY) {
                if let Ok(cred) = Cred::ssh_key_from_agent(username) {
                    return Ok(cred);
                }
            }

            // 2. Try common key files under ~/.ssh/
            if allowed.contains(git2::CredentialType::SSH_KEY) {
                if let Ok(home) = std::env::var("HOME") {
                    for key in &["id_ed25519", "id_ecdsa", "id_rsa"] {
                        let priv_path = PathBuf::from(&home).join(".ssh").join(key);
                        let pub_path = priv_path.with_extension("pub");
                        if priv_path.exists() {
                            if let Ok(cred) = Cred::ssh_key(
                                username,
                                pub_path.exists().then(|| pub_path.as_path()),
                                &priv_path,
                                None,
                            ) {
                                return Ok(cred);
                            }
                        }
                    }
                }
            }

            // 3. Default credentials (e.g. HTTPS via credential helper)
            Cred::default()
        });
        callbacks
    }

    /// Create or update a branch to point at a specific commit
    pub fn create_branch(&self, branch_name: &str, commit_id: &str) -> Result<()> {
        let oid = Oid::from_str(commit_id).context("Invalid commit ID")?;
        let commit = self.repo.find_commit(oid).context("Commit not found")?;

        // Delete existing branch if it exists
        if let Ok(mut branch) = self.repo.find_branch(branch_name, BranchType::Local) {
            branch.delete()?;
        }

        self.repo
            .branch(branch_name, &commit, true)
            .context("Failed to create branch")?;
        Ok(())
    }

    /// Push a branch to remote
    pub fn push_branch(&self, branch_name: &str, remote_name: &str) -> Result<()> {
        let mut remote = self.repo.find_remote(remote_name)?;
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

        let mut opts = PushOptions::new();
        opts.remote_callbacks(Self::remote_callbacks());

        remote
            .push(&[&refspec], Some(&mut opts))
            .context("Failed to push branch")?;
        Ok(())
    }

    /// Fetch a branch from remote
    pub fn fetch_branch(&self, branch_name: &str, remote_name: &str) -> Result<()> {
        let mut remote = self.repo.find_remote(remote_name)?;
        let refspec = format!(
            "+refs/heads/{}:refs/remotes/{}/{}",
            branch_name, remote_name, branch_name
        );

        let mut opts = FetchOptions::new();
        opts.remote_callbacks(Self::remote_callbacks());

        remote
            .fetch(&[&refspec], Some(&mut opts), None)
            .context("Failed to fetch branch")?;
        Ok(())
    }

    /// Get the commit ID for a remote branch
    pub fn get_remote_branch_commit(&self, branch_name: &str, remote_name: &str) -> Result<String> {
        let remote_branch_name = format!("{}/{}", remote_name, branch_name);
        let branch = self.repo
            .find_branch(&remote_branch_name, BranchType::Remote)
            .context("Remote branch not found")?;

        let commit = branch.get().peel_to_commit()?;
        Ok(commit.id().to_string())
    }

    /// Get the log of commits between two commits
    pub fn get_commits_between(&self, base: &str, head: &str) -> Result<Vec<String>> {
        let base_oid = Oid::from_str(base)?;
        let head_oid = Oid::from_str(head)?;

        let mut revwalk = self.repo.revwalk()?;
        revwalk.push(head_oid)?;
        revwalk.hide(base_oid)?;

        let commits: Result<Vec<String>> = revwalk
            .map(|oid| Ok(oid?.to_string()))
            .collect();

        commits
    }

    /// Get current HEAD commit
    pub fn get_head_commit(&self) -> Result<String> {
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;
        Ok(commit.id().to_string())
    }

    /// Check if a branch exists on remote
    pub fn remote_branch_exists(&self, branch_name: &str, remote_name: &str) -> bool {
        let remote_branch_name = format!("{}/{}", remote_name, branch_name);
        self.repo
            .find_branch(&remote_branch_name, BranchType::Remote)
            .is_ok()
    }

    /// Get the repository's remote URL
    pub fn get_remote_url(&self, remote_name: &str) -> Result<String> {
        let remote = self.repo.find_remote(remote_name)?;
        let url = remote.url().context("Remote has no URL")?;
        Ok(url.to_string())
    }

    /// Parse GitHub owner/repo from remote URL
    pub fn parse_github_repo(&self, remote_name: &str) -> Result<(String, String)> {
        let url = self.get_remote_url(remote_name)?;

        // Parse URLs like:
        // - https://github.com/owner/repo.git
        // - git@github.com:owner/repo.git
        let parts: Vec<&str> = if url.contains("github.com:") {
            url.split("github.com:").collect()
        } else if url.contains("github.com/") {
            url.split("github.com/").collect()
        } else {
            anyhow::bail!("Not a GitHub URL: {}", url);
        };

        if parts.len() != 2 {
            anyhow::bail!("Invalid GitHub URL: {}", url);
        }

        let repo_part = parts[1].trim_end_matches(".git");
        let repo_parts: Vec<&str> = repo_part.split('/').collect();

        if repo_parts.len() != 2 {
            anyhow::bail!("Invalid GitHub repo path: {}", repo_part);
        }

        Ok((repo_parts[0].to_string(), repo_parts[1].to_string()))
    }
}
