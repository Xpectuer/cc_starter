---
doc_type: module
generated_by: mci-phase-3
created: 2026-03-03
updated: 2026-03-03
revision: 2
---

# cct — Module Documentation Index

## Project Summary

One-line summary: `cct` is a Rust terminal UI launcher that reads Claude Code profiles from a TOML config file and exec-replaces itself with `claude <args>` when the user selects a profile.

## Key Statistics

- First-level modules: **5** (config, app, ui, launch, cli)
- Total public interface points: **24** (7 types/structs/enums + 17 functions/methods)
- Key external dependencies: `ratatui`, `crossterm`, `serde`, `toml`, `dirs`, `anyhow`

## System Architecture Overview

Five-module flat architecture with unidirectional data flow and no shared mutable state:

```
config (leaf) ─→ app ─→ ui
                    └─→ launch
cli ─────────────→ config
```

`config` is the leaf (no internal deps). `app`, `ui`, `launch`, and `cli` each depend on `config`. `ui` additionally depends on `app`. There are no circular dependencies.

---

<!-- BEGIN:module-index -->
## Module Index

| Module | Doc Path | Primary Responsibility | Depends On |
|--------|----------|----------------------|------------|
| `config` | [docs/modules/config.md](config.md) | TOML deserialization, default config bootstrap, config path resolution, profile append with env-var generation | *(leaf — no internal deps)* |
| `app` | [docs/modules/app.md](app.md) | Cursor state (`selected`), circular navigation, `AppMode` (Normal/AddForm), 5-field `FormState` | `config::Profile` |
| `ui` | [docs/modules/ui.md](ui.md) | ratatui rendering: 35/65 split list+detail panel + footer; inline add-form; sensitive-value masking | `app::App`, `app::AppMode`, `app::FormState`, `app::FIELD_LABELS`, `config::Profile` |
| `launch` | [docs/modules/launch.md](launch.md) | Build `claude` CLI args; Unix exec-replace; open `$EDITOR`; restore terminal | `config::Profile` |
| `cli` | *(inline in src/cli.rs)* | `cct add` interactive CLI flow: 5 prompts, masked API key summary, duplicate guard | `config::NewProfile`, `config::profile_name_exists`, `config::append_profile` |
<!-- END:module-index -->

---

<!-- BEGIN:dependency-graph -->
## Dependency Graph

```mermaid
graph TD
    main["main.rs (binary entry)"]
    config["config\n(src/config.rs)"]
    app["app\n(src/app.rs)"]
    ui["ui\n(src/ui.rs)"]
    launch["launch\n(src/launch.rs)"]
    cli["cli\n(src/cli.rs)"]
    serde["serde + toml\n(external)"]
    ratatui["ratatui + crossterm\n(external)"]
    dirs["dirs\n(external)"]
    anyhow["anyhow\n(external)"]
    claude["claude binary\n(external process)"]

    main --> config
    main --> app
    main --> ui
    main --> launch
    main --> cli
    app --> config
    ui --> app
    ui --> config
    ui --> ratatui
    launch --> config
    launch --> ratatui
    cli --> config
    config --> serde
    config --> dirs
    config --> anyhow
    launch --> anyhow
    launch --> claude
```

**Notes**:
- `config` is the only leaf module; it has no internal imports.
- `ui`, `launch`, and `cli` all consume `config` exports but are independent of each other.
- `main.rs` is the only orchestrator; no module calls a sibling module (except `ui` → `app`).
- No circular dependencies exist.
<!-- END:dependency-graph -->

---

<!-- BEGIN:interface-index -->
## Global Interface Index

### config module (`src/config.rs`)

- `struct Profile` — deserialized profile (name, description, model, skip_permissions, extra_args, env)
- `struct NewProfile` — input for profile creation (name, description, base_url, api_key, model)
- `fn config_path() -> PathBuf` — resolves config file path (`CCT_CONFIG` env var → XDG dirs)
- `fn ensure_default_config() -> Result<()>` — creates default TOML on first run (idempotent)
- `fn load_profiles() -> Result<Vec<Profile>>` — reads and parses the TOML file
- `fn profile_name_exists(name: &str) -> Result<bool>` — case-insensitive duplicate check
- `fn append_profile(profile: &NewProfile) -> Result<()>` — appends `[[profiles]]` + optional `[profiles.env]` block

### app module (`src/app.rs`)

- `const FIELD_LABELS: [&str; 5]` — ordered field labels for the add form: `["Name *", "Description", "Base URL", "API Key", "Model"]`
- `enum AppMode` — `Normal` | `AddForm(FormState)` — discriminates TUI modes
- `struct FormState { fields: [String; 5], active_field: usize, confirming: bool, error: Option<String> }` — add-form transient state
- `fn FormState::new() -> Self` — construct empty form
- `fn FormState::next_field(&mut self)` — advance field cursor (clamped at 4)
- `fn FormState::prev_field(&mut self)` — retreat field cursor (clamped at 0)
- `struct App { profiles: Vec<Profile>, selected: usize, mode: AppMode }` — sole mutable TUI state owner
- `fn App::new(profiles: Vec<Profile>) -> Self` — constructs with `selected = 0`, `mode = Normal`
- `fn App::next(&mut self)` — advance profile cursor (wraps, no-op if empty)
- `fn App::prev(&mut self)` — retreat profile cursor (wraps, no-op if empty)

### ui module (`src/ui.rs`)

- `fn mask_value<'a>(key: &str, val: &'a str) -> &'a str` — returns `"***"` for TOKEN/KEY/SECRET keys
- `fn draw(app: &App, frame: &mut Frame)` — full TUI render (list + detail/form + footer); dispatches on `app.mode`

### launch module (`src/launch.rs`)

- `fn restore_terminal()` — disable raw mode, leave alternate screen (errors suppressed)
- `fn build_args(profile: &Profile) -> Vec<String>` — pure arg builder (model → skip-perms → extra)
- `fn exec_claude(profile: &Profile) -> anyhow::Error` — injects env vars, exec-replaces process
- `fn open_editor(path: &Path) -> Result<()>` — spawns `$EDITOR` (fallback: `vi`), blocks until exit

### cli module (`src/cli.rs`)

- `fn run_add() -> Result<()>` — entry point for `cct add`; delegates to `run_add_with(stdin, stdout)`
- `fn run_add_with<R: BufRead, W: Write>(reader, writer) -> Result<()>` — testable 5-prompt interactive flow; calls `config::append_profile` on confirmation
<!-- END:interface-index -->

---

## Cross-Reference Consistency Check

| Claim | Verified |
|-------|----------|
| `app` depends only on `config::Profile` | ✅ — only `use crate::config::Profile` in source |
| `ui` depends on `app::{App, AppMode, FormState, FIELD_LABELS}` and `config::Profile` | ✅ — verified in `src/ui.rs` use statement |
| `launch` depends only on `config::Profile` | ✅ — only `use crate::config::Profile` |
| `cli` depends on `config::{self, NewProfile}` | ✅ — verified in `src/cli.rs` use statement |
| No circular dependencies | ✅ — `config` is a pure leaf, others are consumers |
| No orphan modules | ✅ — all 5 modules are referenced from `src/lib.rs` and used by `main.rs` |
| `FIELD_LABELS` order matches `FormState.fields` index contract | ✅ — both use index 0=Name, 1=Description, 2=Base URL, 3=API Key, 4=Model |
