/*
 * GitHub Integration tests for jjpr
 * These tests create actual PRs on GitHub using SSH for git operations
 *
 * Prerequisites:
 *   1. SSH keys configured for GitHub (git clone via SSH must work)
 *   2. GitHub token for API access (creating PRs, etc.)
 *   3. A dedicated test repository
 *
 * To run these tests:
 *   export GITHUB_TOKEN=your_token           # For GitHub API (creating PRs)
 *   export JJPR_TEST_REPO=owner/repo       # Test repo with SSH access
 *   cargo test --test github_integration_test -- --ignored
 *
 * IMPORTANT: These tests will create real PRs in the specified repo!
 * Use a dedicated test repository.
 *
 * Authentication model:
 *   - Git operations (clone, push): Use SSH keys (git@github.com:owner/repo.git)
 *   - GitHub API calls (create PR): Use GITHUB_TOKEN
 */

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Test repository setup helper with GitHub integration
struct GitHubTestRepo {
    _temp_dir: TempDir,
    path: PathBuf,
    owner: String,
    repo: String,
    github_token: String,
}

impl GitHubTestRepo {
    /// Create a new test repository configured for a real GitHub repo
    fn new() -> Option<Self> {
        // Check if we have the required environment variables
        let github_token = env::var("GITHUB_TOKEN").ok()?;
        let test_repo = env::var("JJPR_TEST_REPO").ok()?;

        let parts: Vec<&str> = test_repo.split('/').collect();
        if parts.len() != 2 {
            eprintln!("JJPR_TEST_REPO must be in format owner/repo");
            return None;
        }
        let owner = parts[0].to_string();
        let repo = parts[1].to_string();

        // Create local working directory
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path = temp_dir.path().to_path_buf();

        // Clone the test repo using SSH
        let clone_url = format!("git@github.com:{}/{}.git", owner, repo);
        let clone_output = Command::new("git")
            .args(["clone", &clone_url, "."])
            .current_dir(&path)
            .output()
            .expect("Failed to clone repo");

        if !clone_output.status.success() {
            eprintln!("Failed to clone test repo via SSH.");
            eprintln!("Make sure:");
            eprintln!("  1. Your SSH keys are configured for GitHub");
            eprintln!("  2. You have access to {}/{}", owner, repo);
            eprintln!("  3. You can run: ssh -T git@github.com");
            eprintln!("\nError: {}", String::from_utf8_lossy(&clone_output.stderr));
            return None;
        }

        // Configure git user
        Command::new("git")
            .args(["config", "user.name", "JStack Test"])
            .current_dir(&path)
            .output()
            .expect("Failed to set git user name");

        Command::new("git")
            .args(["config", "user.email", "jstack-test@example.com"])
            .current_dir(&path)
            .output()
            .expect("Failed to set git user email");

        // Initialize jj repo (colocated)
        let jj_init = Command::new("jj")
            .args(["git", "init", "--colocate"])
            .current_dir(&path)
            .output()
            .expect("Failed to init jj repo");

        if !jj_init.status.success() {
            eprintln!("jj not available or failed to initialize");
            return None;
        }

        // Import Git refs into jj
        Command::new("jj")
            .args(["git", "import"])
            .current_dir(&path)
            .output()
            .expect("Failed to import git refs after init");

        // Set jj user
        Command::new("jj")
            .args(["config", "set", "--repo", "user.name", "JStack Test"])
            .current_dir(&path)
            .output()
            .expect("Failed to set jj user name");

        Command::new("jj")
            .args(["config", "set", "--repo", "user.email", "jstack-test@example.com"])
            .current_dir(&path)
            .output()
            .expect("Failed to set jj user email");

        // Check if the repository is empty (no branches)
        let branches_output = Command::new("git")
            .args(["branch", "-r"])
            .current_dir(&path)
            .output()
            .expect("Failed to list branches");

        let branches = String::from_utf8_lossy(&branches_output.stdout);
        let has_branches = !branches.trim().is_empty();

        // If empty, create initial commit on main branch
        let default_branch = if !has_branches {
            println!("Repository is empty, creating initial commit...");

            // Create initial file
            fs::write(path.join("README.md"), "# JStack Integration Test Repository\n")
                .expect("Failed to write README");

            // Add and commit
            Command::new("git")
                .args(["add", "README.md"])
                .current_dir(&path)
                .output()
                .expect("Failed to git add");

            Command::new("git")
                .args(["commit", "-m", "Initial commit"])
                .current_dir(&path)
                .output()
                .expect("Failed to create initial commit");

            // Get current branch name
            let branch_output = Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD"])
                .current_dir(&path)
                .output()
                .expect("Failed to get current branch");

            let branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();

            // Push to origin
            Command::new("git")
                .args(["push", "-u", "origin", &branch])
                .current_dir(&path)
                .output()
                .expect("Failed to push initial commit");

            // Import Git refs into jj
            Command::new("jj")
                .args(["git", "import"])
                .current_dir(&path)
                .output()
                .expect("Failed to import git refs");

            println!("Created initial commit on branch: {}", branch);
            format!("origin/{}", branch)
        } else {
            // Detect existing default branch
            Command::new("git")
                .args(["remote", "set-head", "origin", "--auto"])
                .current_dir(&path)
                .output()
                .ok();

            // Get current branch
            let branch_output = Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD"])
                .current_dir(&path)
                .output()
                .ok();

            if let Some(output) = branch_output {
                if output.status.success() {
                    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    format!("origin/{}", branch)
                } else {
                    "origin/main".to_string()
                }
            } else {
                "origin/main".to_string()
            }
        };

        println!("Using base branch: {}", default_branch);

        // Configure jstack to use this base branch
        Command::new("jj")
            .args(["config", "set", "--repo", "jstack.baseBranch", &default_branch])
            .current_dir(&path)
            .output()
            .expect("Failed to set base branch");

        Some(Self {
            _temp_dir: temp_dir,
            path,
            owner,
            repo,
            github_token,
        })
    }

