use std::path::Path;

use chrono::Utc;

use crate::gitutil::{self, GitError};

/// Backup all refs under refs/rmai-backup/<timestamp>/ mirroring refs/heads, tags, notes, etc.
pub fn backup_refs(repo: &Path) -> Result<String, GitError> {
    let stamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let prefix = format!("refs/rmai-backup/{stamp}");

    let refs = gitutil::run_git_ok(
        repo,
        &[
            "for-each-ref",
            "--format=%(refname)%00%(objectname)",
            "refs/heads",
            "refs/tags",
            "refs/notes",
        ],
    )?;

    let mut count = 0usize;
    for line in refs.split('\n') {
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split('\0');
        let refname = match parts.next() {
            Some(r) => r,
            None => continue,
        };
        let oid = match parts.next() {
            Some(o) => o,
            None => continue,
        };
        // refs/heads/main -> refs/rmai-backup/STAMP/heads/main
        let suffix = refname.strip_prefix("refs/").unwrap_or(refname);
        let dest = format!("{prefix}/{suffix}");
        gitutil::run_git_ok(repo, &["update-ref", &dest, oid])?;
        count += 1;
    }

    if count == 0 {
        // Still create a marker so user knows backup ran
        let _ = stamp;
    }

    println!("Backup saved under {prefix} ({count} ref(s))");
    Ok(prefix)
}

pub fn print_restore_hint(backup_prefix: &str) {
    eprintln!("If you need to restore, reset refs from {backup_prefix}/… or re-clone from remote.");
}
