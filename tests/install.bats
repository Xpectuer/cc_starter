#!/usr/bin/env bats

@test "detect sets TARGET to x86_64-unknown-linux-gnu on Linux x86_64" {
    uname() {
        case "$1" in
            -s) echo "Linux" ;;
            -m) echo "x86_64" ;;
            *)  command uname "$@" ;;
        esac
    }
    export -f uname
    source "${BATS_TEST_DIRNAME}/../install.sh"
    detect
    [ "$TARGET" = "x86_64-unknown-linux-gnu" ]
}

@test "detect sets TARGET to aarch64-apple-darwin on macOS arm64" {
    uname() {
        case "$1" in
            -s) echo "Darwin" ;;
            -m) echo "arm64" ;;
            *)  command uname "$@" ;;
        esac
    }
    export -f uname
    source "${BATS_TEST_DIRNAME}/../install.sh"
    detect
    [ "$TARGET" = "aarch64-apple-darwin" ]
}

@test "detect exits with error on unsupported OS" {
    uname() {
        case "$1" in
            -s) echo "FreeBSD" ;;
            -m) echo "x86_64" ;;
            *)  command uname "$@" ;;
        esac
    }
    export -f uname
    source "${BATS_TEST_DIRNAME}/../install.sh"
    run detect
    [ "$status" -ne 0 ]
    [[ "$output" == *"Unsupported OS"* ]]
}

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

@test "fetch_latest fails on bad response with no tag_name" {
    curl() {
        echo '{"error": "not found"}'
    }
    export -f curl
    source "${BATS_TEST_DIRNAME}/../install.sh"
    run fetch_latest
    [ "$status" -ne 0 ]
    [[ "$output" == *"Could not parse release version"* ]]
}

@test "download retries on failure and eventually errors" {
    curl() {
        return 1
    }
    export -f curl
    # Override sleep to avoid waiting
    sleep() { :; }
    export -f sleep
    source "${BATS_TEST_DIRNAME}/../install.sh"
    TMPDIR_INSTALL="$(mktemp -d)"
    TARGET="x86_64-unknown-linux-gnu"
    VERSION="v0.1.0"
    MAX_RETRIES=2
    RETRY_DELAY=0
    run download
    rm -rf "$TMPDIR_INSTALL"
    [ "$status" -ne 0 ]
    [[ "$output" == *"Download failed after"* ]]
}

@test "install_binary creates dir and copies binary" {
    source "${BATS_TEST_DIRNAME}/../install.sh"
    # Set up a fake tarball with a "cct" binary inside
    TMPDIR_INSTALL="$(mktemp -d)"
    local test_install_dir="$(mktemp -d)/install_test"
    INSTALL_DIR="${test_install_dir}"
    # Create a fake cct binary and tar it
    echo '#!/bin/bash' > "${TMPDIR_INSTALL}/cct"
    chmod +x "${TMPDIR_INSTALL}/cct"
    tar -czf "${TMPDIR_INSTALL}/cct.tar.gz" -C "${TMPDIR_INSTALL}" cct
    # Run install_binary
    install_binary
    # Verify
    [ -f "${test_install_dir}/cct" ]
    [ -x "${test_install_dir}/cct" ]
    # Cleanup
    rm -rf "${TMPDIR_INSTALL}" "${test_install_dir}"
}

@test "path_hint shown when INSTALL_DIR not in PATH" {
    source "${BATS_TEST_DIRNAME}/../install.sh"
    INSTALL_DIR="/some/nonexistent/path"
    run path_hint
    [ "$status" -eq 0 ]
    [[ "$output" == *"Add /some/nonexistent/path to your PATH"* ]]
}

@test "path_hint silent when INSTALL_DIR already in PATH" {
    source "${BATS_TEST_DIRNAME}/../install.sh"
    INSTALL_DIR="/usr/bin"
    run path_hint
    [ "$status" -eq 0 ]
    [ -z "$output" ]
}
