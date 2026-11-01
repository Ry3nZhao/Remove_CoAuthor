use std::fs;
use std::path::{Path, PathBuf};

use crate::gitutil;
use crate::nomore::NoMoreError;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

fn make_executable(path: &Path) -> Result<(), NoMoreError> {
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms)?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

pub const MANAGED_MARKER: &str = "# rmai-managed: commit-msg v1";
pub const BACKUP_NAME: &str = "commit-msg.rmai-bak";

const HOOK_BODY: &str = r#"#!/bin/sh
# rmai-managed: commit-msg v1
# remove: rmai no-more uninstall
set -e
MSGFILE="$1"
if command -v rmai >/dev/null 2>&1; then
  rmai no-more run-commit-msg "$MSGFILE"
else
  echo "rmai: no-more hook present but rmai not in PATH; skipping" >&2
fi
HOOK_DIR=$(dirname "$0")
if [ -x "$HOOK_DIR/commit-msg.rmai-bak" ]; then
  exec "$HOOK_DIR/commit-msg.rmai-bak" "$@"
fi
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookState {
    Managed,
    Missing,
    ThirdParty,
    NotRepo,
}

pub fn hooks_dir(repo: &Path) -> Result<PathBuf, NoMoreError> {
    let git_dir = gitutil::git_dir(repo)?;
    Ok(git_dir.join("hooks"))
}

pub fn hook_state(repo: &Path) -> HookState {
    let hooks = match hooks_dir(repo) {
        Ok(h) => h,
        Err(_) => return HookState::NotRepo,
    };
    let path = hooks.join("commit-msg");
    if !path.exists() {
        return HookState::Missing;
    }
    match fs::read_to_string(&path) {
        Ok(content) if content.contains(MANAGED_MARKER) => HookState::Managed,
        Ok(_) => HookState::ThirdParty,
        Err(_) => HookState::ThirdParty,
    }
}

pub fn write_hook_script(path: &Path, _global: bool) -> Result<(), NoMoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, HOOK_BODY)?;
    make_executable(path)?;
    Ok(())
}

pub fn install_repo(repo: &Path) -> Result<(), NoMoreError> {
    let dir = hooks_dir(repo)?;
    fs::create_dir_all(&dir)?;
    let hook = dir.join("commit-msg");
    let backup = dir.join(BACKUP_NAME);

    if hook.exists() {
        let content = fs::read_to_string(&hook).unwrap_or_default();
        if content.contains(MANAGED_MARKER) {
            write_hook_script(&hook, false)?;
            return Ok(());
        }
        // Third-party: back up then wrap
        if !backup.exists() {
            fs::rename(&hook, &backup)?;
            make_executable(&backup)?;
        } else {
            fs::remove_file(&hook)?;
        }
    }

    write_hook_script(&hook, false)?;
    Ok(())
}

pub fn uninstall_repo(repo: &Path) -> Result<(), NoMoreError> {
    let dir = hooks_dir(repo)?;
    let hook = dir.join("commit-msg");
    let backup = dir.join(BACKUP_NAME);

    if hook.exists() {
        let content = fs::read_to_string(&hook).unwrap_or_default();
        if content.contains(MANAGED_MARKER) {
            fs::remove_file(&hook)?;
        } else {
            return Err(NoMoreError::Msg(format!(
                "commit-msg is not rmai-managed; refusing to delete {}",
                hook.display()
            )));
        }
    }

    if backup.exists() {
        fs::rename(&backup, &hook)?;
        make_executable(&hook)?;
    }
    Ok(())
}
