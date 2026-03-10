## Case 9 — path_hint_silent_when_in_path

### RED
- Added test setting INSTALL_DIR to /usr/bin (which is in PATH)
- Asserts output is empty (no hint shown)

### GREEN
- path_hint() already handles this — the case match skips output
- Test passes

### REFACTOR
- No changes needed

### Result
SUCCESS
