#!/usr/bin/env bash
# Point git at the versioned hooks. Run once per clone (scripts/bootstrap.sh does it).
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"
chmod +x .githooks/* scripts/*.sh .claude/hooks/*.sh .claude/plan-coordination/*.sh .codex/hooks/*.sh 2>/dev/null || true
git config core.hooksPath .githooks
printf 'core.hooksPath -> .githooks (pre-commit and pre-push run scripts/dod.sh)\n'
