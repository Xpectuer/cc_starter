use std::io::Write;
use std::process::Command;

macro_rules! require_live {
    () => {
        if std::env::var("CCT_LIVE_TESTS").is_err() {
            eprintln!("Skipped: set CCT_LIVE_TESTS=1 to run");
            return;
        }
    };
}

fn helpers_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/helpers")
}

fn prepend_path(dir: &std::path::Path) -> String {
    let orig = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", dir.display(), orig)
}

// --- Test 1: release binary builds ---

#[test]
fn release_binary_builds() {
    require_live!();

    let status = Command::new("cargo")
        .args(["build", "--release"])
        .status()
        .expect("cargo build --release");
    assert!(status.success(), "release build failed");

    let bin = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/release/cct");
    assert!(bin.exists(), "target/release/cct not found");
}

// --- Test 2: real config loads ---

#[test]
fn real_config_loads() {
    require_live!();

    let config_path = cct::config::config_path();
    if !config_path.exists() {
        eprintln!(
            "Skipped: real config not found at {}",
            config_path.display()
        );
        return;
    }
    let profiles = cct::config::load_profiles().expect("load real profiles");
    assert!(
        !profiles.is_empty(),
        "expected at least 1 profile in real config"
    );
}

// --- Test 3: binary spawns cleanly ---

#[test]
fn binary_spawns_cleanly() {
    require_live!();

    let bin = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/release/cct");
    if !bin.exists() {
        eprintln!("Skipped: target/release/cct not found (run release_binary_builds first)");
        return;
    }

    let mut child = Command::new(&bin)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn cct");

    // Send 'q' to quit the TUI
    if let Some(ref mut stdin) = child.stdin {
        let _ = stdin.write_all(b"q");
        let _ = stdin.flush();
    }

    let output = child.wait_with_output().expect("wait for cct");
    assert!(
        output.status.success(),
        "cct exited with error: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// --- Test 4: arg passthrough via fake binary ---

#[test]
fn arg_passthrough_via_fake() {
    require_live!();

    let toml = r#"
[[profiles]]
name = "live-fake-test"
model = "live-model"
skip_permissions = true
extra_args = ["--verbose"]
"#;
    let mut toml_file = tempfile::NamedTempFile::new().expect("create temp toml");
    toml_file
        .write_all(toml.as_bytes())
        .expect("write temp toml");
    toml_file.flush().expect("flush temp toml");

    let args_file = tempfile::NamedTempFile::new().expect("create args temp");
    let args_path = args_file.path().to_str().unwrap().to_string();

    let output = Command::new("cargo")
        .args(["run", "--example", "exec_profile", "--quiet"])
        .env("CCT_TEST_TOML", toml_file.path())
        .env("CCT_TEST_ARGS_FILE", &args_path)
        .env("PATH", prepend_path(&helpers_dir()))
        .output()
        .expect("spawn exec_profile");

    assert!(
        output.status.success(),
        "exec_profile failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let captured = std::fs::read_to_string(&args_path).expect("read args file");
    assert_eq!(
        captured,
        "--model live-model --dangerously-skip-permissions --verbose"
    );
}
