use std::collections::BTreeMap;
use std::path::Path;

use crate::gitutil::{self, GitError};
use crate::patterns::{is_ai_author_name, is_ai_email, message_has_ai_markers};
use crate::whitelist::Whitelist;

#[derive(Debug, Clone)]
pub struct Finding {
    pub kind: FindingKind,
    pub id: String,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingKind {
    CommitMessage,
    CommitIdentity,
    TagMessage,
    Note,
}

#[derive(Debug, Default)]
pub struct ScanReport {
    pub findings: Vec<Finding>,
}

impl ScanReport {
    pub fn is_empty(&self) -> bool {
        self.findings.is_empty()
    }

    pub fn count_by_kind(&self) -> BTreeMap<&'static str, usize> {
        let mut m = BTreeMap::new();
        for f in &self.findings {
            let k = match f.kind {
                FindingKind::CommitMessage => "commit-message",
                FindingKind::CommitIdentity => "commit-identity",
                FindingKind::TagMessage => "tag-message",
                FindingKind::Note => "note",
            };
            *m.entry(k).or_insert(0) += 1;
        }
        m
    }

    pub fn print_summary(&self) {
        if self.is_empty() {
            println!("No AI attribution markers found.");
            return;
        }
        println!("Found {} marker(s):", self.findings.len());
        for (k, n) in self.count_by_kind() {
            println!("  {k}: {n}");
        }
        println!();
        for f in &self.findings {
            let kind = match f.kind {
                FindingKind::CommitMessage => "commit",
                FindingKind::CommitIdentity => "identity",
                FindingKind::TagMessage => "tag",
                FindingKind::Note => "note",
            };
            println!("  [{kind}] {} — {}", &f.id[..f.id.len().min(12)], f.summary);
        }
    }
}

pub fn scan_repo(repo: &Path, whitelist: &Whitelist) -> Result<ScanReport, GitError> {
    let mut report = ScanReport::default();
    scan_commits(repo, whitelist, &mut report)?;
    scan_tags(repo, whitelist, &mut report)?;
    scan_notes(repo, whitelist, &mut report)?;
    Ok(report)
}

fn scan_commits(
    repo: &Path,
    whitelist: &Whitelist,
    report: &mut ScanReport,
) -> Result<(), GitError> {
    // %H %an %ae %cn %ce %B with record separator
    let fmt = "%H%x1f%an%x1f%ae%x1f%cn%x1f%ce%x1f%B%x1e";
    let out = gitutil::run_git_ok(repo, &["log", "--all", &format!("--format={fmt}")])?;
    for record in out.split('\u{1e}') {
        let record = record.trim_matches('\n');
        if record.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = record.split('\u{1f}').collect();
        if parts.len() < 6 {
            continue;
        }
        let hash = parts[0].trim();
        let an = parts[1].trim();
        let ae = parts[2].trim();
        let cn = parts[3].trim();
        let ce = parts[4].trim();
        let body = parts[5];

        if message_has_ai_markers(body, whitelist) {
            let subject = body.lines().next().unwrap_or("").trim();
            report.findings.push(Finding {
                kind: FindingKind::CommitMessage,
                id: hash.to_string(),
                summary: subject.to_string(),
            });
        }

        let author_ai = is_ai_email(ae) || is_ai_author_name(an);
        let committer_ai = is_ai_email(ce) || is_ai_author_name(cn);
        if author_ai || committer_ai {
            let mut bits = Vec::new();
            if author_ai {
                bits.push(format!("author={an} <{ae}>"));
            }
            if committer_ai {
                bits.push(format!("committer={cn} <{ce}>"));
            }
            report.findings.push(Finding {
                kind: FindingKind::CommitIdentity,
                id: hash.to_string(),
                summary: bits.join(", "),
            });
        }
    }
    Ok(())
}

fn scan_tags(repo: &Path, whitelist: &Whitelist, report: &mut ScanReport) -> Result<(), GitError> {
    let out = gitutil::run_git_ok(
        repo,
        &[
            "for-each-ref",
            "refs/tags",
            "--format=%(objecttype)%00%(refname:short)%00%(objectname)%00%(contents)",
        ],
    )?;
    for line in out.split('\n') {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\0').collect();
        if parts.len() < 4 {
            continue;
        }
        if parts[0] != "tag" {
            continue;
        }
        let name = parts[1];
        let oid = parts[2];
        let contents = parts[3];
        if message_has_ai_markers(contents, whitelist) {
            report.findings.push(Finding {
                kind: FindingKind::TagMessage,
                id: oid.to_string(),
                summary: format!("tag {name}"),
            });
        }
    }
    Ok(())
}

fn scan_notes(repo: &Path, whitelist: &Whitelist, report: &mut ScanReport) -> Result<(), GitError> {
    let (ok, out, _) = gitutil::run_git_status(
        repo,
        &[
            "for-each-ref",
            "refs/notes",
            "--format=%(refname)%00%(objectname)",
        ],
    )?;
    if !ok || out.trim().is_empty() {
        return Ok(());
    }
    // List notes via git notes list if available
    let (ok2, list, _) = gitutil::run_git_status(repo, &["notes", "list"])?;
    if !ok2 {
        return Ok(());
    }
    for line in list.lines() {
        let mut bits = line.split_whitespace();
        let note_oid = match bits.next() {
            Some(v) => v,
            None => continue,
        };
        let target = bits.next().unwrap_or("");
        let (ok3, content, _) = gitutil::run_git_status(repo, &["cat-file", "-p", note_oid])?;
        if !ok3 {
            continue;
        }
        if message_has_ai_markers(&content, whitelist) {
            report.findings.push(Finding {
                kind: FindingKind::Note,
                id: note_oid.to_string(),
                summary: format!("note on {}", &target[..target.len().min(12)]),
            });
        }
    }
    Ok(())
}
