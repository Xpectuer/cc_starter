---
title: "Lesson: BATS Shell Function Stubbing with export -f"
doc_type: lesson
brief: "How to stub external commands (curl, uname, sleep) in BATS tests by overriding them as exported bash functions"
confidence: verified
created: 2026-03-10
updated: 2026-03-10
revision: 1
---

# Lesson: BATS Shell Function Stubbing with export -f

## Context

When testing `install.sh` with BATS, the script calls external tools (`curl`, `uname`, `sleep`,
`tar`) that are not safe or practical to call in a test environment. BATS provides a clean
mechanism to stub them without modifying the script under test.

## The Pattern

Define a shell function with the same name as the command, then export it with `export -f`.
When the script is `source`d into the same shell, the function shadows the real binary.

```bash
@test "fetch_latest parses version from GitHub API response" {
    curl() {
        cat <<'MOCK_JSON'
{
  "tag_name": "v0.3.1",
  "name": "Release v0.3.1"
}
MOCK_JSON
    }
    export -f curl

    source "${BATS_TEST_DIRNAME}/../install.sh"
    fetch_latest
    [ "$VERSION" = "v0.3.1" ]
}
```

## Rules

1. **Declare the function before `source`ing the script.** The override must be in scope when
   the script's function definitions are executed.
2. **Use `export -f <name>`** after the function definition so it is visible to subshells that
   the sourced script might spawn (e.g., command substitutions like `$(curl ...)`).
3. **Override `sleep` to a no-op** in retry tests to avoid wall-clock delays:
   ```bash
   sleep() { :; }
   export -f sleep
   ```
4. **The script must support sourcing.** `install.sh` uses:
   ```bash
   if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
       main
   fi
   ```
   This guard ensures `main` is not called when sourced — a requirement for test isolation.

## When to Use `run`

Use `run <function>` when you expect the function to exit with a non-zero status:

```bash
@test "detect exits with error on unsupported OS" {
    uname() {
        case "$1" in
            -s) echo "FreeBSD" ;;
            -m) echo "x86_64" ;;
        esac
    }
    export -f uname
    source "${BATS_TEST_DIRNAME}/../install.sh"
    run detect
    [ "$status" -ne 0 ]
    [[ "$output" == *"Unsupported OS"* ]]
}
```

Without `run`, a non-zero exit from `detect` would abort the test with a confusing failure.

## Global Variables Set by Functions

`install.sh` uses globals (`TARGET`, `VERSION`, `TMPDIR_INSTALL`) that must be set or
pre-populated before calling a downstream function in isolation:

```bash
@test "download retries on failure and eventually errors" {
    # Pre-set globals that download() reads
    TMPDIR_INSTALL="$(mktemp -d)"
    TARGET="x86_64-unknown-linux-gnu"
    VERSION="v0.1.0"
    MAX_RETRIES=2     # Override to keep test fast
    RETRY_DELAY=0
    run download
    ...
}
```

## What Doesn't Work

- **Symlink or PATH prepend approach**: Creating a fake `curl` binary in a temp dir and
  prepending it to `$PATH` works for subshell invocations but not for functions called
  within the same shell (command substitutions in sourced scripts). `export -f` is more
  reliable for sourced scripts.
- **Calling `main` directly with mocks**: The `main()` function calls all sub-functions in
  sequence, making it hard to isolate individual functions. Test each function independently
  after sourcing.

## Discovered During

TDD proc: `docs/procs/tdd-install-script-20260310150440` — 9 test cases for `install.sh`,
all green. Test file: `tests/install.bats`.
