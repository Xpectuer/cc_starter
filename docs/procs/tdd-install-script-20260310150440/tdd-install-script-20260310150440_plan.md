---
title: "Plan: cct install script"
doc_type: proc
brief: "Implementation plan for curl|bash installer that downloads latest cct binary"
confidence: verified
created: 2026-03-10
updated: 2026-03-10
revision: 1
---

# Plan: cct install script

## Files Changed

| File | Change Type |
|------|-------------|
| `install.sh` | New file |

## Step 1 — Create install.sh

**File**: `install.sh`
**What**: Create the complete install script with detect, fetch, download+verify+retry, install, and PATH hint.

**New**:
```bash
#!/bin/bash
set -euo pipefail

REPO="zhengjy/cc_starter"
INSTALL_DIR="${HOME}/.local/bin"
MAX_RETRIES=3
RETRY_DELAY=2

err() {
    echo "Error: $*" >&2
    exit 1
}

log() {
    echo ":: $*"
}

detect() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "${os}" in
        Darwin)
            case "${arch}" in
                arm64|aarch64) TARGET="aarch64-apple-darwin" ;;
                x86_64)        TARGET="x86_64-apple-darwin" ;;
                *)             err "Unsupported architecture on macOS: ${arch}" ;;
            esac
            ;;
        Linux)
            case "${arch}" in
                x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
                *)      err "Unsupported architecture on Linux: ${arch}" ;;
            esac
            ;;
        *) err "Unsupported OS: ${os}" ;;
    esac

    log "Detected platform: ${TARGET}"
}

fetch_latest() {
    local api_url="https://api.github.com/repos/${REPO}/releases/latest"
    local response

    response="$(curl -fsSL "${api_url}" 2>/dev/null)" \
        || err "Failed to fetch latest release from GitHub API"

    VERSION="$(echo "${response}" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)"

    [ -n "${VERSION}" ] || err "Could not parse release version from GitHub API response"

    log "Latest release: ${VERSION}"
}

download() {
    local url="https://github.com/${REPO}/releases/download/${VERSION}/cct-${TARGET}.tar.gz"
    local attempt=1

    while [ "${attempt}" -le "${MAX_RETRIES}" ]; do
        log "Downloading cct-${TARGET}.tar.gz (attempt ${attempt}/${MAX_RETRIES})..."

        if curl -fSL "${url}" -o "${TMPDIR_INSTALL}/cct.tar.gz" 2>/dev/null \
            && tar -tzf "${TMPDIR_INSTALL}/cct.tar.gz" >/dev/null 2>&1; then
            log "Download verified."
            return 0
        fi

        rm -f "${TMPDIR_INSTALL}/cct.tar.gz"

        if [ "${attempt}" -lt "${MAX_RETRIES}" ]; then
            log "Download failed. Retrying in ${RETRY_DELAY}s..."
            sleep "${RETRY_DELAY}"
        fi

        attempt=$((attempt + 1))
    done

    err "Download failed after ${MAX_RETRIES} attempts"
}

install_binary() {
    mkdir -p "${INSTALL_DIR}"

    tar -xzf "${TMPDIR_INSTALL}/cct.tar.gz" -C "${TMPDIR_INSTALL}/"

    install -m 755 "${TMPDIR_INSTALL}/cct" "${INSTALL_DIR}/cct" \
        || err "Failed to install cct to ${INSTALL_DIR}"

    log "Installed cct to ${INSTALL_DIR}/cct"
}

path_hint() {
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            echo ""
            echo "Add ${INSTALL_DIR} to your PATH:"
            echo "  export PATH=\"\${HOME}/.local/bin:\$PATH\""
            echo ""
            echo "Add the line above to ~/.bashrc or ~/.zshrc to make it permanent."
            ;;
    esac
}

main() {
    command -v curl >/dev/null 2>&1 || err "curl is required but not found"
    command -v tar  >/dev/null 2>&1 || err "tar is required but not found"

    TMPDIR_INSTALL="$(mktemp -d)"
    trap 'rm -rf "${TMPDIR_INSTALL}"' EXIT

    detect
    fetch_latest
    download
    install_binary
    path_hint

    echo ""
    log "cct ${VERSION} installed successfully!"
}

main
```

**Verify**: `bash -n install.sh && echo PASS || echo FAIL`

## Step 2 — ShellCheck lint

**File**: `install.sh`
**What**: Run shellcheck to verify the script has no lint issues.

**Verify**: `shellcheck install.sh && echo PASS || echo FAIL`

## Step 3 — Proof-Read End-to-End

Read `install.sh` in full. Check: formatting, no leftover TODOs, spec intent preserved.

## Step 4 — Cross-Check Acceptance Criteria

| Criterion | Addressed in |
|-----------|--------------|
| install.sh exists at repo root | Step 1 |
| Detects OS/arch on macOS (arm64, x86_64) and Linux (x86_64) | Step 1 — `detect()` |
| Exits with clear error on unsupported platform | Step 1 — `detect()` case `*` |
| Fetches latest release tag from GitHub API | Step 1 — `fetch_latest()` |
| Downloads correct cct-{target}.tar.gz asset | Step 1 — `download()` |
| Extracts and places cct binary in ~/.local/bin/ | Step 1 — `install_binary()` |
| Creates ~/.local/bin/ if it doesn't exist | Step 1 — `mkdir -p` in `install_binary()` |
| Prints PATH hint when ~/.local/bin not in PATH | Step 1 — `path_hint()` |
| Prints success message with installed version | Step 1 — `main()` final log |
| Fails gracefully if curl/tar not available | Step 1 — `command -v` checks in `main()` |
| Idempotent (re-running overwrites existing binary) | Step 1 — `install -m 755` overwrites |
| Download verified via tar -tzf | Step 1 — `download()` integrity check |
| Failed downloads retried up to 3 times | Step 1 — `download()` retry loop |
| Script wrapped in main() | Step 1 — `main` called at EOF |

All criteria mapped.

## Step 5 — Review

Follow Phase 3 (self-review). Write `review.md`.

## Step 6 — Commit

Use /commit. Suggested message:
feat: add curl|bash install script for cct binary
- detect OS/arch and map to release target
- fetch latest release from GitHub API
- download with tar integrity verification and 3-retry loop
- install to ~/.local/bin with PATH hint

## Execution Order

Step 1 → Step 2 → Step 3 → Step 4 → Step 5 → Step 6

(No parallel-safe steps — all sequential.)
