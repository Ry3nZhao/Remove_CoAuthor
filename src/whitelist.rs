use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};

use crate::config::{self, ConfigError};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Whitelist {
    pub identities: Vec<WhitelistEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistEntry {
    pub value: String,
    pub added_at: String,
}

#[derive(Debug, thiserror::Error)]
pub enum WhitelistError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("identity not found in whitelist: {0}")]
    NotFound(String),
}

impl Whitelist {
    pub fn load() -> Result<Self, WhitelistError> {
        let path = config::whitelist_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = fs::read_to_string(&path)?;
        if data.trim().is_empty() {
            return Ok(Self::default());
        }
        Ok(serde_json::from_str(&data)?)
    }

    pub fn save(&self) -> Result<(), WhitelistError> {
        let path = config::whitelist_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data + "\n")?;
        Ok(())
    }

    pub fn add(&mut self, value: &str) -> bool {
        let value = value.trim();
        if value.is_empty() {
            return false;
        }
        let lower = value.to_ascii_lowercase();
        if self
            .identities
            .iter()
            .any(|e| e.value.to_ascii_lowercase() == lower)
        {
            return false;
        }
        self.identities.push(WhitelistEntry {
            value: value.to_string(),
            added_at: Utc::now().to_rfc3339(),
        });
        true
    }

    pub fn remove(&mut self, value: &str) -> Result<(), WhitelistError> {
        let lower = value.trim().to_ascii_lowercase();
        let before = self.identities.len();
        self.identities
            .retain(|e| e.value.to_ascii_lowercase() != lower);
        if self.identities.len() == before {
            return Err(WhitelistError::NotFound(value.trim().to_string()));
        }
        Ok(())
    }

    /// Returns true if a trailer/line containing this identity should be kept.
    pub fn protects_line(&self, line: &str) -> bool {
        let line_lower = line.to_ascii_lowercase();
        for entry in &self.identities {
            let v = entry.value.trim();
            if v.is_empty() {
                continue;
            }
            let v_lower = v.to_ascii_lowercase();
            if line_lower.contains(&v_lower) {
                return true;
            }
            // Also match bare name if whitelist is "Name <email>"
            if let Some((name, email)) = parse_identity(v) {
                if !name.is_empty() && line_lower.contains(&name.to_ascii_lowercase()) {
                    return true;
                }
                if !email.is_empty() && line_lower.contains(&email.to_ascii_lowercase()) {
                    return true;
                }
            }
        }
        false
    }
}

fn parse_identity(value: &str) -> Option<(String, String)> {
    let value = value.trim();
    if let Some(start) = value.find('<') {
        if let Some(end) = value[start..].find('>') {
            let name = value[..start].trim().to_string();
            let email = value[start + 1..start + end].trim().to_string();
            return Some((name, email));
        }
    }
    if value.contains('@') {
        return Some((String::new(), value.to_string()));
    }
    Some((value.to_string(), String::new()))
}

pub fn cmd_add(value: &str) -> Result<(), WhitelistError> {
    let mut wl = Whitelist::load()?;
    if wl.add(value) {
        wl.save()?;
        println!("added: {}", value.trim());
    } else {
        println!("already present: {}", value.trim());
    }
    Ok(())
}

pub fn cmd_remove(value: &str) -> Result<(), WhitelistError> {
    let mut wl = Whitelist::load()?;
    wl.remove(value)?;
    wl.save()?;
    println!("removed: {}", value.trim());
    Ok(())
}

pub fn cmd_list(out: &mut dyn Write) -> Result<(), WhitelistError> {
    let wl = Whitelist::load()?;
    if wl.identities.is_empty() {
        writeln!(out, "(empty)")?;
        return Ok(());
    }
    for entry in &wl.identities {
        writeln!(out, "{}", entry.value)?;
    }
    Ok(())
}

pub fn load_or_default() -> Whitelist {
    Whitelist::load().unwrap_or_default()
}