    /// Create a new commit with given description
    fn create_commit(&self, description: &str, file_content: &str) -> String {
        // Create a new change
        Command::new("jj")
            .args(["new"])
            .current_dir(&self.path)
            .output()
            .expect("Failed to create new change");

        // Modify a file with timestamp to ensure uniqueness
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let filename = format!("test_{}_{}.txt", timestamp, description.replace(' ', "_"));
        fs::write(self.path.join(&filename), file_content)
            .expect("Failed to write file");

        // Set description
        Command::new("jj")
            .args(["describe", "-m", description])
            .current_dir(&self.path)
            .output()
            .expect("Failed to set description");

        // Get the change ID
        let output = Command::new("jj")
            .args(["log", "-r", "@", "-T", "change_id", "--no-graph"])
            .current_dir(&self.path)
            .output()
            .expect("Failed to get change ID");

        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    /// Run jstack mail command
    fn run_jstack_mail(&self) -> bool {
        let output = Command::new(env!("CARGO_BIN_EXE_jj_pr"))
            .args(["mail"])
            .current_dir(&self.path)
            .env("GITHUB_TOKEN", &self.github_token)
            .output()
            .expect("Failed to run jstack mail");

        println!("jstack mail output:");
        println!("{}", String::from_utf8_lossy(&output.stdout));
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }

        output.status.success()
    }

    /// Run jstack sync command
    fn run_jstack_sync(&self) -> bool {
        let output = Command::new(env!("CARGO_BIN_EXE_jj_pr"))
            .args(["sync"])
            .current_dir(&self.path)
            .env("GITHUB_TOKEN", &self.github_token)
            .output()
            .expect("Failed to run jstack sync");

        println!("jstack sync output:");
        println!("{}", String::from_utf8_lossy(&output.stdout));
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }

