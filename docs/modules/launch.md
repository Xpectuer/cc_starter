---
doc_type: module
module_name: "launch"
module_path: "src/launch.rs"
generated_by: mci-phase-2
---

# launch Module Documentation

> **Purpose**: Handles all process-lifecycle concerns for `cct`: builds the `claude` CLI argument list from a profile, exec-replaces the current process with `claude`, restores terminal state before any exec or editor spawn, and opens `$EDITOR` for config hot-reload.
> **Path**: src/launch.rs

---

<!-- BEGIN:interface -->
## 1. Interface

### Exported Functions

- `pub fn restore_terminal()`
  - Disables crossterm raw mode and emits `LeaveAlternateScreen` to stdout.
  - Returns: `()` (errors from crossterm are silently discarded with `let _ = ...`).
  - Must be called before any `exec_claude` or `open_editor` invocation to ensure the terminal is returned to cooked mode.

- `pub fn build_args(profile: &Profile) -> Vec<String>`
  - Pure function with no side effects.
  - Constructs the ordered CLI argument list for the `claude` binary from a `Profile`.
  - Argument ordering: `--model <value>` (if `profile.model` is `Some`), then `--dangerously-skip-permissions` (if `profile.skip_permissions` is `Some(true)`), then each element of `profile.extra_args` in order.
  - Returns: `Vec<String>` — may be empty if all profile fields are absent or false.

- `pub fn exec_claude(profile: &Profile) -> anyhow::Error`
  - Injects all key-value pairs from `profile.env` into the current process environment via `env::set_var`.
  - Calls `build_args(profile)` then exec-replaces the current process with `claude <args>` using `std::os::unix::process::CommandExt::exec`.
  - **Never returns on success** — the process image is replaced.
  - Returns: `anyhow::Error` only when `exec` itself fails (e.g., `claude` binary not found on `$PATH`).

- `pub fn open_editor(path: &Path) -> Result<()>`
  - Reads `$EDITOR`; falls back to `"vi"` if the variable is unset or empty.
  - Spawns the editor as a child process, blocking until it exits.
  - Returns: `Ok(())` on clean editor exit; `Err(anyhow::Error)` with context message `"spawn editor \"<editor>\""` if the spawn fails.

### Exported Types

None — all public surface is functions. The module consumes `crate::config::Profile` from the `config` module.

<!-- END:interface -->

---

<!-- BEGIN:dependency_graph -->
## 2. Dependency Graph

- **Imports from `crate::config`** → `Profile` struct (fields: `model`, `skip_permissions`, `extra_args`, `env`). The `launch` module is a pure consumer of `Profile`; it does not write to the config layer.
- **Imports from `std::os::unix::process::CommandExt`** → Provides the `.exec()` method on `std::process::Command`. This trait is Unix-only; the module will not compile on Windows targets.
- **Imports from `std::process::Command`** → Used to construct both the `claude` subprocess (for exec) and the editor subprocess (for `open_editor`).
- **Imports from `std::env`** → `env::set_var` (inject profile env vars) and `env::var` (read `$EDITOR`).
- **Imports from `crossterm`** → `terminal::disable_raw_mode` and `execute!(stdout, LeaveAlternateScreen)` for terminal cleanup in `restore_terminal`.
- **Imports from `anyhow`** → `Context` trait (adds context to `open_editor` errors) and `Result` alias. `exec_claude` returns a bare `anyhow::Error` rather than `Result<_>` because it has no success path.
- **Does NOT depend on**: `app`, `ui`, or any async runtime. The module is synchronous and has no shared mutable state.

<!-- END:dependency_graph -->

---

<!-- BEGIN:state_management -->
## 3. State Management

**Type**: Effectively stateless for `build_args` and `open_editor`; **process-mutating** for `exec_claude`.

- **`build_args`** — Purely functional. Takes a `&Profile` reference, performs no I/O, allocates a `Vec<String>`, and returns it. Calling it multiple times with the same input produces identical output.

- `open_editor` — Spawns a child process and blocks. It reads `$EDITOR` from the environment at call time but does not retain or modify any state. The child's exit status is checked via `.status()` and discarded beyond the `Ok`/`Err` distinction.

- **`exec_claude`** — Has two permanent side effects on process-global state:
  1. **Environment mutation**: calls `env::set_var(k, v)` for every entry in `profile.env`. These changes persist for the lifetime of the process (and are inherited by any forked children). Because `exec` replaces the process image on success, this is acceptable; on failure the caller exits with code 1 (see `main.rs:39`).
  2. **Process replacement**: `CommandExt::exec()` replaces the current process image with `claude`. There is no return path, no stack unwind, and no destructor invocation on success. The TUI terminal cleanup (`restore_terminal`) must therefore be called by the caller **before** `exec_claude`.

- **`restore_terminal`** — Interacts with global terminal state via crossterm. Errors are intentionally suppressed (`let _ = ...`) to ensure the function is always safe to call even if raw mode was never enabled.

