---
title: "Spec: Config Add Functionality for cct"
doc_type: proc
brief: "Design spec for adding profiles via CLI subcommand and in-TUI inline form"
confidence: speculative
created: 2026-03-03
updated: 2026-03-03
revision: 2
source_skill: idea
---

# Spec: Config Add Functionality for cct

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| TUI form implementation | Hand-rolled text input | Zero new UI deps; 5 fields; consistent with existing codebase (no external widgets) |
| TOML writing strategy | String append | Preserves user comments and formatting; simpler than full serde round-trip |
| CLI arg parser | clap (existing dependency) | Industry-standard; handles subcommands/help elegantly; `Option<Commands>` pattern for default TUI |
| CLI add UX | Interactive prompts | Lower friction than flag-based; matches the "easy-to-use" goal |
| Validation | Duplicate name check only | Keeps it simple; no model/URL validation needed |
| Post-add behavior | Auto-select new profile | Immediate usability after adding |
| Env var auto-population | Model-triggered defaults | When model is provided, auto-populate 5 model env vars + 2 static defaults; reduces manual toml editing for third-party providers |
| API key masking | Reuse existing `mask_value` | `ui.rs` already masks keys containing TOKEN/KEY/SECRET; API key env var will be masked automatically |

## Architecture

Two entry points converge on shared config-writing logic:

```
cct (no args) ──► TUI ──► press 'a' ──► AddForm mode ──► append_profile()
cct add       ──► interactive CLI prompts ──────────────► append_profile()
```

### Module Changes (revision 2 — delta from existing implementation)

| Module | Change |
|--------|--------|
| `config.rs` | Extend `NewProfile` with `base_url`, `api_key`; update `append_profile()` to write `[profiles.env]` |
| `app.rs` | Expand `FormState` from 3 to 5 fields; update `FIELD_LABELS`; adjust navigation bounds |
| `cli.rs` | Add base_url and api_key prompts; update summary display |
| `ui.rs` | Render 5 form fields; update confirmation summary to show base_url/api_key |
| `main.rs` | Pass 5 fields to `NewProfile`; update TUI AddForm → save logic |
| `launch.rs` | No changes |

## Data Structures

### `NewProfile` (config.rs)

```rust
pub struct NewProfile {
    pub name: String,
    pub description: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
}
```

### `AppMode` / `FormState` (app.rs)

```rust
pub enum AppMode {
    Normal,
    AddForm(FormState),
}

pub struct FormState {
    pub fields: [String; 5],    // [name, description, base_url, api_key, model]
    pub active_field: usize,    // 0..4
    pub confirming: bool,       // true = showing summary, awaiting y/n
    pub error: Option<String>,
}
```

### CLI struct (main.rs) — unchanged

```rust
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new profile interactively
    Add,
}
```

## Config Writing

`append_profile` formats a TOML string and appends to the config file. When env vars are needed, a `[profiles.env]` section is appended:

```toml

[[profiles]]
name = "minimax-provider"
description = "MiniMax API backend"

[profiles.env]
ANTHROPIC_BASE_URL = "https://api.minimax.chat/v1"
ANTHROPIC_API_KEY = "sk-xxx"
ANTHROPIC_MODEL = "MiniMax-M2.1"
ANTHROPIC_SMALL_FAST_MODEL = "MiniMax-M2.1"
ANTHROPIC_DEFAULT_SONNET_MODEL = "MiniMax-M2.1"
ANTHROPIC_DEFAULT_OPUS_MODEL = "MiniMax-M2.1"
ANTHROPIC_DEFAULT_HAIKU_MODEL = "MiniMax-M2.1"
API_TIMEOUT_MS = "600000"
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
```

### Env var generation rules

| Condition | Env vars written |
|-----------|-----------------|
| `base_url` provided | `ANTHROPIC_BASE_URL = "<base_url>"` |
| `api_key` provided | `ANTHROPIC_API_KEY = "<api_key>"` |
| `model` provided | `ANTHROPIC_MODEL`, `ANTHROPIC_SMALL_FAST_MODEL`, `ANTHROPIC_DEFAULT_SONNET_MODEL`, `ANTHROPIC_DEFAULT_OPUS_MODEL`, `ANTHROPIC_DEFAULT_HAIKU_MODEL` = `"<model>"`; plus `API_TIMEOUT_MS = "600000"`, `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"` |
| None provided | No `[profiles.env]` section written |

Only non-empty optional fields are included. Existing file content (including comments) is preserved. The `[profiles.env]` section is only written when at least one env var is needed.

## TUI Form Flow

1. Press `a` → mode switches to `AddForm(FormState::new())`
2. Key routing: printable chars append, Backspace deletes, Tab/Down next field, Shift-Tab/Up prev field
3. Enter on last field or Ctrl-S → `confirming = true`, UI shows summary
4. In confirmation: `y` → validate name → append → reload → auto-select → Normal mode
5. `n` or Esc → back to editing; Esc in editing → Normal (discard)
6. Field labels: `["Name *", "Description", "Base URL", "API Key", "Model"]`

## CLI Add Flow

1. `cct add` → `cli::run_add()`
2. Prompt: `Name:` → `Description (optional):` → `Base URL (optional):` → `API Key (optional):` → `Model (optional):`
3. Validate name non-empty (re-prompt if empty), check duplicate (exit 1 if exists)
4. Print summary (mask API key) → `Save? (y/n)` → `y` writes, `n` cancels

## Error Handling

- File I/O errors: `anyhow` context, propagate. TUI: show in detail panel, return to Normal. CLI: print and exit 1.
- Empty name: rejected at input level (form won't confirm, CLI re-prompts)
- Duplicate name: clear message, non-destructive

## Testing Strategy

**Unit tests (config.rs):**
- `append_profile` writes valid TOML that round-trips through `load_profiles` (with env vars)
- `append_profile` preserves existing profiles and comments
- `profile_name_exists` returns true/false correctly (case-insensitive)
- Minimal `NewProfile` (name only) produces valid block without `[profiles.env]`
- Full `NewProfile` (all fields) generates correct `[profiles.env]` with all expected env vars
- `append_profile` with only base_url writes only `ANTHROPIC_BASE_URL`

**Unit tests (app.rs):**
- `FormState::new()` initialization (5 empty fields)
- Field navigation wrapping (0..4)
- Mode transitions

**Integration tests:**
- `cct add` with piped stdin (5 fields) → verify profile appended to temp config with env vars
- All tests use `tempfile` + `CCT_CONFIG` env var for isolation

## Open Questions

None — all requirements are clear.
