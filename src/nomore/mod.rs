mod hook;
mod registry;

use std::io;
use std::path::{Path, PathBuf};

use crate::gitutil::{self, GitError};
use crate::patterns::strip_message;
use crate::whitelist;

pub use hook::{install_repo, uninstall_repo};
pub use registry::Supervised;

#[derive(Debug, thiserror::Error)]
pub enum NoMoreError {
    #[error(transparent)]
    Git(#[from] GitError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Config(#[from] crate::config::ConfigError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Msg(String),
}

pub fn install(path: Option<&Path>, all: bool) -> Result<(), NoMoreError> {
    if all {
        return install_global();
    }
    let repo = gitutil::resolve_repo(path.unwrap_or_else(|| Path::new(".")))?;
    install_repo(&repo)?;
    let mut reg = Supervised::load()?;
    reg.upsert_repo(&repo)?;
    reg.save()?;
    println!("no-more installed for {}", repo.display());
    Ok(())
}

pub fn uninstall(path: Option<&Path>, all: bool) -> Result<(), NoMoreError> {
    if all {
        return uninstall_global();
    }
    let repo = gitutil::resolve_repo(path.unwrap_or_else(|| Path::new(".")))?;
    uninstall_repo(&repo)?;
    let mut reg = Supervised::load()?;
    reg.remove_repo(&repo);
    reg.save()?;
    println!("no-more uninstalled for {}", repo.display());
    Ok(())
}

pub fn status(path: Option<&Path>) -> Result<(), NoMoreError> {
    let reg = Supervised::load()?;
    if let Some(p) = path {
        let repo = gitutil::resolve_repo(p)?;
        print_repo_status(&repo)?;
        return Ok(());
    }

    println!("global: {}", if reg.global { "on" } else { "off" });
    if let Ok(Some(hp)) = gitutil::config_get_global("core.hooksPath") {
        println!("core.hooksPath: {hp}");
    }
    if reg.repos.is_empty() {
        println!("supervised repos: (empty)");
        return Ok(());
    }
    println!("supervised repos:");
    for entry in &reg.repos {
        let path = PathBuf::from(&entry.path);
        let flag = if !path.exists() {
            "missing"
        } else {
            match hook::hook_state(&path) {
                hook::HookState::Managed => "ok",
                hook::HookState::Missing => "hook-gone",
                hook::HookState::ThirdParty => "third-party",
                hook::HookState::NotRepo => "not-repo",
            }
        };
        println!("  [{flag}] {}", entry.path);
    }
    Ok(())
}

fn print_repo_status(repo: &Path) -> Result<(), NoMoreError> {
    let state = hook::hook_state(repo);
    let label = match state {
        hook::HookState::Managed => "rmai-managed",
        hook::HookState::Missing => "not installed",
        hook::HookState::ThirdParty => "third-party hook present",
        hook::HookState::NotRepo => "not a git repo",
    };
    println!("{}: {label}", repo.display());
    Ok(())
}

fn install_global() -> Result<(), NoMoreError> {
    let hooks_dir = crate::config::global_hooks_dir()?;
    hook::write_hook_script(&hooks_dir.join("commit-msg"), true)?;

    let current = gitutil::config_get_global("core.hooksPath")?;
    let hooks_dir_str = hooks_dir.display().to_string();

    match current {
        None => {
            gitutil::config_set_global("core.hooksPath", &hooks_dir_str)?;
            registry::GlobalState {
                previous_hooks_path: None,
            }
            .save()?;
        }
        Some(ref hp) if paths_equal(hp, &hooks_dir_str) => {
            // already ours
        }
        Some(hp) => {
            return Err(NoMoreError::Msg(format!(
                "refusing to overwrite existing core.hooksPath={hp}\n\
                 Use per-repo: rmai no-more install <path>\n\
                 Or unset it first, then retry install --all"
            )));
        }
    }

    let mut reg = Supervised::load()?;
    reg.global = true;
    reg.save()?;
    println!("no-more global install enabled");
    println!("core.hooksPath -> {hooks_dir_str}");
    println!("Note: Git will no longer run per-repo .git/hooks while this is set.");
    Ok(())
}

fn uninstall_global() -> Result<(), NoMoreError> {
    let hooks_dir = crate::config::global_hooks_dir()?;
    let hooks_dir_str = hooks_dir.display().to_string();

    if let Some(hp) = gitutil::config_get_global("core.hooksPath")? {
        if paths_equal(&hp, &hooks_dir_str) {
            let state = registry::GlobalState::load().unwrap_or_default();
            match state.previous_hooks_path {
                Some(prev) => gitutil::config_set_global("core.hooksPath", &prev)?,
                None => gitutil::config_unset_global("core.hooksPath")?,
            }
        }
    }

    let commit_msg = hooks_dir.join("commit-msg");
    if commit_msg.exists() {
        std::fs::remove_file(&commit_msg)?;
    }

    let mut reg = Supervised::load()?;
    let repos: Vec<PathBuf> = reg.repos.iter().map(|r| PathBuf::from(&r.path)).collect();
    for repo in &repos {
        if repo.exists() {
            let _ = uninstall_repo(repo);
        }
    }
    reg.global = false;
    reg.repos.clear();
    reg.save()?;
    println!("no-more global uninstall complete; registry cleared");
    Ok(())
}

fn paths_equal(a: &str, b: &str) -> bool {
    let ca = PathBuf::from(a)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(a));
    let cb = PathBuf::from(b)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(b));
    ca == cb
}

pub fn run_commit_msg(msg_file: &Path) -> Result<(), NoMoreError> {
    let original = std::fs::read_to_string(msg_file)?;
    let wl = whitelist::load_or_default();
    let (stripped, changed) = strip_message(&original, &wl);
    if changed {
        std::fs::write(msg_file, stripped)?;
    }
    Ok(())
}
