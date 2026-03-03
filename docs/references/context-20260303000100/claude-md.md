# CLAUDE.md Snapshot

- **Project**: `cct` — terminal UI launcher for Claude Code
- **Config**: TOML file at `~/.config/cc-tui/profiles.toml`
- **Modules**: config (deserialize TOML), app (cursor state), ui (ratatui rendering), launch (exec-replace)
- **Data flow**: main loads profiles → App → draw loop → Enter → exec_claude
- **Key design**: exec (not spawn), env var masking, config hot-reload on `e` key
