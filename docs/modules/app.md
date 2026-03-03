---
doc_type: module
module_name: "app"
module_path: "src/app.rs"
generated_by: mci-phase-2
---

# app Module Documentation

> **Purpose**: Owns the mutable cursor state (selected profile index) and provides circular navigation (`next`/`prev`) over the profile list for the TUI event loop.
> **Path**: src/app.rs

---

<!-- BEGIN:interface -->
## 1. Interface

### Exported Struct

- `pub struct App` — sole exported type; holds all runtime UI state.

### Fields

- `pub profiles: Vec<Profile>` — ordered list of profiles loaded from `~/.config/cc-tui/profiles.toml`; may be replaced in-place during hot-reload without restarting the process.
- `pub selected: usize` — zero-based index of the currently highlighted profile row in the TUI list.

### Methods

- `App::new(profiles: Vec<Profile>) -> Self` — constructs an `App` with `selected` initialised to `0`; accepts an empty vec without panicking.
- `app.next(&mut self)` — advances `selected` by one position, wrapping from the last index back to `0` (circular); no-op when `profiles` is empty.
- `app.prev(&mut self)` — retreats `selected` by one position, wrapping from `0` to the last index (circular); no-op when `profiles` is empty.

**Quality Check**: 4 public interface points documented (struct, 2 fields, constructor, 2 navigation methods).
<!-- END:interface -->

---

<!-- BEGIN:dependency_graph -->
## 2. Dependency Graph

- **Imports from `crate::config`** → `Profile` struct (the element type of `Vec<Profile>`); this is the only cross-module dependency.
- **No `std` imports beyond language primitives** — no I/O, threading, or collections beyond `Vec` (already provided by the prelude).
- **No external crates** — the module carries zero third-party dependencies.
- **Does NOT depend on**: `ui`, `launch`, `config::load_profiles`, or any OS APIs. It is deliberately isolated so it can be unit-tested without a terminal or filesystem.

**Quality Check**: Single internal dependency clearly stated; absence of external dependencies confirmed.
<!-- END:dependency_graph -->

---

<!-- BEGIN:state_management -->
## 3. State Management

**Type**: Stateful — `App` is the single mutable owner of all TUI runtime state.

- **`selected` field** — mutated in-place by `next()` and `prev()` on every keypress. Its lifecycle begins at `0` (construction) and ends when the process is exec-replaced or exits. It never touches disk.
- **`profiles` field** — initially set from `config::load_profiles()` in `main`. On hot-reload (key `e`), `main.rs` replaces `app.profiles` entirely with a freshly parsed `Vec<Profile>`. After replacement, `main.rs` clamps `selected` with `app.selected = app.profiles.len().saturating_sub(1)` if the cursor is now out of bounds (the clamp logic lives in the caller, not in `App` itself).
- **Circular wrapping** — `next()` uses `% profiles.len()` and `prev()` explicitly wraps from `0` to `len - 1`, so `selected` is always a valid index whenever the list is non-empty.
- **No interior mutability** — there are no `Mutex`, `RefCell`, or `Arc` wrappers; the caller holds a single `&mut App` and drives all mutations sequentially from the event loop.

**Quality Check**: State lifecycle, mutation points, and ownership model fully documented.
<!-- END:state_management -->

---

<!-- BEGIN:edge_cases -->
## 4. Edge Cases

### Empty Profile List

Both `next()` and `prev()` open with an `if !self.profiles.is_empty()` guard and return immediately without touching `selected`. This prevents a panic from `% 0` in `next()` and from underflowing `usize` in `prev()`. The UI and launch code also guard on `!app.profiles.is_empty()` before accessing `app.profiles[app.selected]`.

### Cursor Out-of-Bounds After Hot-Reload

`App` itself does not clamp `selected` when `profiles` is replaced. The caller (`main.rs`, lines 50-51) performs the clamp:

```rust
if app.selected >= app.profiles.len() {
    app.selected = app.profiles.len().saturating_sub(1);
}
```

`saturating_sub(1)` on a zero-length vec yields `0`, so after a reload that empties the list, `selected` lands at `0` and the `is_empty` guards in `next`/`prev` keep it safe.

### Single-Profile List

When `profiles.len() == 1`, both `next()` and `prev()` leave `selected` at `0` — `(0 + 1) % 1 == 0` for next, and the `else` branch of prev computes `selected -= 1` which would underflow, but the `selected == 0` arm is taken first and wraps to `len - 1 == 0`. Navigation appears to do nothing visually, which is the correct behaviour.

### `selected` Type Is `usize`

`selected` cannot be negative. The `prev()` implementation avoids underflow by checking `selected == 0` before subtracting. Any future refactor that changes this check must preserve that guard.

**Quality Check**: 4 edge cases identified with specific code references.
<!-- END:edge_cases -->

---

<!-- BEGIN:usage_example -->
## 5. Usage Example

```rust
// src/main.rs — how the event loop uses App

use cct::{app::App, config, launch, ui};

fn main() -> anyhow::Result<()> {
    // 1. Load profiles from disk once at startup
    let profiles = config::load_profiles()?;

    // 2. Construct App — selected starts at 0
    let mut app = App::new(profiles);

    loop {
        // 3. Pass an immutable reference to the renderer
        tui.draw(|f| ui::draw(&app, f))?;

        match event::read()? {
            // 4. Navigate down (wraps at end of list)
            KeyCode::Down | KeyCode::Char('j') => app.next(),

            // 5. Navigate up (wraps at beginning of list)
            KeyCode::Up | KeyCode::Char('k') => app.prev(),

            // 6. Launch — profiles[selected] is always a valid index here
            KeyCode::Enter if !app.profiles.is_empty() => {
                launch::exec_claude(&app.profiles[app.selected]);
            }

            // 7. Hot-reload: replace profiles, then clamp cursor
            KeyCode::Char('e') => {
                launch::open_editor(&config::config_path())?;
                if let Ok(updated) = config::load_profiles() {
                    app.profiles = updated;
                    // Clamp — App does not do this internally
                    if app.selected >= app.profiles.len() {
                        app.selected = app.profiles.len().saturating_sub(1);
                    }
                }
            }

            _ => {}
        }
    }
}
```

**Quality Check**: Example covers construction, both navigation methods, field access for launch, and the hot-reload cursor-clamp pattern.
<!-- END:usage_example -->

---

## Quality Gate Checklist

- [x] **Interface**: 4 public interface points documented (struct, 2 fields, 3 methods)
- [x] **Dependencies**: Single internal dependency (`crate::config::Profile`) listed with reasoning; no external crates
- [x] **State Management**: Stateful; mutation points, ownership, and hot-reload lifecycle documented
- [x] **Edge Cases**: 4 special cases identified (empty list, out-of-bounds after reload, single-item list, usize underflow guard)
- [x] **Usage Example**: Rust pseudocode mirrors actual `main.rs` patterns with annotations
- [x] **YAML Frontmatter**: `doc_type`, `module_name`, `module_path` present

---

**Template Version**: 2.0
**Generated**: 2026-03-03
