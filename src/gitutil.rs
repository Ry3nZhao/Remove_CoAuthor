use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("not a git repository: {0}")]
    NotRepo(String),
    #[error("git command failed: {0}")]
    Command(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("working tree is dirty; commit or stash changes first")]
    DirtyWorkTree,
}

pub fn run_git(repo: &Path, args: &[&str]) -> Result<Output, GitError> {
    let output = Command::new("git").args(args).current_dir(repo).output()?;
    Ok(output)
}

pub fn run_git_ok(repo: &Path, args: &[&str]) -> Result<String, GitError> {
    let output = run_git(repo, args)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::Command(format!(
            "git {} => {}\n{}",
            args.join(" "),
            output.status,
            stderr.trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn run_git_status(repo: &Path, args: &[&str]) -> Result<(bool, String, String), GitError> {
    let output = run_git(repo, args)?;
    Ok((
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    ))
}

pub fn resolve_repo(path: &Path) -> Result<PathBuf, GitError> {
    let path = if path.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        path.to_path_buf()
    };
    let abs = fs_canonicalize(&path)?;
    let (ok, stdout, stderr) = run_git_status(&abs, &["rev-parse", "--show-toplevel"])?;
    if !ok {
        return Err(GitError::NotRepo(format!(
            "{} ({})",
            abs.display(),
            stderr.trim()
        )));
    }
    let top = stdout.trim();
    Ok(PathBuf::from(top))
}

fn fs_canonicalize(path: &Path) -> Result<PathBuf, GitError> {
    if path.exists() {
        Ok(path.canonicalize()?)
    } else {
        Err(GitError::NotRepo(path.display().to_string()))
    }
}

pub fn ensure_clean_worktree(repo: &Path) -> Result<(), GitError> {
    let status = run_git_ok(repo, &["status", "--porcelain"])?;
    if !status.trim().is_empty() {
        return Err(GitError::DirtyWorkTree);
    }
    Ok(())
}

pub fn git_dir(repo: &Path) -> Result<PathBuf, GitError> {
    let out = run_git_ok(repo, &["rev-parse", "--git-dir"])?;
    let p = PathBuf::from(out.trim());
    if p.is_absolute() {
        Ok(p)
    } else {
        Ok(repo.join(p))
    }
}

pub fn config_get_global(key: &str) -> Result<Option<String>, GitError> {
    let output = Command::new("git")
        .args(["config", "--global", "--get", key])
        .output()?;
    if output.status.success() {
        let v = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if v.is_empty() {
            Ok(None)
        } else {
            Ok(Some(v))
        }
    } else {
        Ok(None)
    }
}

pub fn config_set_global(key: &str, value: &str) -> Result<(), GitError> {
    let status = Command::new("git")
        .args(["config", "--global", key, value])
        .status()?;
    if !status.success() {
        return Err(GitError::Command(format!(
            "failed to set git config --global {key}"
        )));
    }
    Ok(())
}

pub fn config_unset_global(key: &str) -> Result<(), GitError> {
    let _ = Command::new("git")
        .args(["config", "--global", "--unset", key])
        .status()?;
    Ok(())
}

pub fn pipe_git(repo: &Path, args: &[&str], stdin_data: &[u8]) -> Result<Vec<u8>, GitError> {
    use std::io::Write;
    let mut child = Command::new("git")
        .args(args)
        .current_dir(repo)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(stdin_data)?;
    }
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(GitError::Command(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(output.stdout)
}
