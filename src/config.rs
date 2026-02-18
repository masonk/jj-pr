use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Configuration for jstack
#[derive(Debug, Clone)]
pub struct Config {
    pub base_branch: String,
}

impl Config {
    /// Get or create configuration, with optional override from CLI
    pub fn resolve(repo_path: impl AsRef<Path>, base_override: Option<String>) -> Result<Self> {
        // If user provided explicit base branch, use it and save it
        if let Some(base) = base_override {
            // Save to config for future use
            Self::save_base_branch(&base)?;
            return Ok(Self { base_branch: base });
        }

        // Try to load from saved config
        if let Some(saved_base) = Self::load_base_branch()? {
            let normalized = Self::normalize_base_branch(&saved_base);
            if normalized != saved_base {
                Self::save_base_branch(&normalized)?;
            }
            return Ok(Self {
                base_branch: normalized,
            });
        }

        // Auto-detect and prompt if needed
        let detected = Self::detect_base_branch(repo_path)?;

        // Save the detected branch for future use
        Self::save_base_branch(&detected)?;

        Ok(Self {
            base_branch: detected,
        })
    }

    /// Convert old git-style "origin/master" to jj-style "master@origin"
    fn normalize_base_branch(branch: &str) -> String {
        if let Some(name) = branch.strip_prefix("origin/") {
            format!("{}@origin", name)
        } else {
            branch.to_string()
        }
    }

    /// Detect the base branch by checking what exists on origin
    fn detect_base_branch(repo_path: impl AsRef<Path>) -> Result<String> {
        let candidates = ["master@origin", "main@origin"];

        // Try to find which branch exists
        for candidate in candidates {
            if Self::branch_exists(repo_path.as_ref(), candidate)? {
                println!("📍 Auto-detected base branch: {}", candidate);
                return Ok(candidate.to_string());
            }
        }

        // If neither exists, prompt the user
        println!("⚠️  Could not auto-detect base branch (tried origin/master and origin/main)");
        println!("Please specify the base branch:");

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .context("Failed to read input")?;

        let branch = input.trim().to_string();
        if branch.is_empty() {
            anyhow::bail!("Base branch cannot be empty");
        }

        Ok(branch)
    }

    /// Check if a branch exists using jj
    fn branch_exists(repo_path: &Path, branch: &str) -> Result<bool> {
        let output = Command::new("jj")
            .args(["log", "-r", branch, "-T", "commit_id", "--no-graph"])
            .current_dir(repo_path)
            .output()
            .context("Failed to check if branch exists")?;

        Ok(output.status.success())
    }

    /// Load base branch from jj config
    fn load_base_branch() -> Result<Option<String>> {
        let output = Command::new("jj")
            .args(["config", "get", "jjpr.baseBranch"])
            .output()
            .context("Failed to read jj config")?;

        if output.status.success() {
            let value = String::from_utf8(output.stdout)
                .context("Invalid UTF-8 in config")?
                .trim()
                .to_string();

            if !value.is_empty() {
                return Ok(Some(value));
            }
        }

        Ok(None)
    }

    /// Save base branch to jj config
    fn save_base_branch(branch: &str) -> Result<()> {
        let status = Command::new("jj")
            .args(["config", "set", "--repo", "jjpr.baseBranch", branch])
            .status()
            .context("Failed to save base branch to config")?;

        if !status.success() {
            anyhow::bail!("Failed to save base branch to jj config");
        }

        Ok(())
    }

    /// Get the full branch ref (e.g., "origin/master")
    pub fn base_ref(&self) -> &str {
        &self.base_branch
    }

    /// Get just the branch name without remote (e.g., "master" from "master@origin")
    pub fn base_branch_name(&self) -> &str {
        self.base_branch
            .strip_suffix("@origin")
            .or_else(|| self.base_branch.strip_prefix("origin/"))
            .unwrap_or(&self.base_branch)
    }
}
