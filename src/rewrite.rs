use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use crate::backup;
use crate::gitutil::{self, GitError};
use crate::patterns::{is_ai_author_name, is_ai_email, strip_message};
use crate::verify;
use crate::whitelist::Whitelist;

pub fn clean_repo(repo: &Path, whitelist: &Whitelist, skip_confirm: bool) -> Result<(), GitError> {
    gitutil::ensure_clean_worktree(repo)?;

    let report = crate::scan::scan_repo(repo, whitelist)?;
    report.print_summary();
    if report.is_empty() {
        return Ok(());
    }

    if !skip_confirm {
        eprint!("Rewrite history for these markers? [y/N] ");
        let _ = std::io::stderr().flush();
        let mut answer = String::new();
        std::io::stdin()
            .read_line(&mut answer)
            .map_err(GitError::Io)?;
        let answer = answer.trim().to_ascii_lowercase();
        if answer != "y" && answer != "yes" {
            println!("Aborted.");
            return Ok(());
        }
    }

    let trees_before = verify::collect_tree_multiset(repo)?;
    let backup_prefix = backup::backup_refs(repo)?;

    rewrite_via_fast_export(repo, whitelist)?;
    rewrite_notes(repo, whitelist)?;

    let trees_after = verify::collect_tree_multiset(repo)?;
    if let Err(e) = verify::assert_trees_unchanged(&trees_before, &trees_after) {
        backup::print_restore_hint(&backup_prefix);
        return Err(e);
    }
    verify::worktree_paths_unchanged(repo)?;

    println!();
    println!("History rewrite complete. Commit SHAs have changed.");
    println!("Signed commits/tags may need re-signing.");
    println!("To publish (coordinate with collaborators):");
    println!("  git push --force-with-lease --all");
    println!("  git push --force-with-lease --tags");
    backup::print_restore_hint(&backup_prefix);
    Ok(())
}

