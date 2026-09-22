#!/usr/bin/env bash
# One-time developer / agent-host setup. Idempotent.
#
#   1. the pinned toolchain (rust-toolchain.toml) with rustfmt + clippy
#   2. the Linux system packages the audio and the build stack need
#   3. the cargo tools the gate requires (cargo-deny, cargo-machete, shellcheck)
#      and prefers (cargo-nextest at a pinned version, typos-cli)
#   4. the plan-db binary on PATH, so .claude/plan-coordination/db.sh is fast from
#      every worktree without a per-worktree build
#   5. the versioned git hooks
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

command -v rustup >/dev/null 2>&1 || { printf 'bootstrap: install rustup first (https://rustup.rs)\n' >&2; exit 1; }
command -v jq >/dev/null 2>&1 || printf 'bootstrap: WARN jq is missing; the Claude hooks no-op without it (brew install jq)\n' >&2

rustup show active-toolchain >/dev/null   # installs the pinned channel + components on first use

# Linux system packages. PipeWire is the audio server on Linux (section 11.5);
# ALSA supplies the client and the sequencer interface that PipeWire presents.
if [[ "$(uname -s)" == "Linux" ]]; then
  if command -v apt-get >/dev/null 2>&1; then
    sudo apt-get update
    sudo apt-get install -y --no-install-recommends \
      libpipewire-0.3-dev libasound2-dev pkg-config shellcheck
  else
    printf 'bootstrap: WARN apt-get is missing; install libpipewire-0.3-dev, libasound2-dev, pkg-config, and shellcheck by hand\n' >&2
  fi
else
  command -v shellcheck >/dev/null 2>&1 || printf 'bootstrap: WARN shellcheck is missing; the gate skips the shell lint (brew install shellcheck)\n' >&2
fi

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
install_tool typos typos-cli

# cargo-nextest is pinned, because the gate and the plan-lint job pass
# --no-tests=fail, which release 0.9.145 supports (section 14).
NEXTEST_VERSION="0.9.145"
if command -v cargo-nextest >/dev/null 2>&1 \
  && cargo nextest --version 2>/dev/null | grep -q "$NEXTEST_VERSION"; then
  printf 'bootstrap: cargo-nextest %s present\n' "$NEXTEST_VERSION"
else
  cargo install --locked --version "$NEXTEST_VERSION" cargo-nextest
fi

cargo install --locked --path tools/plan-db --force
scripts/install-hooks.sh
printf 'bootstrap: done. Run scripts/dod.sh --plan to see the gate.\n'
