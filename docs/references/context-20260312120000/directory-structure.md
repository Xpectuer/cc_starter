# Directory Structure Snapshot

```
cc_starter/
├── src/
│   ├── main.rs       # entry point, TUI event loop, key bindings
│   ├── app.rs        # App state, AppMode, FormState
│   ├── ui.rs         # ratatui draw, footer, detail panel
│   ├── launch.rs     # build_args, exec_claude, check_claude_installed
│   ├── config.rs     # profile TOML, toggle_skip_permissions, append_profile
│   └── cli.rs        # cct add interactive flow
├── tests/
│   ├── integration.rs
│   ├── live.rs
│   └── install.bats
├── docs/
│   ├── drafts/       # intake sessions
│   ├── procs/        # TDD/progress tracking
│   ├── modules/      # module docs
│   ├── references/   # reference + context snapshots
│   ├── lessons/      # lessons learned
│   └── dashboard.md
├── CLAUDE.md
├── ARCHITECTURE.md
├── Cargo.toml
└── install.sh
```
