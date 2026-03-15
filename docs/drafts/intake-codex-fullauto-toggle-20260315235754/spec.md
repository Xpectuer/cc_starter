---
title: "Spec: Codex full_auto toggle via [s] key"
doc_type: proc
brief: "Design spec for toggling full_auto on Codex profiles with [s] key in TUI"
confidence: verified
created: 2026-03-16
updated: 2026-03-16
revision: 1
---

# Spec: Codex full_auto toggle via [s] key

## Chosen Approach

Mirror the existing `toggle_skip_permissions` pattern. No abstraction, no new dependencies.

## Design

### config.rs — `toggle_full_auto()`

New function `toggle_full_auto(profile_name: &str, new_value: bool) -> Result<()>`:

1. Read `config_path()` into string
2. Parse as `toml_edit::DocumentMut`
3. Navigate to `[[profiles]]` array, find entry where `name == profile_name`
4. Set `entry["full_auto"] = toml_edit::value(new_value)`
5. Write doc back to file

Identical structure to `toggle_skip_permissions` (config.rs:186-206).

### main.rs — `s` key handler

Replace the current Claude-only guard:

```rust
// Before:
if profile.backend == Backend::Claude { toggle_skip_permissions... }

// After:
match profile.backend {
    Backend::Claude => {
        // existing skip_permissions toggle (unchanged)
    }
    Backend::Codex => {
        let old_val = profile.full_auto.unwrap_or(false);
        let new_val = !old_val;
        match config::toggle_full_auto(&profile.name, new_val) {
            Ok(()) => { profile.full_auto = Some(new_val); }
            Err(e) => { eprintln!("Warning: toggle failed: {e:#}"); }
        }
    }
}
```

### ui.rs — dynamic footer

Footer `s` key hint changes based on `app.active_backend`:
- Claude tab: `s: skip_perms`
- Codex tab: `s: full_auto`

The detail panel already renders `full_auto` when present — no changes needed there.

## Testing

### Unit tests (config.rs)

| Test | Description |
|------|-------------|
| `toggle_full_auto_insert` | Profile without `full_auto` field → inserts `full_auto = true` |
| `toggle_full_auto_flip` | `full_auto = true` → `false` → `true` round-trip |
| `toggle_full_auto_not_found` | Missing profile name → returns error |

### Integration test

Verify that a Codex profile's `full_auto` value persists after toggle (already covered by config unit tests; integration test is optional).

## Decisions

- **No generic toggle function**: Two 20-line functions are simpler than one generic with field-name dispatch. KISS principle.
- **No detail panel changes**: `full_auto` field already rendered in the existing detail panel logic.
- **No new dependencies**: `toml_edit` already in Cargo.toml.

## Open Questions

None.
