#!/usr/bin/env bash
# PostToolUse hook (Edit|Write): format the file that was just written.
#
# *.rs        -> rustfmt with the repo rustfmt.toml (in place)
# *.toml      -> taplo fmt when taplo is installed
#
# Advisory: exits 0 always, even when rustfmt cannot parse the file (the commit
# gate `cargo fmt --all --check` is the blocking surface). Silent when jq is
# missing.
set -euo pipefail

command -v jq >/dev/null 2>&1 || exit 0
INPUT=$(cat 2>/dev/null || true)
FILE=$(printf '%s' "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null || true)
[[ -n "$FILE" && -f "$FILE" ]] || exit 0

ROOT="${CLAUDE_PROJECT_DIR:-$(git -C "$(dirname "$FILE")" rev-parse --show-toplevel 2>/dev/null || pwd)}"

case "$FILE" in
  *.rs)
    if command -v rustfmt >/dev/null 2>&1; then
      rustfmt --edition 2024 --config-path "$ROOT/rustfmt.toml" "$FILE" 2>&1 \
        | sed 's/^/post-edit-rustfmt: /' >&2 || true
    fi
    ;;
  *.toml)
    if command -v taplo >/dev/null 2>&1; then
      taplo fmt "$FILE" >/dev/null 2>&1 || true
    fi
    ;;
esac
exit 0
