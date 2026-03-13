---
title: "Plan: cct musl Static Linking for Linux Binaries"
doc_type: proc
brief: "Switch Linux CI build targets to musl for static linking, add smoke tests"
confidence: verified
created: 2026-03-13
updated: 2026-03-13
revision: 1
---

# Plan: cct musl Static Linking for Linux Binaries

## Files Changed

| File | Change Type |
|------|-------------|
| `.github/workflows/release.yml` | Major edit |

## Step 1 — Replace Linux target and add aarch64-musl in matrix

**File**: `.github/workflows/release.yml`
**What**: Replace `x86_64-unknown-linux-gnu` with two musl targets, add `use_cross` flag.

**Old**:
```yaml
          - target: x86_64-unknown-linux-gnu
            runner: ubuntu-latest
```

**New**:
```yaml
          - target: x86_64-unknown-linux-musl
            runner: ubuntu-latest
          - target: aarch64-unknown-linux-musl
            runner: ubuntu-latest
            use_cross: true
```

**Verify**: `grep -c 'linux-musl' .github/workflows/release.yml` returns 2

## Step 2 — Add musl toolchain and cross installation steps

**File**: `.github/workflows/release.yml`
**What**: Add conditional steps to install musl-tools, rustup target, and cross tool before the build step.

**Old**:
```yaml
      - name: Build release binary
        run: cargo build --release
```

**New**:
```yaml
      - name: Install musl toolchain
        if: contains(matrix.target, 'linux-musl')
        run: |
          sudo apt-get update
          sudo apt-get install -y musl-tools
          rustup target add ${{ matrix.target }}

      - name: Install cross
        if: matrix.use_cross == true
        run: cargo install cross --git https://github.com/cross-rs/cross

      - name: Build release binary
        run: |
          if [ "${{ matrix.use_cross }}" = "true" ]; then
            cross build --release --target ${{ matrix.target }}
          else
            cargo build --release --target ${{ matrix.target }}
          fi
```

**Verify**: `grep -c 'musl-tools' .github/workflows/release.yml` returns 1; `grep -c 'cross build' .github/workflows/release.yml` returns 1

## Step 3 — Update package step for target-specific binary path

**File**: `.github/workflows/release.yml`
**What**: Change binary path from `target/release/cct` to `target/${{ matrix.target }}/release/cct`.

**Old**:
```yaml
      - name: Package binary
        run: |
          cd target/release
          tar czf ../../cct-${{ matrix.target }}.tar.gz cct
          cd ../..
```

**New**:
```yaml
      - name: Package binary
        run: |
          cd target/${{ matrix.target }}/release
          tar czf ../../../cct-${{ matrix.target }}.tar.gz cct
          cd ../../..
```

**Verify**: `grep 'matrix.target.*release' .github/workflows/release.yml | grep -c 'cd target'` returns 1

## Step 4 — Add static linking verification and Ubuntu 20.04 smoke test

**File**: `.github/workflows/release.yml`
**What**: Add verification steps after packaging for Linux musl targets.

**Old**:
```yaml
      - name: Upload artifact
```

**New**:
```yaml
      - name: Verify static linking
        if: contains(matrix.target, 'linux-musl')
        run: file target/${{ matrix.target }}/release/cct | grep -i "statically linked"

      - name: Smoke test on Ubuntu 20.04
        if: matrix.target == 'x86_64-unknown-linux-musl'
        run: |
          docker run --rm \
            -v $PWD/target/${{ matrix.target }}/release/cct:/usr/local/bin/cct \
            ubuntu:20.04 cct --help

      - name: Upload artifact
```

**Verify**: `grep -c 'statically linked' .github/workflows/release.yml` returns 1; `grep -c 'ubuntu:20.04' .github/workflows/release.yml` returns 1

## Step 5 — Proof-Read End-to-End

Read `.github/workflows/release.yml` in full. Check: formatting, no leftover TODOs, spec intent preserved, YAML syntax valid.

## Step 6 — Cross-Check Acceptance Criteria

| Criterion | Addressed in Step |
|-----------|-------------------|
| `file` command confirms Linux binaries are "statically linked" | Step 4 |
| cct binary runs successfully (`cct --help`) in an Ubuntu 20.04 Docker container | Step 4 |
| CI includes a smoke test step that validates musl binary functionality | Step 4 |

All criteria map to a step.

## Step 7 — Review

Follow Phase 3 (see `03-self-review.md`). Writes `review.md`.

## Step 8 — Commit

Use /commit. Suggested message:
feat: switch Linux builds to musl for static linking
- Replace x86_64-unknown-linux-gnu with x86_64/aarch64 musl targets
- Add cross tool for aarch64 cross-compilation
- Add static linking verification and Ubuntu 20.04 smoke test

## Execution Order

Step 1 → Step 2 → Step 3 → Step 4 → Step 5 → Step 6 → Step 7 → Step 8
(Steps 1-4 are sequential edits to the same file)
