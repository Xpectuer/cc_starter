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
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
}

pub fn profile_name_exists(name: &str) -> Result<bool> {
    let profiles = load_profiles()?;
    Ok(profiles
        .iter()
        .any(|p| p.name.eq_ignore_ascii_case(name)))
}

/// Returns the non-empty string value if present, or None.
fn non_empty(opt: &Option<String>) -> Option<&str> {
    opt.as_deref().filter(|s| !s.is_empty())
}

pub fn append_profile(profile: &NewProfile) -> Result<()> {
    let path = config_path();
    let mut block = String::from("\n[[profiles]]\n");
    block.push_str(&format!("name = {:?}\n", profile.name));
    if let Some(desc) = non_empty(&profile.description) {
        block.push_str(&format!("description = {:?}\n", desc));
    }
    if let Some(model) = non_empty(&profile.model) {
        block.push_str(&format!("model = {:?}\n", model));
    }

    // Build [profiles.env] section when base_url, api_key, or model are provided
    let base_url = non_empty(&profile.base_url);
    let api_key = non_empty(&profile.api_key);
    let model = non_empty(&profile.model);

    if base_url.is_some() || api_key.is_some() || model.is_some() {
        block.push_str("\n[profiles.env]\n");
        if let Some(url) = base_url {
            block.push_str(&format!("ANTHROPIC_BASE_URL = {:?}\n", url));
        }
        if let Some(key) = api_key {
            block.push_str(&format!("ANTHROPIC_API_KEY = {:?}\n", key));
        }
        if let Some(m) = model {
            block.push_str(&format!("ANTHROPIC_MODEL = {:?}\n", m));
            block.push_str(&format!("ANTHROPIC_SMALL_FAST_MODEL = {:?}\n", m));
            block.push_str(&format!("ANTHROPIC_DEFAULT_SONNET_MODEL = {:?}\n", m));
            block.push_str(&format!("ANTHROPIC_DEFAULT_OPUS_MODEL = {:?}\n", m));
            block.push_str(&format!("ANTHROPIC_DEFAULT_HAIKU_MODEL = {:?}\n", m));
            block.push_str("API_TIMEOUT_MS = \"600000\"\n");
            block.push_str("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = \"1\"\n");
        }
    }

    let mut content =
        fs::read_to_string(&path).with_context(|| format!("read config {path:?}"))?;
    content.push_str(&block);
    fs::write(&path, content).with_context(|| format!("write config {path:?}"))?;
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
            base_url: None,
            api_key: None,
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
            base_url: None,
            api_key: None,
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
    fn append_profile_generates_env_section() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        std::fs::write(&path, DEFAULT_CONFIG).unwrap();
        std::env::set_var("CCT_CONFIG", &path);

        let new = NewProfile {
            name: "env-test".into(),
            description: Some("Test env generation".into()),
            base_url: Some("https://api.example.com".into()),
            api_key: Some("sk-test-key-123".into()),
            model: Some("kimi-k2".into()),
        };
        append_profile(&new).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        // Must contain [profiles.env] section
        assert!(
            content.contains("ANTHROPIC_BASE_URL"),
            "Expected ANTHROPIC_BASE_URL in output, got:\n{content}"
        );
        assert!(
            content.contains("https://api.example.com"),
            "Expected base_url value in output"
        );
        assert!(
            content.contains("ANTHROPIC_API_KEY"),
            "Expected ANTHROPIC_API_KEY in output"
        );
        assert!(
            content.contains("sk-test-key-123"),
            "Expected api_key value in output"
        );
        assert!(
            content.contains("ANTHROPIC_MODEL"),
            "Expected ANTHROPIC_MODEL in output"
        );
        assert!(
            content.contains("ANTHROPIC_SMALL_FAST_MODEL"),
            "Expected ANTHROPIC_SMALL_FAST_MODEL in output"
        );
        assert!(
            content.contains("ANTHROPIC_DEFAULT_SONNET_MODEL"),
            "Expected ANTHROPIC_DEFAULT_SONNET_MODEL in output"
        );
        assert!(
            content.contains("ANTHROPIC_DEFAULT_OPUS_MODEL"),
            "Expected ANTHROPIC_DEFAULT_OPUS_MODEL in output"
        );
        assert!(
            content.contains("ANTHROPIC_DEFAULT_HAIKU_MODEL"),
            "Expected ANTHROPIC_DEFAULT_HAIKU_MODEL in output"
        );
        assert!(
            content.contains("API_TIMEOUT_MS"),
            "Expected API_TIMEOUT_MS in output"
        );
        assert!(
            content.contains("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC"),
            "Expected CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC in output"
        );

        // Verify the profile round-trips through TOML parsing
        let profiles = load_profiles().unwrap();
        let p = profiles.iter().find(|p| p.name == "env-test").unwrap();
        let env = p.env.as_ref().expect("env section should exist");
        assert_eq!(
            env.get("ANTHROPIC_BASE_URL").map(String::as_str),
            Some("https://api.example.com")
        );
        assert_eq!(
            env.get("ANTHROPIC_API_KEY").map(String::as_str),
            Some("sk-test-key-123")
        );
        assert_eq!(
            env.get("ANTHROPIC_MODEL").map(String::as_str),
            Some("kimi-k2")
        );

        std::env::remove_var("CCT_CONFIG");
    }

    #[test]
    #[serial]
    fn append_profile_base_url_only() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        std::fs::write(&path, DEFAULT_CONFIG).unwrap();
        std::env::set_var("CCT_CONFIG", &path);

        let new = NewProfile {
            name: "base-url-only".into(),
            description: Some("Only base URL".into()),
            base_url: Some("https://api.third-party.com/v1".into()),
            api_key: None,
            model: None,
        };
        append_profile(&new).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        // Must contain [profiles.env] with ANTHROPIC_BASE_URL
        assert!(
            content.contains("[profiles.env]"),
            "Expected [profiles.env] section in output, got:\n{content}"
        );
        assert!(
            content.contains("ANTHROPIC_BASE_URL"),
            "Expected ANTHROPIC_BASE_URL in output"
        );
        assert!(
            content.contains("https://api.third-party.com/v1"),
            "Expected base_url value in output"
        );
        // Must NOT contain model-derived env vars
        assert!(
            !content.contains("ANTHROPIC_MODEL"),
            "ANTHROPIC_MODEL should NOT be present when model is None"
        );
        assert!(
            !content.contains("API_TIMEOUT_MS"),
            "API_TIMEOUT_MS should NOT be present when model is None"
        );

        // Round-trip verification
        let profiles = load_profiles().unwrap();
        let p = profiles
            .iter()
            .find(|p| p.name == "base-url-only")
            .unwrap();
        let env = p.env.as_ref().expect("env section should exist");
        assert_eq!(
            env.get("ANTHROPIC_BASE_URL").map(String::as_str),
            Some("https://api.third-party.com/v1")
        );
        assert!(
            env.get("ANTHROPIC_MODEL").is_none(),
            "ANTHROPIC_MODEL should not exist in env"
        );

        std::env::remove_var("CCT_CONFIG");
    }

    #[test]
    #[serial]
    fn append_minimal_no_env_section() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        std::fs::write(&path, DEFAULT_CONFIG).unwrap();
        std::env::set_var("CCT_CONFIG", &path);

        let new = NewProfile {
            name: "no-env".into(),
            description: Some("No env vars at all".into()),
            base_url: None,
            api_key: None,
            model: None,
        };
        append_profile(&new).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        // The appended block should NOT contain [profiles.env]
        // Find the appended block by locating name = "no-env"
        let block_start = content.find("name = \"no-env\"").expect("profile should exist");
        let appended_block = &content[block_start..];
        assert!(
            !appended_block.contains("[profiles.env]"),
            "Expected NO [profiles.env] section for minimal profile, got:\n{appended_block}"
        );
        assert!(
            !appended_block.contains("ANTHROPIC_BASE_URL"),
            "Expected NO ANTHROPIC_BASE_URL for minimal profile"
        );
        assert!(
            !appended_block.contains("ANTHROPIC_API_KEY"),
            "Expected NO ANTHROPIC_API_KEY for minimal profile"
        );
        assert!(
            !appended_block.contains("ANTHROPIC_MODEL"),
            "Expected NO ANTHROPIC_MODEL for minimal profile"
        );

        // Round-trip: env should be None
        let profiles = load_profiles().unwrap();
        let p = profiles.iter().find(|p| p.name == "no-env").unwrap();
        assert!(
            p.env.is_none(),
            "env section should be None for minimal profile"
        );

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
            base_url: None,
            api_key: None,
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
