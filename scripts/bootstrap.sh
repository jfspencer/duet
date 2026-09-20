#!/usr/bin/env bash
# One-time developer / agent-host setup. Idempotent.
#
#   1. the pinned toolchain (rust-toolchain.toml) with rustfmt + clippy
#   2. the cargo tools the gate requires (cargo-deny, cargo-machete) and prefers
#      (cargo-nextest, typos-cli)
#   3. the plan-db binary on PATH, so .claude/plan-coordination/db.sh is fast from
#      every worktree without a per-worktree build
#   4. the versioned git hooks
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

command -v rustup >/dev/null 2>&1 || { printf 'bootstrap: install rustup first (https://rustup.rs)\n' >&2; exit 1; }
command -v jq >/dev/null 2>&1 || printf 'bootstrap: WARN jq is missing; the Claude hooks no-op without it (brew install jq)\n' >&2

rustup show active-toolchain >/dev/null   # installs the pinned channel + components on first use

install_tool() {
  local bin="$1" crate="$2"
  if command -v "$bin" >/dev/null 2>&1; then
    printf 'bootstrap: %s present\n' "$bin"
  else
    cargo install --locked "$crate"
  fi
}
install_tool cargo-deny cargo-deny
install_tool cargo-machete cargo-machete
install_tool cargo-nextest cargo-nextest
install_tool typos typos-cli

cargo install --locked --path tools/plan-db --force
scripts/install-hooks.sh
printf 'bootstrap: done. Run scripts/dod.sh --plan to see the gate.\n'