fn rewrite_via_fast_export(repo: &Path, whitelist: &Whitelist) -> Result<(), GitError> {
    let mut export = Command::new("git")
        .args([
            "fast-export",
            "--all",
            "--signed-tags=strip",
            "--tag-of-filtered-object=rewrite",
        ])
        .current_dir(repo)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(GitError::Io)?;

    let mut import = Command::new("git")
        .args(["fast-import", "--force", "--quiet"])
        .current_dir(repo)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(GitError::Io)?;

    {
        let export_out = export.stdout.take().expect("export stdout");
        let mut import_in = import.stdin.take().expect("import stdin");
        filter_fast_export(export_out, &mut import_in, whitelist)?;
    }

    let export_status = export.wait().map_err(GitError::Io)?;
    let import_output = import.wait_with_output().map_err(GitError::Io)?;

    if !export_status.success() {
        return Err(GitError::Command(format!(
            "git fast-export failed: {export_status}"
        )));
    }
    if !import_output.status.success() {
        return Err(GitError::Command(format!(
            "git fast-import failed: {}",
            String::from_utf8_lossy(&import_output.stderr)
        )));
    }

    // Expire reflog noise optional; reset hard to sync index if needed
    let _ = gitutil::run_git_ok(repo, &["reset", "--hard"]);
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExportCtx {
    None,
    Commit,
    Tag,
    Blob,
}

fn filter_fast_export<R: std::io::Read, W: Write>(
    reader: R,
    writer: &mut W,
    whitelist: &Whitelist,
) -> Result<(), GitError> {
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    let mut ctx = ExportCtx::None;
    let mut pending_author: Option<IdentityLine> = None;
    let mut pending_committer: Option<IdentityLine> = None;

    loop {
        line.clear();
        let n = reader.read_line(&mut line).map_err(GitError::Io)?;
        if n == 0 {
            break;
        }

        if line.starts_with("commit ") {
            flush_identities(
                writer,
                &mut pending_author,
                &mut pending_committer,
                whitelist,
            )?;
            ctx = ExportCtx::Commit;
            writer.write_all(line.as_bytes()).map_err(GitError::Io)?;
            continue;
        }

        if line.starts_with("tag ") {
            flush_identities(
                writer,
                &mut pending_author,
                &mut pending_committer,
                whitelist,
            )?;
            ctx = ExportCtx::Tag;
            writer.write_all(line.as_bytes()).map_err(GitError::Io)?;
            continue;
        }

        if line.starts_with("blob") {
            flush_identities(
                writer,
                &mut pending_author,
                &mut pending_committer,
                whitelist,
            )?;
            ctx = ExportCtx::Blob;
            writer.write_all(line.as_bytes()).map_err(GitError::Io)?;
            continue;
        }

        if line.starts_with("reset ")
            || line.starts_with("progress ")
            || line.starts_with("feature ")
            || line.starts_with("checkpoint")
        {
            flush_identities(
                writer,
                &mut pending_author,
                &mut pending_committer,
                whitelist,
            )?;
            ctx = ExportCtx::None;
            writer.write_all(line.as_bytes()).map_err(GitError::Io)?;
            continue;
        }

        if ctx == ExportCtx::Commit && line.starts_with("author ") {
            pending_author = Some(parse_identity_line("author", &line));
            continue;
        }
        if ctx == ExportCtx::Commit && line.starts_with("committer ") {
            pending_committer = Some(parse_identity_line("committer", &line));
            flush_identities(
                writer,
                &mut pending_author,
                &mut pending_committer,
                whitelist,
            )?;
            continue;
        }

        if let Some(rest) = line.strip_prefix("data ") {
            let size: usize = rest
                .trim()
                .parse()
                .map_err(|e| GitError::Command(format!("bad data size: {e}")))?;
            let mut buf = vec![0u8; size];
            reader.read_exact_compat(&mut buf)?;
            let bytes = if ctx == ExportCtx::Commit || ctx == ExportCtx::Tag {
                let original = String::from_utf8_lossy(&buf);
                let (stripped, _) = strip_message(&original, whitelist);
                stripped.into_bytes()
            } else {
                buf
            };
            writeln!(writer, "data {}", bytes.len()).map_err(GitError::Io)?;
            writer.write_all(&bytes).map_err(GitError::Io)?;
            continue;
        }

        writer.write_all(line.as_bytes()).map_err(GitError::Io)?;
    }

    flush_identities(
        writer,
        &mut pending_author,
        &mut pending_committer,
        whitelist,
    )?;
    writer.flush().map_err(GitError::Io)?;
    Ok(())
}

trait ReadExactCompat {
    fn read_exact_compat(&mut self, buf: &mut [u8]) -> Result<(), GitError>;
}

impl<R: BufRead> ReadExactCompat for R {
    fn read_exact_compat(&mut self, buf: &mut [u8]) -> Result<(), GitError> {
        std::io::Read::read_exact(self, buf).map_err(GitError::Io)
    }
}

#[derive(Clone)]
struct IdentityLine {
    kind: &'static str, // author | committer
    name: String,
    email: String,
    rest: String, // " timestamp tz\n" including leading space and newline
    raw: String,
}

fn parse_identity_line(kind: &'static str, line: &str) -> IdentityLine {
    // author Name <email> 12345 +0000\n
    let raw = line.to_string();
    let without_kind = line[kind.len()..].trim_start();
    if let Some(email_start) = without_kind.rfind(" <") {
        if let Some(email_end) = without_kind[email_start..].find('>') {
            let name = without_kind[..email_start].trim().to_string();
            let email = without_kind[email_start + 2..email_start + email_end].to_string();
            let rest = without_kind[email_start + email_end + 1..].to_string();
            return IdentityLine {
                kind,
                name,
                email,
                rest,
                raw,
            };
        }
    }
    IdentityLine {
        kind,
        name: String::new(),
        email: String::new(),
        rest: String::new(),
        raw,
    }
}

fn identity_is_ai(id: &IdentityLine) -> bool {
    is_ai_email(&id.email) || is_ai_author_name(&id.name)
}

fn format_identity(id: &IdentityLine) -> String {
    if id.name.is_empty() && id.email.is_empty() {
        return id.raw.clone();
    }
    format!("{} {} <{}>{}", id.kind, id.name, id.email, id.rest)
}

fn flush_identities<W: Write>(
    writer: &mut W,
    author: &mut Option<IdentityLine>,
    committer: &mut Option<IdentityLine>,
    _whitelist: &Whitelist,
) -> Result<(), GitError> {
    if author.is_none() && committer.is_none() {
        return Ok(());
    }

    let mut a = author.take();
    let mut c = committer.take();

    if let (Some(ref mut aa), Some(ref cc)) = (&mut a, &c) {
        if identity_is_ai(aa) && !identity_is_ai(cc) {
            aa.name = cc.name.clone();
            aa.email = cc.email.clone();
        }
    }
    if let (Some(ref aa), Some(ref mut cc)) = (&a, &mut c) {
        if identity_is_ai(cc) && !identity_is_ai(aa) {
            cc.name = aa.name.clone();
            cc.email = aa.email.clone();
        }
    }

    if let Some(aa) = a {
        if identity_is_ai(&aa) {
            eprintln!(
                "warning: leaving AI author identity (no human side): {} <{}>",
                aa.name, aa.email
            );
        }
        writer
            .write_all(format_identity(&aa).as_bytes())
            .map_err(GitError::Io)?;
    }
    if let Some(cc) = c {
        if identity_is_ai(&cc) {
            eprintln!(
                "warning: leaving AI committer identity (no human side): {} <{}>",
                cc.name, cc.email
            );
        }
        writer
            .write_all(format_identity(&cc).as_bytes())
            .map_err(GitError::Io)?;
    }
    Ok(())
}

fn rewrite_notes(repo: &Path, whitelist: &Whitelist) -> Result<(), GitError> {
    let (ok, list, _) = gitutil::run_git_status(repo, &["notes", "list"])?;
    if !ok || list.trim().is_empty() {
        return Ok(());
    }
    for line in list.lines() {
        let mut bits = line.split_whitespace();
        let note_oid = match bits.next() {
            Some(v) => v,
            None => continue,
        };
        let target = match bits.next() {
            Some(v) => v,
            None => continue,
        };
        let content = match gitutil::run_git_ok(repo, &["cat-file", "-p", note_oid]) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let (stripped, changed) = strip_message(&content, whitelist);
        if !changed {
            continue;
        }
        // Replace note content
        let _ = gitutil::pipe_git(
            repo,
            &["notes", "add", "-f", "-F", "-", target],
            stripped.as_bytes(),
        )?;
    }
    Ok(())
}
