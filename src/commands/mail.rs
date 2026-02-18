use anyhow::{Context, Result};
use std::path::Path;

use crate::config::Config;
use crate::git::Git;
use crate::github::GitHub;
use crate::jj::Jj;

/// Create PR branches for all nonempty commits between @ and the base branch
pub async fn mail(
    repo_path: impl AsRef<Path>,
    github_token: String,
    base_override: Option<String>,
    revisions: Option<String>,
) -> Result<()> {
    let jj = Jj::new(&repo_path)?;
    let git = Git::new(&repo_path)?;

    // Resolve base branch configuration
    let config = Config::resolve(&repo_path, base_override)?;

    // Parse GitHub owner/repo from remote
    let (owner, repo) = git.parse_github_repo("origin")?;
    let github = GitHub::new(github_token, owner.clone(), repo.clone())?;

    println!("📬 Finding commits to mail...");

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
        println!("✅ No nonempty commits to mail");
        return Ok(());
    }

    println!("Found {} nonempty commit(s)", nonempty_commits.len());

    // Process commits from oldest to newest (reverse order)
    // This ensures parent branches exist before child branches
    let mut previous_branch = config.base_ref().to_string();

    for (index, commit) in nonempty_commits.iter().rev().enumerate() {
        println!("\n📝 Processing commit {}/{}", index + 1, nonempty_commits.len());
        println!("   Change ID: {}", commit.change_id);
        println!("   Message: {}", commit.description.lines().next().unwrap_or(""));

        // Create branch name from change ID
        let branch_name = format!("spr/{}/{}", owner, commit.change_id);

        // Create/update local branch to point at this commit
        git.create_branch(&branch_name, &commit.commit_id)
            .context("Failed to create local branch")?;

        // Push branch to origin
        println!("   ⬆️  Pushing branch: {}", branch_name);
        git.push_branch(&branch_name, "origin")
            .context("Failed to push branch")?;

        let base_branch = previous_branch.trim_start_matches("origin/");

        // Check if commit already has a PR number in its message
        if let Some(pr_num) = commit.pr_number {
            println!("   📌 Found existing PR #{} in commit message", pr_num);

            // Update base branch if needed
            github
                .update_pull_request(pr_num, None, None, Some(base_branch.to_string()))
                .await
                .context("Failed to update PR base branch")?;
        } else {
            // No PR number in commit - check if PR exists on GitHub
            match github.find_pull_request_by_branch(&branch_name).await? {
                Some(pr) => {
                    println!("   🔄 Found existing PR #{} on GitHub", pr.number);

                    // Update base branch if needed
                    if pr.base != base_branch {
                        println!("   🔄 Updating base branch to: {}", base_branch);
                        github
                            .update_pull_request(pr.number, None, None, Some(base_branch.to_string()))
                            .await
                            .context("Failed to update PR base branch")?;
                    }

                    // Add PR number to commit message
                    println!("   📝 Adding PR #{} to commit message", pr.number);
                    jj.add_pr_to_commit(&commit.change_id, &commit.description, pr.number)
                        .context("Failed to update commit message")?;
                }
                None => {
                    // Create new PR
                    let title = commit.description.lines().next().unwrap_or("").to_string();
                    let body = commit
                        .description
                        .lines()
                        .skip(1)
                        .collect::<Vec<_>>()
                        .join("\n")
                        .trim()
                        .to_string();

                    println!("   ✨ Creating PR with base: {}", base_branch);
                    let pr = github
                        .create_pull_request(title, body, branch_name.clone(), base_branch.to_string())
                        .await
                        .context("Failed to create PR")?;

                    println!("   ✅ Created PR #{}: https://github.com/{}/{}/pull/{}",
                             pr.number, owner, repo, pr.number);

                    // Add PR number to commit message
                    println!("   📝 Adding PR #{} to commit message", pr.number);
                    jj.add_pr_to_commit(&commit.change_id, &commit.description, pr.number)
                        .context("Failed to update commit message")?;
                }
            }
        }

        // Update previous_branch for next iteration (for stacking)
        previous_branch = branch_name;
    }

    println!("\n🎉 Successfully mailed {} commit(s)", nonempty_commits.len());
    Ok(())
}
