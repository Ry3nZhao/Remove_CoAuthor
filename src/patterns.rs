use regex::Regex;
use std::sync::OnceLock;

use crate::whitelist::Whitelist;

fn ai_email_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)(noreply@anthropic\.com|cursoragent@cursor\.com|noreply@openai\.com|copilot@github\.com|.*\+copilot\[bot\]@users\.noreply\.github\.com|noreply@google\.com|devin-ai-.*@.*|.*@aider\.chat)",
        )
        .expect("email regex")
    })
}

fn co_authored_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)^co-authored-by:\s*(.+)$").expect("co-authored regex"))
}

fn ai_name_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^(claude(\s+code)?|codex|cursor(\s+agent)?|github\s+copilot|copilot|gemini(\s+cli)?|devin|aider|chatgpt|openai|anthropic|gpt-?\d*)\b",
        )
        .expect("name regex")
    })
}

fn footer_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^(generated\s+with\s+claude\s+code|made\s+with\s+cursor|made-with:\s*cursor|claude-session:|generated\s+by\s+(claude|codex|cursor|copilot|gemini|devin|aider)\b).*",
        )
        .expect("footer regex")
    })
}

fn aider_author_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)\(aider\)").expect("aider regex"))
}

/// Known AI bot emails for author/committer identity rewriting.
pub fn is_ai_email(email: &str) -> bool {
    ai_email_re().is_match(email.trim())
}

pub fn is_ai_author_name(name: &str) -> bool {
    let name = name.trim();
    if aider_author_re().is_match(name) {
        return true;
    }
    ai_name_re().is_match(name)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineKind {
    Keep,
    Strip,
}

pub fn classify_line(line: &str, whitelist: &Whitelist) -> LineKind {
    let trimmed = line.trim_end();
    if trimmed.is_empty() {
        return LineKind::Keep;
    }

    if footer_re().is_match(trimmed) {
        // Footers have no identity; whitelist does not protect them.
        return LineKind::Strip;
    }

    if let Some(caps) = co_authored_re().captures(trimmed) {
        let value = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
        if whitelist.protects_line(trimmed) || whitelist.protects_line(value) {
            return LineKind::Keep;
        }
        if is_ai_coauthor_value(value) {
            return LineKind::Strip;
        }
        return LineKind::Keep;
    }

    LineKind::Keep
}

fn is_ai_coauthor_value(value: &str) -> bool {
    if ai_email_re().is_match(value) {
        return true;
    }
    // "Name <email>"
    if let Some(start) = value.find('<') {
        if let Some(end) = value[start..].find('>') {
            let name = value[..start].trim();
            let email = &value[start + 1..start + end];
            if ai_email_re().is_match(email) || is_ai_author_name(name) {
                return true;
            }
        }
    }
    is_ai_author_name(value)
}

/// Strip AI markers from a commit/tag/note message. Returns (new_message, changed).
pub fn strip_message(message: &str, whitelist: &Whitelist) -> (String, bool) {
    let had_trailing_newline = message.ends_with('\n');
    let lines: Vec<&str> = message.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut changed = false;

    for line in &lines {
        match classify_line(line, whitelist) {
            LineKind::Keep => out.push((*line).to_string()),
            LineKind::Strip => changed = true,
        }
    }

    // Collapse runs of 3+ blank lines to 2; trim trailing blank lines at end of body.
    let collapsed = collapse_blank_lines(&out);
    let mut result = collapsed.join("\n");
    // Ensure conventional single trailing newline for git messages when original had one
    // or when non-empty.
    if !result.is_empty() {
        if (had_trailing_newline || message.contains('\n')) && !result.ends_with('\n') {
            result.push('\n');
        }
        // Trim excess trailing newlines to one
        while result.ends_with("\n\n") {
            result.pop();
        }
    } else if had_trailing_newline {
        result.push('\n');
    }

    if result != message {
        changed = true;
    }
    (result, changed)
}

fn collapse_blank_lines(lines: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut blank_run = 0usize;
    for line in lines {
        if line.trim().is_empty() {
            blank_run += 1;
            if blank_run <= 1 {
                out.push(String::new());
            }
        } else {
            blank_run = 0;
            out.push(line.clone());
        }
    }
    // Remove trailing blank lines
    while out.last().is_some_and(|l| l.trim().is_empty()) {
        out.pop();
    }
    out
}

pub fn message_has_ai_markers(message: &str, whitelist: &Whitelist) -> bool {
    message
        .lines()
        .any(|line| classify_line(line, whitelist) == LineKind::Strip)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::whitelist::Whitelist;

    #[test]
    fn strips_claude_trailer() {
        let wl = Whitelist::default();
        let msg = "fix: hello\n\nGenerated with Claude Code\nCo-Authored-By: Claude <noreply@anthropic.com>\n";
        let (out, changed) = strip_message(msg, &wl);
        assert!(changed);
        assert!(!out.to_ascii_lowercase().contains("co-authored-by"));
        assert!(!out.to_ascii_lowercase().contains("generated with"));
        assert!(out.contains("fix: hello"));
    }

    #[test]
    fn keeps_human_coauthor() {
        let wl = Whitelist::default();
        let msg = "fix\n\nCo-Authored-By: Alice <alice@example.com>\n";
        let (out, changed) = strip_message(msg, &wl);
        assert!(!changed);
        assert!(out.contains("Alice"));
    }

    #[test]
    fn whitelist_protects_ai_identity() {
        let mut wl = Whitelist::default();
        wl.add("noreply@anthropic.com");
        let msg = "fix\n\nCo-Authored-By: Claude <noreply@anthropic.com>\n";
        let (out, changed) = strip_message(msg, &wl);
        assert!(!changed);
        assert!(out.contains("Co-Authored-By"));
    }

    #[test]
    fn whitelist_does_not_protect_footer() {
        let mut wl = Whitelist::default();
        wl.add("Claude");
        let msg = "fix\n\nGenerated with Claude Code\n";
        let (out, changed) = strip_message(msg, &wl);
        assert!(changed);
        assert!(!out.to_ascii_lowercase().contains("generated with"));
    }
}
