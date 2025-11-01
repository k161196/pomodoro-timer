use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::state::SessionInfo;

pub struct Persistence;

impl Persistence {
    pub fn data_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        Ok(PathBuf::from(home).join(".local/share/pomodoro-timer"))
    }

    pub fn state_path() -> Result<PathBuf> {
        Ok(Self::data_dir()?.join("state.json"))
    }

    pub fn load() -> Result<SessionInfo> {
        let state_path = Self::state_path()?;

        if !state_path.exists() {
            return Ok(SessionInfo::new());
        }

        let content = fs::read_to_string(&state_path)
            .context("Failed to read state file")?;

        let session_info: SessionInfo = serde_json::from_str(&content)
            .context("Failed to parse state file")?;

        Ok(session_info)
    }

    pub fn save(session_info: &SessionInfo) -> Result<()> {
        let data_dir = Self::data_dir()?;
        fs::create_dir_all(&data_dir)
            .context("Failed to create data directory")?;

        let state_path = Self::state_path()?;
        let content = serde_json::to_string_pretty(session_info)
            .context("Failed to serialize state")?;

        fs::write(&state_path, content)
            .context("Failed to write state file")?;

        Ok(())
    }

    pub fn clear() -> Result<()> {
        let state_path = Self::state_path()?;
        if state_path.exists() {
            fs::remove_file(&state_path)
                .context("Failed to remove state file")?;
        }
        Ok(())
    }
}
