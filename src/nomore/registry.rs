use std::fs;
use std::path::Path;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::config;
use crate::nomore::NoMoreError;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Supervised {
    #[serde(default)]
    pub global: bool,
    #[serde(default)]
    pub repos: Vec<RepoEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoEntry {
    pub path: String,
    pub installed_at: String,
}

impl Supervised {
    pub fn load() -> Result<Self, NoMoreError> {
        let path = config::supervised_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = fs::read_to_string(&path)?;
        if data.trim().is_empty() {
            return Ok(Self::default());
        }
        Ok(serde_json::from_str(&data)?)
    }

    pub fn save(&self) -> Result<(), NoMoreError> {
        let path = config::supervised_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data + "\n")?;
        Ok(())
    }

    pub fn upsert_repo(&mut self, repo: &Path) -> Result<(), NoMoreError> {
        let abs = repo
            .canonicalize()
            .unwrap_or_else(|_| repo.to_path_buf())
            .display()
            .to_string();
        if let Some(existing) = self.repos.iter_mut().find(|r| r.path == abs) {
            existing.installed_at = Utc::now().to_rfc3339();
        } else {
            self.repos.push(RepoEntry {
                path: abs,
                installed_at: Utc::now().to_rfc3339(),
            });
        }
        Ok(())
    }

    pub fn remove_repo(&mut self, repo: &Path) {
        let abs = repo
            .canonicalize()
            .unwrap_or_else(|_| repo.to_path_buf())
            .display()
            .to_string();
        self.repos.retain(|r| r.path != abs);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalState {
    pub previous_hooks_path: Option<String>,
}

impl GlobalState {
    pub fn load() -> Result<Self, NoMoreError> {
        let path = config::global_state_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = fs::read_to_string(&path)?;
        if data.trim().is_empty() {
            return Ok(Self::default());
        }
        Ok(serde_json::from_str(&data)?)
    }

    pub fn save(&self) -> Result<(), NoMoreError> {
        let path = config::global_state_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data + "\n")?;
        Ok(())
    }
}
