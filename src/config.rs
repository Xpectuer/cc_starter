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
            fs::create_dir_all(parent).with_context(|| format!("create config dir {parent:?}"))?;
        }
        fs::write(&path, DEFAULT_CONFIG)
            .with_context(|| format!("write default config to {path:?}"))?;
    }
    Ok(())
}

pub fn load_profiles() -> Result<Vec<Profile>> {
    let path = config_path();
    let content = fs::read_to_string(&path).with_context(|| format!("read config {path:?}"))?;
    let config: Config =
        toml::from_str(&content).with_context(|| format!("parse TOML in {path:?}"))?;
    Ok(config.profiles)
}

pub struct NewProfile {
    pub name: String,
    pub description: Option<String>,
    pub model: Option<String>,
}

pub fn profile_name_exists(name: &str) -> Result<bool> {
    let profiles = load_profiles()?;
    Ok(profiles
        .iter()
        .any(|p| p.name.eq_ignore_ascii_case(name)))
}

pub fn append_profile(profile: &NewProfile) -> Result<()> {
    let path = config_path();
    let mut block = String::from("\n[[profiles]]\n");
    block.push_str(&format!("name = {:?}\n", profile.name));
    if let Some(desc) = &profile.description {
        if !desc.is_empty() {
            block.push_str(&format!("description = {:?}\n", desc));
        }
    }
    if let Some(model) = &profile.model {
        if !model.is_empty() {
            block.push_str(&format!("model = {:?}\n", model));
        }
    }
    let mut content = fs::read_to_string(&path)
        .with_context(|| format!("read config {path:?}"))?;
    content.push_str(&block);
    fs::write(&path, content)
        .with_context(|| format!("write config {path:?}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

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
        assert_eq!(
            p.extra_args.as_deref(),
            Some(&["--verbose".to_string()][..])
        );
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

    #[test]
    #[serial]
    fn append_profile_roundtrips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        std::fs::write(&path, DEFAULT_CONFIG).unwrap();
        std::env::set_var("CCT_CONFIG", &path);

        let new = NewProfile {
            name: "test-profile".into(),
            description: Some("A test".into()),
            model: Some("claude-sonnet-4-6".into()),
        };
        append_profile(&new).unwrap();
        let profiles = load_profiles().unwrap();
        assert!(profiles.iter().any(|p| p.name == "test-profile"));
        assert_eq!(profiles.len(), 2); // default + new

        std::env::remove_var("CCT_CONFIG");
    }

    #[test]
    #[serial]
    fn append_preserves_existing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        let original = "# My comment\n\n[[profiles]]\nname = \"orig\"\n";
        std::fs::write(&path, original).unwrap();
        std::env::set_var("CCT_CONFIG", &path);

        let new = NewProfile {
            name: "added".into(),
            description: None,
            model: None,
        };
        append_profile(&new).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("# My comment"));
        let profiles = load_profiles().unwrap();
        assert_eq!(profiles.len(), 2);

        std::env::remove_var("CCT_CONFIG");
    }

    #[test]
    #[serial]
    fn profile_name_exists_case_insensitive() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        std::fs::write(&path, "[[profiles]]\nname = \"MyProfile\"\n").unwrap();
        std::env::set_var("CCT_CONFIG", &path);

        assert!(profile_name_exists("myprofile").unwrap());
        assert!(profile_name_exists("MYPROFILE").unwrap());
        assert!(!profile_name_exists("other").unwrap());

        std::env::remove_var("CCT_CONFIG");
    }

    #[test]
    #[serial]
    fn append_minimal_profile() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        std::fs::write(&path, DEFAULT_CONFIG).unwrap();
        std::env::set_var("CCT_CONFIG", &path);

        let new = NewProfile {
            name: "minimal".into(),
            description: None,
            model: None,
        };
        append_profile(&new).unwrap();
        let profiles = load_profiles().unwrap();
        let p = profiles.iter().find(|p| p.name == "minimal").unwrap();
        assert!(p.description.is_none());
        assert!(p.model.is_none());

        std::env::remove_var("CCT_CONFIG");
    }
}
