## Step 1 — Replace linux-gnu target with two musl targets in release workflow

### Actions Taken

- Edited `.github/workflows/release.yml` matrix section.
- Replaced `x86_64-unknown-linux-gnu` entry with two new entries:
  - `x86_64-unknown-linux-musl` on `ubuntu-latest`
  - `aarch64-unknown-linux-musl` on `ubuntu-latest` with `use_cross: true`

### Verify Result

```
$ grep -c 'linux-musl' .github/workflows/release.yml
2
```

Result: **SUCCESS** — both musl targets present in the workflow matrix.
