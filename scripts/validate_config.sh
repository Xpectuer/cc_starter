#!/usr/bin/env bash
# validate_config.sh — Validate a cct profiles.toml file
# Usage: ./scripts/validate_config.sh [path/to/profiles.toml]
#
# Checks:
#   1. File exists and is readable
#   2. Valid TOML syntax
#   3. Every profile has a non-empty "name"
#   4. No duplicate profile names
#   5. env values are all strings
#   6. ANTHROPIC_BASE_URL looks like a URL when present
#   7. Summary table of all profiles

set -euo pipefail

CONFIG="${1:-${CCT_CONFIG:-${XDG_CONFIG_HOME:-$HOME/.config}/cc-tui/profiles.toml}}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo ""
printf "${CYAN}━━━ cct config validator ━━━${NC}\n"
printf "  File: %s\n\n" "$CONFIG"

# --- 1. File exists ---
if [[ ! -f "$CONFIG" ]]; then
    printf "${RED}  ✗${NC} File exists\n"
    printf "\n${RED}File not found. Aborting.${NC}\n"
    exit 1
fi
printf "${GREEN}  ✓${NC} File exists\n"

if [[ ! -r "$CONFIG" ]]; then
    printf "${RED}  ✗${NC} File is readable\n"
    exit 1
fi
printf "${GREEN}  ✓${NC} File is readable\n"

# --- 2-7. All checks via python ---
if ! command -v python3 &>/dev/null; then
    printf "${RED}  ✗${NC} python3 required but not found\n"
    exit 1
fi

python3 - "$CONFIG" <<'PYEOF'
import sys, json

config_path = sys.argv[1]

# Parse TOML
try:
    import tomllib
except ImportError:
    try:
        import tomli as tomllib
    except ImportError:
        print("\033[0;31m  ✗\033[0m python3 tomllib/tomli not available")
        sys.exit(1)

try:
    with open(config_path, "rb") as f:
        data = tomllib.load(f)
    print("\033[0;32m  ✓\033[0m Valid TOML syntax")
except Exception as e:
    print(f"\033[0;31m  ✗\033[0m Valid TOML syntax")
    print(f"    Parse error: {e}")
    sys.exit(1)

profiles = data.get("profiles", [])
if not profiles:
    print("\033[0;31m  ✗\033[0m No [[profiles]] entries found")
    sys.exit(1)

names = []
errors = 0
warnings = 0

for i, p in enumerate(profiles):
    idx = f"profiles[{i}]"

    # 3. name present and non-empty
    name = p.get("name", "")
    if not name or not isinstance(name, str) or not name.strip():
        print(f'\033[0;31m  ✗\033[0m {idx}: missing or empty "name"')
        errors += 1
        name = f"<unnamed-{i}>"
    else:
        print(f'\033[0;32m  ✓\033[0m {idx} "{name}": has name')

    names.append(name)

    # Check optional typed fields
    if "skip_permissions" in p and not isinstance(p["skip_permissions"], bool):
        print(f'\033[0;31m  ✗\033[0m {idx} "{name}": skip_permissions must be bool')
        errors += 1

    if "model" in p and not isinstance(p["model"], str):
        print(f'\033[0;31m  ✗\033[0m {idx} "{name}": model must be string')
        errors += 1

    if "extra_args" in p:
        ea = p["extra_args"]
        if not isinstance(ea, list) or not all(isinstance(x, str) for x in ea):
            print(f'\033[0;31m  ✗\033[0m {idx} "{name}": extra_args must be list of strings')
            errors += 1

    # 5. env values are all strings
    env = p.get("env", {})
    if env:
        for k, v in env.items():
            if not isinstance(v, str):
                print(f'\033[0;31m  ✗\033[0m {idx} "{name}": env.{k} must be string, got {type(v).__name__}')
                errors += 1

        # 6. URL check
        base_url = env.get("ANTHROPIC_BASE_URL", "")
        if base_url and not (base_url.startswith("http://") or base_url.startswith("https://")):
            print(f'\033[0;33m  ⚠\033[0m {idx} "{name}": ANTHROPIC_BASE_URL does not start with http(s)://')
            warnings += 1

        # token present?
        if not env.get("ANTHROPIC_AUTH_TOKEN") and not env.get("ANTHROPIC_API_KEY"):
            print(f'\033[0;33m  ⚠\033[0m {idx} "{name}": no auth token/key in env (may use default)')
            warnings += 1

# 4. Duplicate names
seen = set()
for n in names:
    if n in seen:
        print(f'\033[0;31m  ✗\033[0m Duplicate profile name: "{n}"')
        errors += 1
    seen.add(n)

if not errors:
    print(f"\033[0;32m  ✓\033[0m No duplicate names")

# --- Summary table ---
print()
print("\033[0;36m━━━ Profile Summary ━━━\033[0m")
header = f'  {"Name":<20} {"Model":<25} {"Base URL":<40} {"Skip Perms"}'
print(header)
sep = f'  {"─"*20} {"─"*25} {"─"*40} {"─"*10}'
print(sep)
for p in profiles:
    name = p.get("name", "???")
    model = p.get("model", "-")
    env = p.get("env", {})
    url = env.get("ANTHROPIC_BASE_URL", "(default)")
    if len(url) > 38:
        url = url[:35] + "..."
    skip = "yes" if p.get("skip_permissions") else "-"
    print(f"  {name:<20} {model:<25} {url:<40} {skip}")

print()
total = len(profiles)
print("\033[0;36m━━━ Result ━━━\033[0m")
print(f"  Profiles: {total}   Errors: {errors}   Warnings: {warnings}")
if errors:
    print("\033[0;31m  FAIL\033[0m")
    sys.exit(1)
else:
    print("\033[0;32m  PASS\033[0m")
PYEOF

echo ""
