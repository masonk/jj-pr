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
    /// Remove the PR association from commit messages
    Disassociate {
        /// Base branch to compare against
        /// If not specified, uses saved config or auto-detects
        #[arg(short, long, conflicts_with = "revisions")]
        base: Option<String>,

        /// Revset expression to specify which commits to process
        /// Mutually exclusive with --base
        #[arg(short, long)]
        revisions: Option<String>,
    },
}

fn resolve_github_token() -> String {
    env::var("GITHUB_TOKEN")
        .or_else(|_| env::var("GH_TOKEN"))
        .or_else(|_| {
            std::process::Command::new("gh")
                .args(["auth", "token"])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .ok_or(env::VarError::NotPresent)
        })
        .unwrap_or_else(|_| {
            eprintln!("Error: GitHub authentication required.");
            eprintln!("Provide a token via GITHUB_TOKEN, or authenticate with the GitHub CLI:");
            eprintln!("  gh auth login");
            std::process::exit(1);
        })
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Get current directory as repo path
    let repo_path = env::current_dir()?;

    match cli.command {
        Commands::Mail { base, revisions } => {
            let token = resolve_github_token();
            commands::mail::mail(repo_path, token, base, revisions).await?;
        }
        Commands::Sync { base, revisions, status } => {
            let token = resolve_github_token();
            commands::sync::sync(repo_path, token, base, revisions, status).await?;
        }
        Commands::Disassociate { base, revisions } => {
            commands::disassociate::disassociate(repo_path, base, revisions)?;
        }
    }

    Ok(())
}