<!-- END:state_management -->

---

<!-- BEGIN:edge_cases -->
## 4. Edge Cases

### Hardcoded Values and Fallbacks

- **Editor fallback**: `open_editor` defaults to `"vi"` when `$EDITOR` is unset. There is no validation that `vi` exists on the system; a missing `vi` will produce an `Err` with the context message `spawn editor "vi"`.
- **`--dangerously-skip-permissions` flag**: Only appended when `profile.skip_permissions` is explicitly `Some(true)`. A missing field (`None`) is treated identically to `Some(false)` via `unwrap_or(false)`.

### Error Handling Quirks

- **`exec_claude` return type is `anyhow::Error`, not `Result<!, anyhow::Error>`**: Rust's stable toolchain does not support the never type (`!`) as a return value in all positions. The function signature signals intent through its doc comment ("Returns only on error") but cannot enforce it statically. Callers must treat the return value as always representing failure.
- **`restore_terminal` swallows errors**: Both `disable_raw_mode()` and `execute!(...)` return `Result`s that are explicitly discarded. This is intentional — if the terminal is already in cooked mode, the call is a no-op and failing silently is correct.
- **`exec` error wrapping**: The error from `CommandExt::exec()` is wrapped in an `anyhow::anyhow!("exec claude: {err}")` string rather than using `.context()`, because `exec()` returns `io::Error` directly (not a `Result` with a success arm to chain from).

### Argument Ordering Contract

The ordering of arguments appended by `build_args` is deterministic and tested:
1. `--model <value>` (positional pair, only when `model` is `Some`)
2. `--dangerously-skip-permissions` (flag, only when `skip_permissions` is `Some(true)`)
3. Elements of `extra_args` in their original TOML order (appended verbatim)

Callers must not assume any other ordering. The three unit tests (`build_args_empty`, `build_args_model_only`, `build_args_full`) pin this contract.

### Unix-Only Constraint

`std::os::unix::process::CommandExt` is gated to Unix targets by the standard library. Compiling `cct` on Windows will fail at this import. There is no `#[cfg(unix)]` guard or Windows fallback; this is an intentional design constraint (terminal-based `exec` semantics are Unix-specific).

### Environment Variable Injection Race

`env::set_var` is not thread-safe in a multi-threaded program (it is `unsafe` in Rust editions that expose that). `cct` is single-threaded in its event loop, so this is safe in practice, but care must be taken if the architecture is ever extended to use background threads before the `exec` call.

<!-- END:edge_cases -->

---

<!-- BEGIN:usage_example -->
## 5. Usage Example

The following reproduces the actual call pattern from `src/main.rs`:

```rust
// --- Enter key pressed: launch selected profile ---
// Step 1: restore terminal BEFORE exec (mandatory ordering)
launch::restore_terminal();

// Step 2: exec_claude replaces the process; only returns on error
let err = launch::exec_claude(&app.profiles[app.selected]);

// Step 3: exec failed — print error and exit with non-zero code
eprintln!("Error: {err:#}");
std::process::exit(1);

// --- 'e' key pressed: hot-reload config via $EDITOR ---
// Step 1: restore terminal so the editor gets a clean cooked-mode terminal
launch::restore_terminal();

// Step 2: open editor on the config file path; blocks until editor exits
launch::open_editor(&config::config_path())?;

// Step 3: re-enter raw mode and re-draw the TUI
enable_raw_mode()?;
execute!(io::stdout(), EnterAlternateScreen)?;
tui.clear()?;

// --- Inspecting what args would be built (e.g., for logging or testing) ---
let profile = Profile {
    name: "prod".into(),
    description: Some("Production endpoint".into()),
    model: Some("claude-opus-4-6".into()),
    skip_permissions: Some(false),
    extra_args: Some(vec!["--verbose".into()]),
    env: Some([
        ("ANTHROPIC_BASE_URL".into(), "https://api.example.com".into()),
        ("ANTHROPIC_AUTH_TOKEN".into(), "sk-ant-...".into()),
    ].into()),
};

let args = launch::build_args(&profile);
// args == ["--model", "claude-opus-4-6", "--verbose"]
// (skip_permissions=false → no --dangerously-skip-permissions flag)
```

<!-- END:usage_example -->

---

## Quality Gate Checklist

- [x] **Interface**: 4 public functions documented with signatures, return types, and semantics
- [x] **Dependencies**: All internal and external module dependencies listed with reasoning
- [x] **State Management**: Clearly distinguishes pure functions from process-mutating functions; lifecycle of env mutation explained
- [x] **Edge Cases**: Editor fallback, error-type quirk, argument ordering contract, Unix-only constraint, env set_var threading note
- [x] **Usage Example**: Concrete Rust pseudocode mirroring actual `main.rs` call patterns for both Enter (exec) and 'e' (editor) flows
- [x] **YAML Frontmatter**: `doc_type`, `module_name`, `module_path` present

---

**Template Version**: 2.0
**Last Updated**: 2026-03-03
