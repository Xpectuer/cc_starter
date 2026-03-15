## Case 1 — toggle_full_auto_insert

### Actions Taken
- RED: Added `toggle_full_auto_insert` test to config.rs. Compilation failed (function not found). ✅
- GREEN: Added `toggle_full_auto()` function mirroring `toggle_skip_permissions`. Test passes. ✅
- REFACTOR: No refactoring needed — function mirrors existing pattern.

### Verify Result
`cargo test toggle_full_auto_insert` — PASS
