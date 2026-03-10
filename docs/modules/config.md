---
doc_type: module
module_name: "config"
module_path: "src/config.rs"
generated_by: mci-phase-2
revision: 3
updated: 2026-03-10
---

# config Module Documentation

> **Purpose**: Deserializes `profiles.toml` into typed Rust structs via `serde`/`toml`, bootstraps a default config file on first run, and resolves the config path from the `CCT_CONFIG` environment variable or XDG config directories.
> **Path**: `src/config.rs`

---

<!-- BEGIN:interface -->
## 1. Interface

### Exported Types

- `struct Profile` — Represents one launch profile loaded from TOML. All fields except `name` are optional:
  - `name: String` — Unique display name shown in the TUI list.
  - `description: Option<String>` — Human-readable description shown in the detail panel.
  - `env: Option<HashMap<String, String>>` — Environment variables injected before exec.
  - `extra_args: Option<Vec<String>>` — Additional CLI arguments appended verbatim to `claude`.
  - `skip_permissions: Option<bool>` — When `true`, adds `--dangerously-skip-permissions` to the `claude` invocation.
  - `model: Option<String>` — When set, adds `--model <value>` to the `claude` invocation.
  - Derives: `Debug`, `Deserialize`, `Clone`.

- `struct NewProfile` — Input type for creating a new profile via `append_profile`. All fields except `name` are optional:
  - `name: String` — Required profile name (must be unique, case-insensitive).
  - `description: Option<String>` — Human-readable description.
  - `base_url: Option<String>` — When set, written as `ANTHROPIC_BASE_URL` in `[profiles.env]`.
  - `api_key: Option<String>` — When set, written as `ANTHROPIC_API_KEY` in `[profiles.env]`.
  - `model: Option<String>` — When set, written as `model = ...` in `[[profiles]]` and as 5 model alias env vars plus `API_TIMEOUT_MS` and `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` in `[profiles.env]`.

### Exported Functions

- `config_path() -> PathBuf`
  - Returns the resolved path to `profiles.toml`.
  - Priority: `CCT_CONFIG` env var (if set and non-empty) → `dirs::config_dir()/cc-tui/profiles.toml` → fallback `~/.config/cc-tui/profiles.toml` if `dirs` returns `None`.
  - No I/O performed; pure path resolution.

- `ensure_default_config() -> Result<()>`
  - Checks whether the file at `config_path()` exists.
  - If absent: creates all parent directories with `fs::create_dir_all`, then writes `DEFAULT_CONFIG` to the path.
  - If present: no-op (idempotent).
  - Returns `anyhow::Result<()>`; propagates errors with context messages.

- `load_profiles() -> Result<Vec<Profile>>`
  - Reads the file at `config_path()` to a `String`.
  - Parses the full TOML document into the private `Config` struct (which holds `profiles: Vec<Profile>`).
  - Returns the unwrapped `Vec<Profile>` to callers; the outer `Config` wrapper is not exposed.
  - Returns `anyhow::Result<Vec<Profile>>`; propagates I/O and parse errors with context messages.

- `profile_name_exists(name: &str) -> Result<bool>`
  - Calls `load_profiles()` and returns `true` if any profile's name matches `name` case-insensitively.
  - Used by both `cli::run_add_with` and the TUI AddForm to guard against duplicate names before appending.

- `append_profile(profile: &NewProfile) -> Result<()>`
  - Appends a new `[[profiles]]` block (and optional `[profiles.env]` block) to the existing config file.
  - **Env-var generation rules** (when `[profiles.env]` is emitted):
    - `base_url` present → `ANTHROPIC_BASE_URL = "<value>"`
    - `api_key` present → `ANTHROPIC_API_KEY = "<value>"`
    - `model` present → `ANTHROPIC_MODEL`, `ANTHROPIC_SMALL_FAST_MODEL`, `ANTHROPIC_DEFAULT_SONNET_MODEL`, `ANTHROPIC_DEFAULT_OPUS_MODEL`, `ANTHROPIC_DEFAULT_HAIKU_MODEL` all set to `<value>`; plus `API_TIMEOUT_MS = "600000"` and `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"`.
    - `[profiles.env]` section is omitted entirely when none of the above fields are non-empty.
  - Reads then appends (never rewrites) the config file to preserve comments and ordering of existing profiles.
  - Uses the private `non_empty()` helper to skip blank strings.

