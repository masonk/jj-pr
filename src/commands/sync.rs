use anyhow::{Context, Result};
use std::path::Path;

use crate::config::Config;
use crate::git::Git;
use crate::github::GitHub;
use crate::jj::Jj;

/// Sync PR branches: pull changes from remote and reset branches to local commits
pub async fn sync(
    repo_path: impl AsRef<Path>,
    github_token: String,
    base_override: Option<String>,
    revisions: Option<String>,
    status_only: bool,
) -> Result<()> {
    let jj = Jj::new(&repo_path)?;
    let git = Git::new(&repo_path)?;

    // Resolve base branch configuration
    let config = Config::resolve(&repo_path, base_override)?;

    // Parse GitHub owner/repo from remote
    let (owner, repo) = git.parse_github_repo("origin")?;
    let _github = GitHub::new(github_token, owner.clone(), repo.clone())?;

    if status_only {
        println!("📊 Checking sync status...");
    } else {
        println!("🔄 Syncing commits...");
    }

    // Get all commits - either from revset or between base branch and @
    let commits = if let Some(ref revset) = revisions {
        println!("   Using revset: {}", revset);
        jj.get_commits_from_revset(revset)
            .context("Failed to get commits from revset")?
    } else {
        println!("   Base branch: {}", config.base_ref());
        jj.get_commits(config.base_ref(), "@")
            .context("Failed to get commits from jj")?
    };

    // Filter out empty commits
    let nonempty_commits: Vec<_> = commits.into_iter().filter(|c| !c.empty).collect();

    if nonempty_commits.is_empty() {
        println!("✅ No commits to sync");
        return Ok(());
    }

    if status_only {
        println!("Found {} commit(s) to check\n", nonempty_commits.len());
    } else {
        println!("Found {} commit(s) to sync", nonempty_commits.len());
    }

    let mut synced_count = 0;
    let mut skipped_count = 0;
    let mut error_count = 0;

    for (index, commit) in nonempty_commits.iter().enumerate() {
        if status_only {
            println!("📝 Commit {}/{}", index + 1, nonempty_commits.len());
        } else {
            println!("\n📝 Syncing commit {}/{}", index + 1, nonempty_commits.len());
        }
        println!("   Change ID: {}", commit.change_id);
        println!("   Message: {}", commit.description.lines().next().unwrap_or(""));

        // Skip commits that haven't been mailed yet
        if commit.pr_number.is_none() {
            println!("   ⏭️  Not mailed yet (run 'jj-pr mail' first)");
            skipped_count += 1;
            continue;
        }

        let pr_number = commit.pr_number.unwrap();
        println!("   PR: #{}", pr_number);

        // Construct branch name from change ID
        let branch_name = format!("spr/{}/{}", owner, commit.change_id);

        // Check if PR branch exists on remote
        if !git.remote_branch_exists(&branch_name, "origin") {
            println!("   ⏭️  No remote branch found");
            skipped_count += 1;
            continue;
        }

        // Fetch the remote branch
        if !status_only {
            print!("   ⬇️  Fetching remote branch...");
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }
        git.fetch_branch(&branch_name, "origin")
            .context("Failed to fetch remote branch")?;
        if !status_only {
            println!(" done");
        }

        // Get remote commit ID
        let remote_commit = git
            .get_remote_branch_commit(&branch_name, "origin")
            .context("Failed to get remote commit")?;

        // Check if commits are identical
        if remote_commit == commit.commit_id {
            println!("   ✅ In sync");
            synced_count += 1;
            continue;
        }

        // Check if remote has diverged from local (commits only on remote)
        let remote_only_commits = git
            .get_commits_between(&commit.commit_id, &remote_commit)
            .context("Failed to check remote divergence")?;

        // Check if local has diverged from remote (commits only on local)
        let local_only_commits = git
            .get_commits_between(&remote_commit, &commit.commit_id)
            .context("Failed to check local divergence")?;

        let has_remote_changes = !remote_only_commits.is_empty();
        let has_local_changes = !local_only_commits.is_empty();

        // Report status
        match (has_remote_changes, has_local_changes) {
            (true, true) => {
                println!(
                    "   ⚠️  Both local and remote have changes ({} remote, {} local)",
                    remote_only_commits.len(),
                    local_only_commits.len()
                );
                if status_only {
                    println!("      Run 'jj-pr sync' to merge (may require conflict resolution)");
                }
            }
            (true, false) => {
                println!("   ⬇️  Remote has {} new commit(s)", remote_only_commits.len());
                if status_only {
                    println!("      Run 'jj-pr sync' to pull changes");
                }
            }
            (false, true) => {
                println!("   ⬆️  Local has {} new commit(s)", local_only_commits.len());
                if status_only {
                    println!("      Run 'jj-pr sync' to push changes");
                }
            }
            (false, false) => {
                // Trees must differ even though no commits between them
                println!("   🔄 Out of sync (trees differ)");
                if status_only {
                    println!("      Run 'jj-pr sync' to synchronize");
                }
            }
        }

        // If status-only mode, skip actual sync
        if status_only {
            continue;
        }

        // Perform sync operations
        if has_remote_changes {
            println!("   🔄 Merging remote changes into local commit...");

            // Attempt to squash remote changes onto local
            match jj.squash(&remote_commit, &commit.change_id) {
                Ok(_) => {
                    println!("   ✅ Successfully merged remote changes");
                }
                Err(e) => {
                    let error_msg = format!("{:#}", e);
                    if error_msg.to_lowercase().contains("conflict") {
                        println!("   ❌ MERGE CONFLICT DETECTED");
                        println!("      JJ has created conflict markers in your working copy.");
                        println!("      Please:");
                        println!("        1. Resolve the conflicts manually");
                        println!("        2. Run 'jj squash' to finalize the merge");
                        println!("        3. Re-run 'jj-pr sync' to complete synchronization");
                        println!("\n   Error details: {}", error_msg);
                        error_count += 1;
                        continue;
                    } else {
                        return Err(e).context("Failed to squash remote changes");
                    }
                }
            }
        }

        // Get the updated commit ID after potential squash; use latest() in case the
        // change ID is temporarily divergent (e.g. left over from a mail annotation).
        let updated_commit_id = jj
            .resolve_change_id(&commit.change_id)
            .context("Failed to resolve updated commit")?;

        // Update local branch to point at updated commit
        git.create_branch(&branch_name, &updated_commit_id)
            .context("Failed to update local branch")?;

        // Push to remote if there were any changes (remote or local)
        if has_remote_changes || has_local_changes {
            println!("   ⬆️  Pushing to remote...");
            git.push_branch(&branch_name, "origin")
                .context("Failed to push updated branch")?;
            println!("   ✅ Synced successfully");
            synced_count += 1;
        }
    }

    // Print summary
    if status_only {
        println!("\n📊 Status Summary:");
        println!("   {} commit(s) in sync", synced_count);
        if skipped_count > 0 {
            println!("   {} commit(s) skipped", skipped_count);
        }
        if nonempty_commits.len() - synced_count - skipped_count > 0 {
            println!(
                "   {} commit(s) need syncing",
                nonempty_commits.len() - synced_count - skipped_count
            );
        }
    } else {
        if error_count > 0 {
            println!("\n⚠️  Sync completed with {} error(s)", error_count);
            println!("   {} commit(s) synced successfully", synced_count);
            println!("   {} commit(s) need conflict resolution", error_count);
        } else {
            println!("\n🎉 Successfully synced {} commit(s)", synced_count);
            if skipped_count > 0 {
                println!("   {} commit(s) skipped", skipped_count);
            }
        }
    }
    Ok(())
}
