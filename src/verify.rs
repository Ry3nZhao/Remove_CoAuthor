use std::collections::BTreeMap;
use std::path::Path;

use crate::gitutil::{self, GitError};

/// Collect multiset of tree OIDs for all commits reachable from --all.
pub fn collect_tree_multiset(repo: &Path) -> Result<BTreeMap<String, usize>, GitError> {
    let out = gitutil::run_git_ok(repo, &["log", "--all", "--format=%T"])?;
    let mut map = BTreeMap::new();
    for line in out.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        *map.entry(t.to_string()).or_insert(0) += 1;
    }
    Ok(map)
}

pub fn assert_trees_unchanged(
    before: &BTreeMap<String, usize>,
    after: &BTreeMap<String, usize>,
) -> Result<(), GitError> {
    if before == after {
        println!(
            "Verified: commit tree OIDs unchanged ({})",
            before.values().sum::<usize>()
        );
        return Ok(());
    }
    Err(GitError::Command(format!(
        "tree OID multiset changed after rewrite (before {} unique trees / {} commits, after {} / {}). Restore from refs/rmai-backup/.",
        before.len(),
        before.values().sum::<usize>(),
        after.len(),
        after.values().sum::<usize>(),
    )))
}

pub fn worktree_paths_unchanged(repo: &Path) -> Result<(), GitError> {
    // After rewrite HEAD content should match; porcelain should still be clean
    let status = gitutil::run_git_ok(repo, &["status", "--porcelain"])?;
    if !status.trim().is_empty() {
        return Err(GitError::Command(
            "working tree became dirty after rewrite".into(),
        ));
    }
    Ok(())
}
