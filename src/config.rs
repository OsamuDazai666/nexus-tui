//! Config loader — reads ~/.config/nexus/config.toml
//! Falls back gracefully to env vars and defaults.

use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub tmdb_api_key: Option<String>,

    #[serde(default)]
    pub player: PlayerConfig,

    #[serde(default)]
    pub ui: UiConfig,
}

#[derive(Debug, Deserialize)]
pub struct PlayerConfig {
    /// Path to mpv binary (defaults to "mpv" on PATH)
    pub mpv_path: String,
    /// Extra mpv flags
    pub extra_args: Vec<String>,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            mpv_path: "mpv".to_string(),
            extra_args: vec!["--no-terminal".to_string()],
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UiConfig {
    /// Override image protocol: "kitty" | "sixel" | "halfblock" | "auto"
    pub image_protocol: String,
    /// Number of results to show
    pub results_limit: usize,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            image_protocol: "auto".to_string(),
            results_limit: 25,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        match Self::try_load() {
            Ok(cfg) => cfg,
            Err(_) => Self::default(),
        }
    }

    fn try_load() -> Result<Self> {
        let path = config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)?;
        let mut cfg: Config = toml::from_str(&content)?;

        // Env var takes precedence over config file
        if let Ok(key) = std::env::var("TMDB_API_KEY") {
            cfg.tmdb_api_key = Some(key);
        }

        Ok(cfg)
    }

    /// Write a sample config if none exists
    pub fn write_sample() -> Result<()> {
        let path = config_path();
        if path.exists() {
            return Ok(());
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(
            &path,
            r#"# nexus-tui configuration
# Full docs: https://github.com/you/nexus-tui

# TMDB API key (free at https://www.themoviedb.org/settings/api)
# tmdb_api_key = "your_key_here"

[player]
mpv_path   = "mpv"
extra_args = ["--no-terminal", "--really-quiet"]

[ui]
# Image rendering: "auto" | "kitty" | "sixel" | "halfblock"
image_protocol = "auto"
results_limit  = 25
"#,
        )?;
        Ok(())
    }
}

fn config_path() -> PathBuf {
    directories::ProjectDirs::from("dev", "nexus", "nexus-tui")
        .map(|d| d.config_dir().join("config.toml"))
        .unwrap_or_else(|| PathBuf::from(".nexus/config.toml"))
}