        output.status.success()
    }

    /// Get commit description for a change
    fn get_description(&self, revision: &str) -> String {
        let output = Command::new("jj")
            .args(["log", "-r", revision, "-T", "description", "--no-graph"])
            .current_dir(&self.path)
            .output()
            .expect("Failed to get description");

        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    /// Get the Git commit ID for a JJ revision
    fn get_commit_id(&self, revision: &str) -> String {
        let output = Command::new("jj")
            .args(["log", "-r", revision, "-T", "commit_id", "--no-graph"])
            .current_dir(&self.path)
            .output()
            .expect("Failed to get commit ID");

        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    /// Clean up test branches and PRs
    fn cleanup_test_branches(&self, branch_prefix: &str) {
        // List all branches with the test prefix
        let output = Command::new("git")
            .args(["branch", "-r"])
            .current_dir(&self.path)
            .output()
            .expect("Failed to list branches");

        let branches = String::from_utf8_lossy(&output.stdout);
        for line in branches.lines() {
            let branch = line.trim();
            if branch.starts_with(&format!("origin/{}", branch_prefix)) {
                let branch_name = branch.strip_prefix("origin/").unwrap();
                // Delete remote branch
                Command::new("git")
                    .args(["push", "origin", "--delete", branch_name])
                    .current_dir(&self.path)
                    .output()
                    .ok(); // Ignore errors
            }
        }
    }
}

// Helper to check if GitHub tests should run
fn can_run_github_tests() -> bool {
    env::var("GITHUB_TOKEN").is_ok() && env::var("JJPR_TEST_REPO").is_ok()
}

// ============================================================================
// GITHUB INTEGRATION TEST 1: Basic PR Creation and Stacking
// ============================================================================

#[test]
#[ignore] // Requires GITHUB_TOKEN and JJPR_TEST_REPO
fn test_github_stacked_pr_creation() {
    if !can_run_github_tests() {
        println!("Skipping GitHub test - GITHUB_TOKEN or JJPR_TEST_REPO not set");
        return;
    }

    let Some(repo) = GitHubTestRepo::new() else {
        panic!("Failed to set up GitHub test repo");
    };

    println!("🧪 Testing stacked PR creation on GitHub...");
    println!("   Repository: {}/{}", repo.owner, repo.repo);

    // Create two commits: A and B
    let change_a = repo.create_commit("Test commit A", "Content A");
    let change_b = repo.create_commit("Test commit B", "Content B");

    println!("   Created commits: {} <- {}", change_a, change_b);

    // Run jstack mail
    assert!(
        repo.run_jstack_mail(),
        "jstack mail should succeed"
    );

    // Verify PR numbers were added to commit messages
    let desc_a = repo.get_description(&change_a);
    let desc_b = repo.get_description(&change_b);

    assert!(
        desc_a.contains("Pull Request:"),
        "Commit A should have PR number: {}",
        desc_a
    );
    assert!(
        desc_b.contains("Pull Request:"),
        "Commit B should have PR number: {}",
        desc_b
    );

    // Extract PR numbers
    let pr_a = extract_pr_number(&desc_a).expect("Should have PR number for A");
    let pr_b = extract_pr_number(&desc_b).expect("Should have PR number for B");

    println!("   ✅ Created PR #{} for commit A", pr_a);
    println!("   ✅ Created PR #{} for commit B", pr_b);

    // Verify via GitHub API that B targets A's branch
    // (This would require using the octocrab client to query PR details)
    println!("   Note: Manual verification needed - check that PR #{} targets PR #{}'s branch", pr_b, pr_a);

    // Cleanup
    let branch_prefix = format!("spr/{}", repo.owner);
    repo.cleanup_test_branches(&branch_prefix);

    println!("✅ GitHub stacked PR test completed");
}

// ============================================================================
// GITHUB INTEGRATION TEST 2: Local Amendment and Sync
// ============================================================================

#[test]
#[ignore] // Requires GITHUB_TOKEN and JJPR_TEST_REPO
fn test_github_local_amendment_sync() {
    if !can_run_github_tests() {
        println!("Skipping GitHub test - GITHUB_TOKEN or JJPR_TEST_REPO not set");
        return;
    }

    let Some(repo) = GitHubTestRepo::new() else {
        panic!("Failed to set up GitHub test repo");
    };

    println!("🧪 Testing local amendment and sync...");

    // Create commit A
    let change_a = repo.create_commit("Test amendment A", "Original content");

    // Mail it
    assert!(repo.run_jstack_mail(), "Initial mail should succeed");

    let original_commit = repo.get_commit_id(&change_a);
    let original_desc = repo.get_description(&change_a);
    let original_pr = extract_pr_number(&original_desc).expect("Should have PR number");

    println!("   Original commit: {}", original_commit);
    println!("   Original PR: #{}", original_pr);

    // Amend the commit locally
    println!("   Amending commit locally...");
    Command::new("jj")
        .args(["edit", &change_a])
        .current_dir(&repo.path)
        .output()
        .expect("Failed to edit commit");

    fs::write(
        repo.path.join(&format!("test_amended_{}.txt", change_a)),
        "Amended content"
    ).expect("Failed to write amended file");

    Command::new("jj")
        .args(["describe", "-m", "Test amendment A (updated)\n\nPull Request: #123"])
        .current_dir(&repo.path)
        .output()
        .expect("Failed to amend description");

    let amended_commit = repo.get_commit_id(&change_a);
    println!("   Amended commit: {}", amended_commit);

    assert_ne!(
        original_commit, amended_commit,
        "Commit should have changed after amendment"
    );

    // Mail again to push the update
    assert!(repo.run_jstack_mail(), "Mail after amendment should succeed");

    // Verify PR number is preserved
    let updated_desc = repo.get_description(&change_a);
    assert!(
        updated_desc.contains(&format!("Pull Request: #{}", original_pr)),
        "PR number should be preserved after amendment"
    );

    println!("   ✅ Amendment synced successfully, PR number preserved");

    // Cleanup
    let branch_prefix = format!("spr/{}", repo.owner);
    repo.cleanup_test_branches(&branch_prefix);

    println!("✅ Local amendment test completed");
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn extract_pr_number(description: &str) -> Option<u64> {
    // Look for "Pull Request: #123" or "Pull Request: https://github.com/owner/repo/pull/123"
    if let Some(line) = description.lines().find(|l| l.contains("Pull Request:")) {
        if let Some(pos) = line.find('#') {
            let num_str: String = line[pos + 1..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            return num_str.parse().ok();
        } else if let Some(pos) = line.find("/pull/") {
            let num_str: String = line[pos + 6..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            return num_str.parse().ok();
        }
    }
    None
}

// ============================================================================
// GITHUB TEST SUMMARY
// ============================================================================

#[test]
#[ignore]
fn test_github_integration_requirements() {
    println!("🧪 Checking GitHub integration test requirements...");

    let has_token = env::var("GITHUB_TOKEN").is_ok();
    let has_repo = env::var("JJPR_TEST_REPO").is_ok();

    println!("   GITHUB_TOKEN set: {}", if has_token { "✅" } else { "❌" });
    println!("   JJPR_TEST_REPO set: {}", if has_repo { "✅" } else { "❌" });

    // Check if SSH is configured
    let ssh_check = Command::new("ssh")
        .args(["-T", "git@github.com"])
        .output();

    let ssh_configured = if let Ok(output) = ssh_check {
        let stderr = String::from_utf8_lossy(&output.stderr);
        stderr.contains("successfully authenticated")
    } else {
        false
    };

    println!("   SSH keys configured: {}", if ssh_configured { "✅" } else { "❌" });

    if !has_token || !has_repo {
        println!("\n⚠️  To run GitHub integration tests:");
        println!("   1. Set up SSH keys for GitHub:");
        println!("      ssh-keygen -t ed25519 -C \"your_email@example.com\"");
        println!("      # Add ~/.ssh/id_ed25519.pub to GitHub Settings → SSH Keys");
        println!("      ssh -T git@github.com  # Test connection");
        println!("\n   2. Get a GitHub personal access token:");
        println!("      GitHub Settings → Developer settings → Personal access tokens");
        println!("      Scopes needed: repo (full control)");
        println!("\n   3. Set environment variables:");
        println!("      export GITHUB_TOKEN=ghp_your_token_here");
        println!("      export JJPR_TEST_REPO=owner/test-repo");
        println!("\n   4. Run tests:");
        println!("      cargo test --test github_integration_test -- --ignored");
        println!("\n   ⚠️  Use a dedicated test repository!");
    } else if !ssh_configured {
        println!("\n⚠️  SSH keys don't appear to be configured for GitHub");
        println!("   Test with: ssh -T git@github.com");
    } else {
        println!("\n✅ Ready to run GitHub integration tests");
    }

    assert!(
        has_token && has_repo,
        "Set GITHUB_TOKEN and JJPR_TEST_REPO to run GitHub tests"
    );
}
