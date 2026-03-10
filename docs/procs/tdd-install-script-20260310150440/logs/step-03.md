## Case 3 — detect_unsupported_os

### RED
- Added test stubbing uname to return FreeBSD/x86_64
- Uses `run detect` to capture exit status and output
- Asserts non-zero exit and "Unsupported OS" in output

### GREEN
- No code changes needed — detect() already handles unsupported OS with err()

### REFACTOR
- No changes needed
- All 3 tests pass

### Result
SUCCESS — detect() error path already implemented in Case 1
