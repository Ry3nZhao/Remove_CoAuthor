mod backup;
mod cli;
mod config;
mod gitutil;
mod nomore;
mod patterns;
mod rewrite;
mod scan;
mod verify;
mod whitelist;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;

use cli::{Cli, Commands, NoMoreCmd, WhitelistCmd};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Top-level shorthand flags
    if cli.dry_run || cli.clean_flag || cli.clean_yes_flag {
        if cli.command.is_some() {
            return Err("use either a subcommand or -c/-C/--dry-run, not both".into());
        }
        let path = cli.path.unwrap_or_else(|| PathBuf::from("."));
        if cli.dry_run {
            return dry_run(&path);
        }
        let yes = cli.clean_yes_flag;
        return do_clean(&path, yes);
    }

    match cli.command {
        None => {
            eprintln!("Usage: rmai [--dry-run|-c|-C] [PATH]");
            eprintln!("       rmai clean [--yes] [PATH]");
            eprintln!("       rmai no-more <install|uninstall|status> ...");
            eprintln!("       rmai whitelist [add|remove|list] ...");
            eprintln!("Try `rmai --help`");
            Err("missing command".into())
        }
        Some(Commands::Clean { yes, path }) => {
            let path = path.unwrap_or_else(|| PathBuf::from("."));
            do_clean(&path, yes)
        }
        Some(Commands::NoMore { action }) => match action {
            NoMoreCmd::Install { all, path } => {
                nomore::install(path.as_deref(), all)?;
                Ok(())
            }
            NoMoreCmd::Uninstall { all, path } => {
                nomore::uninstall(path.as_deref(), all)?;
                Ok(())
            }
            NoMoreCmd::Status { path } => {
                nomore::status(path.as_deref())?;
                Ok(())
            }
            NoMoreCmd::RunCommitMsg { msg_file } => {
                nomore::run_commit_msg(&msg_file)?;
                Ok(())
            }
        },
        Some(Commands::Whitelist { action }) => {
            match action.unwrap_or(WhitelistCmd::List) {
                WhitelistCmd::Add { identity } => whitelist::cmd_add(&identity)?,
                WhitelistCmd::Remove { identity } => whitelist::cmd_remove(&identity)?,
                WhitelistCmd::List | WhitelistCmd::Status => {
                    whitelist::cmd_list(&mut std::io::stdout())?
                }
            }
            Ok(())
        }
    }
}

fn dry_run(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let repo = gitutil::resolve_repo(path)?;
    let wl = whitelist::load_or_default();
    let report = scan::scan_repo(&repo, &wl)?;
    println!("Repository: {}", repo.display());
    report.print_summary();
    Ok(())
}

fn do_clean(path: &Path, yes: bool) -> Result<(), Box<dyn std::error::Error>> {
    let repo = gitutil::resolve_repo(path)?;
    let wl = whitelist::load_or_default();
    rewrite::clean_repo(&repo, &wl, yes)?;
    Ok(())
}
