use std::fs;
use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("cannot resolve home/config directory")]
    NoConfigDir,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn rmai_config_dir() -> Result<PathBuf, ConfigError> {
    let base = dirs::config_dir().ok_or(ConfigError::NoConfigDir)?;
    let dir = base.join("rmai");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn whitelist_path() -> Result<PathBuf, ConfigError> {
    Ok(rmai_config_dir()?.join("whitelist.json"))
}

pub fn supervised_path() -> Result<PathBuf, ConfigError> {
    Ok(rmai_config_dir()?.join("supervised.json"))
}

pub fn global_state_path() -> Result<PathBuf, ConfigError> {
    Ok(rmai_config_dir()?.join("global-state.json"))
}

pub fn global_hooks_dir() -> Result<PathBuf, ConfigError> {
    let dir = rmai_config_dir()?.join("global-hooks");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}
