## Case 4 — fetch_latest_parses_version (MANUAL → stub)

### RED
- Added test that stubs `curl` to return mock JSON with `"tag_name": "v0.3.1"`
- Test asserts `VERSION` is set to `"v0.3.1"` after calling fetch_latest()

### GREEN
- Implemented fetch_latest() in install.sh — uses curl + sed to parse tag_name
- Test passes

### REFACTOR
- No changes needed

### Result
SUCCESS — handled via curl stub instead of real API call
