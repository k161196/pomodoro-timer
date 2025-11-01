use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Work session duration in minutes
    pub work_duration: u32,

    /// Short break duration in minutes
    pub short_break_duration: u32,

    /// Long break duration in minutes
    pub long_break_duration: u32,

    /// Number of work sessions before a long break
    pub sessions_until_long_break: u32,

    /// Enable desktop notifications
    pub enable_notifications: bool,

    /// Auto-start breaks after work completes
    pub auto_start_breaks: bool,

    /// Auto-start work after breaks complete
    pub auto_start_work: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            work_duration: 25,
            short_break_duration: 5,
            long_break_duration: 15,
            sessions_until_long_break: 4,
            enable_notifications: true,
            auto_start_breaks: false,
            auto_start_work: false,
        }
    }
}

impl Config {
    pub fn config_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        Ok(PathBuf::from(home).join(".config/pomodoro-timer"))
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            // Create default config
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(&config_path)
            .context("Failed to read config file")?;

        let config: Config = toml::from_str(&content)
            .context("Failed to parse config file")?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = Self::config_dir()?;
        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;

        let config_path = Self::config_path()?;
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(&config_path, content)
            .context("Failed to write config file")?;

        Ok(())
    }

    pub fn work_duration_secs(&self) -> u32 {
        self.work_duration * 60
    }

    pub fn short_break_duration_secs(&self) -> u32 {
        self.short_break_duration * 60
    }

    pub fn long_break_duration_secs(&self) -> u32 {
        self.long_break_duration * 60
    }

    pub fn validate(&self) -> Result<()> {
        if self.work_duration == 0 {
            anyhow::bail!("Work duration must be greater than 0");
        }
        if self.short_break_duration == 0 {
            anyhow::bail!("Short break duration must be greater than 0");
        }
        if self.long_break_duration == 0 {
            anyhow::bail!("Long break duration must be greater than 0");
        }
        if self.sessions_until_long_break == 0 {
            anyhow::bail!("Sessions until long break must be greater than 0");
        }
        Ok(())
    }
}
