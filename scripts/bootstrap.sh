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
# The same set the ci.yml `dod` job installs, plus shellcheck. A developer who
# runs this script on a clean Ubuntu host must be able to build crates/duet,
# which pins gpui-kit.
LINUX_PACKAGES="libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev \
libx11-xcb-dev libxcb1-dev libxcb-shape0-dev libxcb-xfixes0-dev \
libasound2-dev libpipewire-0.3-dev libfontconfig1-dev libfreetype6-dev \
libssl-dev libgit2-dev libvulkan-dev mesa-vulkan-drivers pkg-config cmake \
clang mold shellcheck"
if [[ "$(uname -s)" == "Linux" ]]; then
  # Select the privilege escalation rather than assume it. A root container
  # carries no sudo, and `set -e` would end the bootstrap before the git hooks.
  SUDO=""
  if [ "$(id -u)" -ne 0 ]; then
    if command -v sudo >/dev/null 2>&1; then SUDO="sudo"; fi
  fi
  if ! command -v apt-get >/dev/null 2>&1; then
    printf 'bootstrap: WARN apt-get is missing; install %s by hand\n' "$LINUX_PACKAGES" >&2
  elif [ "$(id -u)" -ne 0 ] && [ -z "$SUDO" ]; then
    printf 'bootstrap: WARN sudo is missing and the user is not root; install %s by hand\n' "$LINUX_PACKAGES" >&2
  else
    # shellcheck disable=SC2086
    $SUDO apt-get update \
      && $SUDO apt-get install -y --no-install-recommends $LINUX_PACKAGES \
      || printf 'bootstrap: WARN the package install failed; install %s by hand\n' "$LINUX_PACKAGES" >&2
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
