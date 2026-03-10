## Case 1 — detect_linux_x86_64

### RED

- Created `install.sh` with shebang, `err()`, `log()` helpers, and an empty `detect()` function (just `:`).
- Created `tests/install.bats` with a test that stubs `uname -s` to return `Linux` and `uname -m` to return `x86_64`, then sources `install.sh`, calls `detect`, and asserts `TARGET="x86_64-unknown-linux-gnu"`.
- Ran `bats tests/install.bats` — **FAILED** as expected (`TARGET: unbound variable`).

### GREEN

- Implemented the full `detect()` function in `install.sh` with `case` dispatch on `uname -s` (Darwin/Linux) and `uname -m` (x86_64, arm64/aarch64).
- Ran `bats tests/install.bats` — **PASSED** (1/1 ok).

### REFACTOR

- No refactoring needed; code is minimal and clean.
- Ran `bats tests/install.bats` — **PASSED** (1/1 ok).

### Result

**SUCCESS** — `detect()` correctly sets `TARGET="x86_64-unknown-linux-gnu"` when `uname -s` returns `Linux` and `uname -m` returns `x86_64`.
