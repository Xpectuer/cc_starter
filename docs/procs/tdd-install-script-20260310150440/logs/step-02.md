## Case 2 — detect_macos_arm64

### RED
- Added test stubbing uname to return Darwin/arm64
- Test passed immediately — detect() already implemented in Case 1

### GREEN
- No code changes needed — already green

### REFACTOR
- Restructured test file: moved uname stubs into each test function (was previously global)
- All tests still pass

### Result
SUCCESS — detect() already handles macOS arm64 from Case 1 implementation
