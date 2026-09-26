#!/usr/bin/env bash
# db.sh — the single store tool every Hypervisor / Orchestrator / operating agent
# calls: `.claude/plan-coordination/db.sh <cmd> <plan-dir> [args...]`.
#
# It is a thin launcher for the Rust `plan-db` binary (tools/bc_plan_store/plan-db/lang_rust), the LMDB
# plan store CLI. Subcommands, key classes, and the plan-key resolver are
# documented in tools/bc_plan_store/plan-db/lang_rust/src/main.rs and README.md in this directory.
#
# Resolution order (fast path first; every path yields the SAME store because the
# plan key is derived from git, not from where the binary lives):
#   1. `plan-db` on PATH            (scripts/bootstrap.sh installs it)
#   2. <main checkout>/target/release/plan-db
#   3. <main checkout>/target/debug/plan-db
#   4. `cargo run --release -p plan-db` from the main checkout (slow first time)
set -euo pipefail

if command -v plan-db >/dev/null 2>&1; then
  exec plan-db "$@"
fi

# The MAIN checkout root is stable across worktrees: parent of --git-common-dir.
common="$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null || true)"
if [[ -n "$common" ]]; then
  root="$(cd "$common/.." && pwd)"
else
  root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
fi

for candidate in "$root/target/release/plan-db" "$root/target/debug/plan-db"; do
  if [[ -x "$candidate" ]]; then
    exec "$candidate" "$@"
  fi
done

exec cargo run --quiet --release --manifest-path "$root/Cargo.toml" --package plan-db -- "$@"
