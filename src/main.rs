mod commands;
mod config;
mod git;
mod github;
mod jj;
mod message;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::env;

#[derive(Parser)]
#[command(name = "jj-pr")]
#[command(about = "A utility that facilitates stacked PRs with Jujutsu", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create PR branches for nonempty commits between @ and the base branch
    Mail {
        /// Base branch to compare against (e.g., "origin/master" or "origin/main")
        /// If not specified, uses saved config or auto-detects
        #[arg(short, long, conflicts_with = "revisions")]
        base: Option<String>,

        /// Revset expression to specify which commits to process (e.g., "origin/main..@")
        /// Mutually exclusive with --base
        #[arg(short, long)]
        revisions: Option<String>,
    },
    /// Pull down changes from PR branches and reset them to local commits
    Sync {
        /// Base branch to compare against (e.g., "origin/master" or "origin/main")
        /// If not specified, uses saved config or auto-detects
        #[arg(short, long, conflicts_with = "revisions")]
        base: Option<String>,

        /// Revset expression to specify which commits to process (e.g., "origin/main..@")
        /// Mutually exclusive with --base
        #[arg(short, long)]
        revisions: Option<String>,

        /// Show sync status without actually syncing
        #[arg(short, long)]
        status: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Get current directory as repo path
    let repo_path = env::current_dir()?;

    // Get GitHub token from environment
    let github_token = env::var("GITHUB_TOKEN")
        .or_else(|_| env::var("GH_TOKEN"))
        .expect("GITHUB_TOKEN or GH_TOKEN environment variable required");

    match cli.command {
        Commands::Mail { base, revisions } => {
            commands::mail::mail(repo_path, github_token, base, revisions).await?;
        }
        Commands::Sync { base, revisions, status } => {
            commands::sync::sync(repo_path, github_token, base, revisions, status).await?;
        }
    }

    Ok(())
}

