use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Commit {
    pub change_id: String,
    pub commit_id: String,
    pub description: String,
    pub author: String,
    pub empty: bool,
    pub pr_number: Option<u64>,
}

pub struct Jj {
    repo_path: PathBuf,
    jj_bin: PathBuf,
}

impl Jj {
    pub fn new(repo_path: impl AsRef<Path>) -> Result<Self> {
        let jj_bin = which::which("jj").context("jj binary not found in PATH")?;
        Ok(Self {
            repo_path: repo_path.as_ref().to_path_buf(),
            jj_bin,
        })
    }

    /// Execute a jj command and return stdout
    fn execute(&self, args: &[&str]) -> Result<String> {
        let output = Command::new(&self.jj_bin)
            .args(args)
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to execute jj command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("jj command failed: {}", stderr);
        }

        String::from_utf8(output.stdout).context("Invalid UTF-8 in jj output")
    }

    /// Get commits between two revisions (e.g., "origin/master..@")
    pub fn get_commits(&self, from: &str, to: &str) -> Result<Vec<Commit>> {
        let revset = format!("{}..{}", from, to);
        self.get_commits_from_revset(&revset)
    }

    /// Get commits from a revset expression
    pub fn get_commits_from_revset(&self, revset: &str) -> Result<Vec<Commit>> {
        // Field separator: \0 (null). Record separator: \n\0\n.
        // \n\0\n cannot appear in real commit descriptions, and avoids the ambiguity that
        // arises when an empty description produces \0\0\0 (field-sep + empty-desc + rec-sep),
        // which would cause \0\0 splitting to bleed a leading \0 into the next record.
        // Record format: change_id\0commit_id\0author\0empty\0description\n\0\n
        let template = r#"change_id ++ "\0" ++ commit_id ++ "\0" ++ author.email() ++ "\0" ++ if(empty, "true", "false") ++ "\0" ++ description ++ "\n\0\n""#;

        let output = self.execute(&[
            "log",
            "-r",
            revset,
            "--no-graph",
            "-T",
            template,
        ])?;

        let mut commits = Vec::new();
        for record in output.split("\n\x00\n") {
            if record.trim_matches('\n').is_empty() {
                continue;
            }
            // Split into at most 5 fields; description is last and may contain \0 (unlikely but safe)
            let parts: Vec<&str> = record.splitn(5, '\0').collect();
            if parts.len() < 5 {
                continue;
            }
            let mut commit = Commit {
                change_id: parts[0].to_string(),
                commit_id: parts[1].to_string(),
                author: parts[2].to_string(),
                empty: parts[3] == "true",
                description: parts[4].to_string(),
                pr_number: None,
            };
            commit.pr_number = crate::message::parse_pr_number(&commit.description);
            commits.push(commit);
        }

        Ok(commits)
    }

    /// Get the commit ID for a specific revision
    pub fn resolve_revision(&self, revision: &str) -> Result<String> {
        let output = self.execute(&["log", "-r", revision, "-T", "commit_id", "--no-graph"])?;
        Ok(output.trim().to_string())
    }

    /// Get the change ID for a specific revision
    pub fn get_change_id(&self, revision: &str) -> Result<String> {
        let output = self.execute(&["log", "-r", revision, "-T", "change_id", "--no-graph"])?;
        Ok(output.trim().to_string())
    }

    /// Update commit message for a change
    pub fn describe(&self, change_id: &str, message: &str) -> Result<()> {
        self.execute(&["describe", "--ignore-immutable", "-r", change_id, "-m", message])?;
        Ok(())
    }

    /// Add PR number to commit message
    pub fn add_pr_to_commit(&self, change_id: &str, current_message: &str, pr_number: u64) -> Result<()> {
        let new_message = crate::message::add_pr_number(current_message, pr_number);
        self.describe(change_id, &new_message)?;
        Ok(())
    }

    /// Squash changes onto a commit
    pub fn squash(&self, from_revision: &str, into_revision: &str) -> Result<()> {
        self.execute(&["squash", "--from", from_revision, "--into", into_revision])?;
        Ok(())
    }

    /// Get current repository root
    pub fn get_root(&self) -> Result<PathBuf> {
        let output = self.execute(&["root"])?;
        Ok(PathBuf::from(output.trim()))
    }
}
