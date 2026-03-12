# Tech Stack Snapshot

Detected from Cargo.toml (Rust project):

- Language: Rust (edition 2021)
- TUI: ratatui + crossterm
- Config: serde + toml + toml_edit
- CLI parsing: clap
- Error handling: anyhow
- Process exec: std::os::unix::process::CommandExt
- Testing: cargo test (unit + integration), bats-core (shell tests for install.sh)
