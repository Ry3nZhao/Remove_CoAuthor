use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "rmai",
    version,
    about = "Remove AI co-author trailers and related markers from Git history",
    long_about = None
)]
pub struct Cli {
    /// Scan only; do not rewrite (implies no confirmation)
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Clean with confirmation (same as `clean`)
    #[arg(short = 'c', long = "clean-confirm", global = true)]
    pub clean_flag: bool,

    /// Clean without confirmation (same as `clean --yes`)
    #[arg(short = 'C', global = true)]
    pub clean_yes_flag: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Repository path (default: .) when using top-level -c/-C/--dry-run
    pub path: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Scan and rewrite history to remove AI attribution markers
    Clean {
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
        /// Repository path
        path: Option<PathBuf>,
    },
    /// Install/uninstall commit-msg hooks that strip AI markers on new commits
    #[command(name = "no-more")]
    NoMore {
        #[command(subcommand)]
        action: NoMoreCmd,
    },
    /// Manage global identity whitelist (never strip these co-authors)
    Whitelist {
        #[command(subcommand)]
        action: Option<WhitelistCmd>,
    },
}

#[derive(Debug, Subcommand)]
pub enum NoMoreCmd {
    /// Install hook for a repo, or --all for global hooksPath
    Install {
        #[arg(long)]
        all: bool,
        path: Option<PathBuf>,
    },
    /// Uninstall hook for a repo, or --all for global + all registered repos
    Uninstall {
        #[arg(long)]
        all: bool,
        path: Option<PathBuf>,
    },
    /// List supervised repos (no path) or show one repo's status
    Status { path: Option<PathBuf> },
    /// Internal: strip AI markers from a commit-msg file
    #[command(name = "run-commit-msg", hide = true)]
    RunCommitMsg { msg_file: PathBuf },
}

#[derive(Debug, Subcommand)]
pub enum WhitelistCmd {
    /// Add an identity (email or Name <email>)
    Add { identity: String },
    /// Remove an identity
    Remove { identity: String },
    /// List whitelist entries
    List,
    /// Alias for list
    Status,
}