- `toggle_skip_permissions(profile_name: &str, new_value: bool) -> Result<()>`
  - Surgically updates the `skip_permissions` field of the named profile in the config file.
  - Uses `toml_edit::DocumentMut` to parse and rewrite the file while preserving all comments,
    whitespace, and key ordering for other profiles.
  - Finds the profile by matching `name` exactly (case-sensitive) against the `[[profiles]]` array.
  - Sets `entry["skip_permissions"] = toml_edit::value(new_value)` — inserts the key if absent,
    overwrites it if already present.
  - Returns `Err` if the profile name is not found or if the file cannot be read/written.
  - Callers in `main.rs` reflect the change optimistically in `app.profiles[app.selected]`
    immediately after a successful return, without calling `load_profiles()` again.

### Private Constants

- `DEFAULT_CONFIG: &str` — A `const` string literal containing a commented example `profiles.toml` with one minimal `[[profiles]]` block. Written to disk only when no config file exists. Verified by the `default_config_is_valid_toml` unit test to be parseable TOML.

- `struct Config` — Private deserialization wrapper with a single field `profiles: Vec<Profile>`. Exists only to satisfy TOML's top-level table requirement; not exposed outside this module.
<!-- END:interface -->

---

<!-- BEGIN:dependency_graph -->
## 2. Dependency Graph

### External Crate Dependencies

- **`serde`** (feature `derive`) — Provides the `Deserialize` derive macro applied to `Profile` and `Config`. No `Serialize` is used; config is read-only from Rust's perspective.
- **`toml`** — `toml::from_str::<Config>(&content)` performs the TOML-to-struct deserialization.
- **`toml_edit`** — `toml_edit::DocumentMut` is used by `toggle_skip_permissions` for surgical
  in-place edits that preserve comments and formatting. Only this function uses `toml_edit`;
  read paths continue to use the simpler `toml` crate.
- **`anyhow`** — `anyhow::Result` and the `.with_context(|| ...)` combinator are used for all error propagation, giving callers human-readable error chains.
- **`dirs`** — `dirs::config_dir()` maps to the OS-appropriate XDG config directory (`~/.config` on Linux, `~/Library/Application Support` on macOS). Falls back to `PathBuf::from("~/.config")` if `dirs` returns `None`.

### Standard Library Dependencies

- **`std::collections::HashMap`** — Type for `Profile.env`; maps env var names to values.
- **`std::fs`** — `fs::read_to_string`, `fs::write`, `fs::create_dir_all` for all disk I/O.
- **`std::path::PathBuf`** — Return type of `config_path()` and intermediate path construction.
- **`std::env::var`** — Used inside `config_path()` to read `CCT_CONFIG`.

### Internal Module Dependencies

- **None.** The `config` module is a leaf in the internal dependency graph. It does not import from `app`, `ui`, or `launch`. All other modules that need config data receive a `Vec<Profile>` from `main` rather than calling into this module directly.
<!-- END:dependency_graph -->

---

<!-- BEGIN:state_management -->
## 3. State Management

**Type**: Stateless at runtime.

The `config` module holds no heap-allocated state between calls. Every function is a standalone I/O operation or pure path computation:

- `config_path()` reads one environment variable and constructs a `PathBuf`; the result is not cached.
- `ensure_default_config()` performs file-system side effects (create dirs, write file) and then returns, retaining nothing.
- `load_profiles()` reads the file, parses it into an owned `Vec<Profile>`, and transfers ownership to the caller. No reference to the parsed data remains in this module.

**State on disk** (not in memory):

| Location | Format | Lifecycle |
|---|---|---|
| `$CCT_CONFIG` or `~/.config/cc-tui/profiles.toml` | TOML | Persistent; created once by `ensure_default_config`, mutated only by the user's `$EDITOR` via `launch::open_editor` |

**Hot-reload pattern**: `main.rs` calls `config::load_profiles()` a second time after the editor closes (key `e`). Because this module is stateless, the second call reads the freshly-saved file without any cache invalidation step.
<!-- END:state_management -->

---

<!-- BEGIN:edge_cases -->
## 4. Edge Cases

### CCT_CONFIG Environment Variable Override

- When `CCT_CONFIG` is set to a non-empty value, `config_path()` returns it unconditionally without consulting `dirs`. This allows test harnesses and CI pipelines to supply a fixture file without touching the user's real config directory.
- If `CCT_CONFIG` contains a path whose parent directory does not exist, `ensure_default_config()` will attempt to create it via `fs::create_dir_all`. Failure produces an `anyhow` error with a descriptive context string.

### Missing or Unresolvable XDG Config Directory

- `dirs::config_dir()` returns `None` on platforms where no home directory is configured. The code guards this with `.unwrap_or_else(|| PathBuf::from("~/.config"))`. Note: the fallback is a literal tilde string, which is not automatically expanded by `fs`; on such a system the path would be relative to the working directory and likely fail at the I/O call site.

### TOML Parse Errors

