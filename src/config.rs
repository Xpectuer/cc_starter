use anyhow::{Context, Result};
use serde::Deserialize;
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Debug, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    pub description: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub extra_args: Option<Vec<String>>,
    pub skip_permissions: Option<bool>,
    pub model: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Config {
    profiles: Vec<Profile>,
}

pub fn config_path() -> PathBuf {
    if let Ok(p) = std::env::var("CCT_CONFIG") {
        return PathBuf::from(p);
    }
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("cc-tui")
        .join("profiles.toml")
}

const DEFAULT_CONFIG: &str = r#"# cct — Claude Code TUI profile configuration
# Each [[profiles]] block defines one launch profile.

[[profiles]]
name = "default"
description = "Default Claude Code"
# model = "claude-sonnet-4-6"
# skip_permissions = false
# extra_args = []

# [profiles.env]
# ANTHROPIC_API_KEY = "sk-ant-..."
"#;

pub fn ensure_default_config() -> Result<()> {
    let path = config_path();
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create config dir {parent:?}"))?;
        }
        fs::write(&path, DEFAULT_CONFIG)
            .with_context(|| format!("write default config to {path:?}"))?;
    }
    Ok(())
}

pub fn load_profiles() -> Result<Vec<Profile>> {
    let path = config_path();
    let content = fs::read_to_string(&path)
        .with_context(|| format!("read config {path:?}"))?;
    let config: Config = toml::from_str(&content)
        .with_context(|| format!("parse TOML in {path:?}"))?;
    Ok(config.profiles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_profile() {
        let src = r#"
[[profiles]]
name = "kclaude"
description = "Kimi AI"
model = "kimi-k1.5"
skip_permissions = true
extra_args = ["--verbose"]

[profiles.env]
ANTHROPIC_BASE_URL = "https://api.example.com"
ANTHROPIC_AUTH_TOKEN = "sk-secret"
"#;
        let cfg: Config = toml::from_str(src).unwrap();
        assert_eq!(cfg.profiles.len(), 1);
        let p = &cfg.profiles[0];
        assert_eq!(p.name, "kclaude");
        assert_eq!(p.model.as_deref(), Some("kimi-k1.5"));
        assert_eq!(p.skip_permissions, Some(true));
        assert_eq!(p.extra_args.as_deref(), Some(&["--verbose".to_string()][..]));
        let env = p.env.as_ref().unwrap();
        assert_eq!(
            env.get("ANTHROPIC_BASE_URL").map(String::as_str),
            Some("https://api.example.com")
        );
    }

    #[test]
    fn parse_minimal_profile() {
        let src = "[[profiles]]\nname = \"default\"";
        let cfg: Config = toml::from_str(src).unwrap();
        assert_eq!(cfg.profiles[0].name, "default");
        assert!(cfg.profiles[0].description.is_none());
        assert!(cfg.profiles[0].env.is_none());
    }

    #[test]
    fn default_config_is_valid_toml() {
        let _: Config = toml::from_str(DEFAULT_CONFIG).unwrap();
    }
}
