use anyhow::{Context, Result};
use std::path::Path;

use crate::config::Config;
use crate::jj::Jj;

/// Remove the PR association (Pull Request: #N line) from commit messages
pub fn disassociate(
    repo_path: impl AsRef<Path>,
    base_override: Option<String>,
    revisions: Option<String>,
) -> Result<()> {
    let jj = Jj::new(&repo_path)?;
    let config = Config::resolve(&repo_path, base_override)?;

    println!("🔗 Finding associated commits...");

    let commits = if let Some(ref revset) = revisions {
        println!("   Using revset: {}", revset);
        jj.get_commits_from_revset(revset)
            .context("Failed to get commits from revset")?
    } else {
        println!("   Base branch: {}", config.base_ref());
        jj.get_commits(config.base_ref(), "@")
            .context("Failed to get commits from jj")?
    };

    let associated: Vec<_> = commits
        .into_iter()
        .filter(|c| !c.empty && c.pr_number.is_some())
        .collect();

    if associated.is_empty() {
        println!("✅ No commits with PR associations found");
        return Ok(());
    }

    println!("Found {} associated commit(s)\n", associated.len());

    let mut count = 0;
    for commit in &associated {
        let pr_number = commit.pr_number.unwrap();
        println!("📝 Commit: {}", commit.description.lines().next().unwrap_or(""));
        println!("   Change ID: {}", commit.change_id);
        println!("   Removing PR #{}", pr_number);

        let new_message = crate::message::remove_pr_number(&commit.description);
        jj.describe(&commit.change_id, &new_message)
            .context("Failed to update commit message")?;

        println!("   ✅ Disassociated");
        count += 1;
    }

    println!("\n🎉 Disassociated {} commit(s)", count);
    Ok(())
}