- `load_profiles()` wraps `toml::from_str` with `.with_context(|| format!("parse TOML in {path:?}"))`. A malformed `profiles.toml` (e.g., after a user edit) surfaces as an `anyhow::Error` in `main`. The hot-reload path in `main.rs` handles this gracefully with a `match` that prints a warning and retains the previously-loaded profiles rather than crashing.
- Missing required field: `Profile.name` is the only non-optional field. A `[[profiles]]` block without `name` will fail deserialization with a serde error.

### DEFAULT_CONFIG Bootstrap

- `ensure_default_config()` is idempotent: it only writes the file if it does not already exist. A partially-written or corrupted file that exists on disk will NOT be overwritten; `load_profiles()` will return a parse error instead.
- The `DEFAULT_CONFIG` constant intentionally comments out all optional fields so users see the available knobs without having them take effect. The `default_config_is_valid_toml` unit test guarantees this string is always parseable, preventing template drift.

### Empty Profiles List

- A valid `profiles.toml` containing `profiles = []` (or simply no `[[profiles]]` blocks) passes TOML parsing and returns an empty `Vec<Profile>`. The TUI renders an empty list; the Enter key is guarded by `!app.profiles.is_empty()` in `main.rs`, so no panic occurs.

### File Permissions

- `fs::write` and `fs::create_dir_all` use the process's default umask. No explicit permission bits are set, so the config file and directory inherit the user's umask (typically `0644` / `0755`). Sensitive values like `ANTHROPIC_AUTH_TOKEN` are stored in plaintext; the `ui` module masks them on display, but the file itself is not encrypted.
<!-- END:edge_cases -->

---

<!-- BEGIN:usage_example -->
## 5. Usage Example

The following pseudocode mirrors the actual call sites in `src/main.rs`:

```rust
use cct::{config, app, launch, ui};
use app::App;

fn main() -> anyhow::Result<()> {
    // Step 1: Ensure ~/.config/cc-tui/profiles.toml exists.
    // Creates parent dirs and writes DEFAULT_CONFIG on first run.
    // No-op on subsequent runs. Fails fast with a descriptive error
    // if the directory cannot be created (e.g., permission denied).
    config::ensure_default_config()?;

    // Step 2: Read and deserialize all profiles.
    // Returns Vec<Profile>; ownership transfers entirely to the caller.
    // Errors here mean the file is unreadable or contains invalid TOML.
    let profiles: Vec<config::Profile> = config::load_profiles()?;

    // Step 3: Hand profiles to the App state machine.
    let mut app = App::new(profiles);

    // --- Main event loop ---
    loop {
        // ... draw TUI, read key events ...

        // On Enter: exec-replace with claude using the selected profile.
        // Profile fields (name, model, skip_permissions, extra_args, env)
        // are read by launch::exec_claude — config module is not called again.
        if user_pressed_enter {
            launch::exec_claude(&app.profiles[app.selected]);
        }

        // On 'e': open editor, then hot-reload profiles without restart.
        // config_path() is called to pass the file path to the editor.
        if user_pressed_e {
            launch::open_editor(&config::config_path())?;

            // Second call to load_profiles() picks up edits.
            // Errors are warned rather than propagated, preserving the
            // previous valid state in app.profiles.
            match config::load_profiles() {
                Ok(updated) => {
                    app.profiles = updated;
                    // Clamp cursor if list shrank.
                    if app.selected >= app.profiles.len() {
                        app.selected = app.profiles.len().saturating_sub(1);
                    }
                }
                Err(e) => eprintln!("Warning: profile reload failed: {e:#}"),
            }
        }
    }
}

// In tests or CI: override the config path via environment variable.
// CCT_CONFIG=/tmp/fixture.toml cargo test
// config::config_path() will return PathBuf::from("/tmp/fixture.toml").
```
<!-- END:usage_example -->

---

## Quality Gate Checklist

- [x] **Interface**: 3 exported types (`Profile`, `NewProfile`; `Config` private) + 5 public functions + 1 constant documented
- [x] **Dependencies**: All external crates (`serde`, `toml`, `anyhow`, `dirs`) and std modules listed with reasoning; internal leaf status stated
- [x] **State Management**: Clearly stateless at runtime; on-disk state lifecycle documented with hot-reload pattern explained
- [x] **Edge Cases**: 6 cases identified — CCT_CONFIG override, missing XDG dir, TOML parse errors, DEFAULT_CONFIG bootstrap, empty profiles list, file permissions
- [x] **Usage Example**: Rust pseudocode mirrors real `main.rs` call sites, covers initial load and hot-reload paths
- [x] **YAML Frontmatter**: `doc_type`, `module_name`, `module_path`, `generated_by` all present

---

**Template Version**: 2.0
**Last Updated**: 2026-03-03 (revision 2 — added NewProfile, append_profile, profile_name_exists)
