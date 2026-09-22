#!/usr/bin/env bash
# scripts/dod.sh — the ONLY Definition of Done gate.
#
# The native git hooks (.githooks/pre-commit, .githooks/pre-push) exec this
# script, and CI runs it verbatim. No Claude hook runs the DoD, and no agent
# hand-runs it as a ritual: make the change, `git commit`, and the hook runs it.
#
#   scripts/dod.sh          run every gate
#   scripts/dod.sh --plan   print the gate list and exit 0 (read-only dry run)
#
# Required tools: rustup (rust-toolchain.toml pins the channel), cargo-deny,
# and cargo-machete. Optional tools: cargo-nextest (the preferred test
# runner), typos, and shellcheck. `scripts/bootstrap.sh` installs the set.
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

GATES=(
  "fmt        cargo fmt --all --check"
  "clippy     cargo clippy --workspace --all-targets --locked -- -D warnings"
  "doc        cargo doc --workspace --no-deps --document-private-items --locked"
  "test       cargo nextest run --workspace --locked (or cargo test --workspace --locked)"
  "doctest    cargo test --doc --workspace --locked (when a library target exists)"
  "deny       cargo deny check"
  "machete    cargo machete"
  "agents     cargo xtask sync-agents --check"
  "converts   cargo xtask check-conversions (CG9 when the change touches roadmap/ or convert.rs)"
  "manifests  cargo xtask check-manifests"
  "plan       cargo xtask check-placement and check-plan-graph (when the change touches roadmap/)"
  "hooks      bash -n on every shell hook and script (shellcheck when installed)"
  "typos      typos (when installed)"
)

if [[ "${1:-}" == "--plan" ]]; then
  printf 'Definition of Done gates (scripts/dod.sh):\n'
  for g in "${GATES[@]}"; do printf '  %s\n' "$g"; done
  exit 0
fi

step() { printf '\n\033[1m==> %s\033[0m\n' "$*"; }
need() {
  command -v "$1" >/dev/null 2>&1 || {
    printf 'dod: required tool "%s" is missing. Run scripts/bootstrap.sh.\n' "$1" >&2
    exit 1
  }
}

need cargo
need cargo-deny
need cargo-machete

# The change under test, as ONE union that two steps read. Section 14 rung one
# of roadmap/duet-v1/architecture.md states the three clauses. Clause 3 is why
# the step is reachable at a push and at an amend, where the first two clauses
# are empty. The union is computed here, above every step, because the first
# reader of it is the `converts` step and not the `plan` step.
#
# `gitdiff` prints one changed path per line, whatever bytes the path holds. A
# condition that cannot see its own input must not answer no, and a plain
# `--name-only` cannot see four shapes. `-z` prints each path RAW and NUL
# separated, so nothing is quoted and no byte is escaped; `core.quotePath=false`
# alone covers the non-ASCII case and leaves a quotation mark, a backslash, a
# tab and a newline quoted, and a quoted path fails every anchored pattern
# below. `--no-renames` prints BOTH sides of a rename, so a document moved OUT
# of `roadmap/` is still seen; with renames on, git prints the destination
# alone and the step skips although the plan directory lost a file.
#
# A path that holds a newline prints as two lines here. That can only ADD a
# match and never remove one, so the condition stays fail-safe in the one
# direction that matters.
gitdiff() {
  git -c core.quotePath=false diff --no-renames -z "$@" | tr '\0' '\n'
}

CHANGED_PATHS=""
DENOMINATOR_UNKNOWN=0
if git rev-parse --verify --quiet HEAD >/dev/null 2>&1; then
  CHANGED_PATHS="$(gitdiff --cached --name-only)"$'\n'"$(gitdiff --name-only HEAD)"
  if git rev-parse --verify --quiet '@{upstream}' >/dev/null 2>&1; then
    CHANGED_PATHS="$CHANGED_PATHS"$'\n'"$(gitdiff --name-only '@{upstream}..HEAD')"
  elif merge_base="$(git merge-base origin/main HEAD 2>/dev/null)"; then
    CHANGED_PATHS="$CHANGED_PATHS"$'\n'"$(gitdiff --name-only "$merge_base..HEAD")"
  else
    DENOMINATOR_UNKNOWN=1
  fi
else
  DENOMINATOR_UNKNOWN=1
fi

# Whether the change under test names a path one pattern matches. A run that
# cannot compute its denominator answers YES, so a step never skips on a
# denominator it does not hold.
touches() {
  if [[ "$DENOMINATOR_UNKNOWN" == "1" ]]; then
    return 0
  fi
  printf '%s\n' "$CHANGED_PATHS" | grep -qE "$1"
}

step "fmt: cargo fmt --all --check"
cargo fmt --all --check

step "clippy: cargo clippy --workspace --all-targets --locked -- -D warnings"
cargo clippy --workspace --all-targets --locked -- -D warnings

step "doc: cargo doc --workspace --no-deps --document-private-items --locked"
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items --locked

if command -v cargo-nextest >/dev/null 2>&1; then
  step "test: cargo nextest run --workspace --locked"
  cargo nextest run --workspace --locked --profile "${NEXTEST_PROFILE:-default}"
else
  step "test: cargo test --workspace --locked (install cargo-nextest for the faster runner)"
  cargo test --workspace --locked
fi

# `cargo test --doc` errors when no crate has a library target; doctests only exist on libs.
if cargo metadata --no-deps --format-version 1 | grep -qE '"kind":\["(lib|rlib|proc-macro)"\]'; then
  step "doctest: cargo test --doc --workspace --locked"
  cargo test --doc --workspace --locked
else
  step "doctest: skipped (no library targets in the workspace)"
fi

step "deny: cargo deny check"
cargo deny check

step "machete: cargo machete"
cargo machete

step "agents: cargo xtask sync-agents --check"
cargo xtask sync-agents --check

if touches '^roadmap/|^crates/duet-time/src/convert\.rs$'; then
  step "conversions: cargo xtask check-conversions --appendix roadmap/duet-v1/architecture.md"
  cargo xtask check-conversions --appendix roadmap/duet-v1/architecture.md
else
  step "conversions: cargo xtask check-conversions"
  cargo xtask check-conversions
fi

step "manifests: cargo xtask check-manifests"
cargo xtask check-manifests

if touches '^roadmap/'; then
  step "plan: cargo xtask check-placement and check-plan-graph"
  cargo xtask check-placement roadmap/duet-v1/architecture.md
  cargo xtask check-plan-graph roadmap/duet-v1
  cargo xtask check-plan-graph roadmap/duet-v1 --check-manifest
else
  step "plan guards: skipped (no change under roadmap/)"
fi

step "hooks: bash -n"
while IFS= read -r -d '' f; do
  bash -n "$f"
done < <(find .claude/hooks .codex/hooks .claude/plan-coordination scripts .githooks -type f \( -name '*.sh' -o -name 'pre-commit' -o -name 'pre-push' -o -name 'commit-msg' \) -print0 2>/dev/null)
if command -v shellcheck >/dev/null 2>&1; then
  find .claude/hooks .codex/hooks .claude/plan-coordination scripts -type f -name '*.sh' -print0 2>/dev/null \
    | xargs -0 shellcheck -x -S warning
fi

if command -v typos >/dev/null 2>&1; then
  step "typos"
  typos
else
  step "typos: skipped (typos is not installed; run scripts/bootstrap.sh)"
fi

printf '\n\033[1;32mDefinition of Done: every gate passed.\033[0m\n'
