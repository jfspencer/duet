#!/usr/bin/env bash
# Run PP25, the roster probe of architecture section 1.9, and the three
# planted gate defects that section records beside it.
#
# PG25 is the one rule that needs a compiler, so the placement harness cannot
# reproduce its result. Revision 12 recorded a PP25 cell that no run of that
# revision produced, and both of its shapes were false (critic CR-13). This
# script is the run.
#
# usage: probe_roster.sh <architecture.md> <scratch-dir> <repo-root>
#
# It plants one defect per shape in a throwaway copy of the document, runs
# `roster_compile.sh` over the copy, and prints the exit code and the first
# compiler line. It exits 0 when every planted shape is red, 1 when one is not,
# and 2 on a usage or input failure. Every `cargo` invocation happens inside
# the scratch directory, which must sit outside the repository.
#
# **It also compares the recorded text of the PG25 row with its own runs**
# through `probe_roster_text.py` (critic C17-7). Revision 17 gave that machine
# to `probe_run.py` and exempted PP25 by name, which is the one row whose
# false record caused CR-13 and C16-1. The exemption's reason was that PP25
# needs a compiler; this script runs the compiler, so this script owns the
# compare.
set -uo pipefail

if [ "$#" -ne 3 ]; then
    printf 'usage: probe_roster.sh <architecture.md> <scratch-dir> <repo-root>\n'
    exit 2
fi

DOCUMENT="$1"
SCRATCH="$2"
REPO="$3"
HERE="$(cd "$(dirname "$0")" && pwd)"

if [ ! -f "$DOCUMENT" ]; then
    printf 'FAIL: cannot open %s\n' "$DOCUMENT"
    exit 2
fi
mkdir -p "$SCRATCH/copies" || exit 2

# ONE cargo target for the whole run, and it does not outlive the run. The
# eleven shapes below each call `roster_compile.sh`, and a target per shape
# would be eleven copies of the whole `gpui` dependency tree.
#
# THE TRAP COVERS ONLY THE TARGET THIS SCRIPT CREATED (critic C21-W4).
# Revision 21 set the trap whatever the source of the value was, so a caller
# that exported `ROSTER_TARGET_DIR` had its own shared target deleted by a
# script it invoked, against the ownership contract section 1.9 states. The
# twenty-first Critic lost its shared target to exactly that trap.
if [ -n "${ROSTER_TARGET_DIR:-}" ]; then
    case "$ROSTER_TARGET_DIR" in
        /*) ;;
        *)
            printf 'FAIL: ROSTER_TARGET_DIR must be an absolute path; the harness is fail-closed.\n'
            exit 2
            ;;
    esac
    # THE COMPARE IS ON THE RESOLVED PATH (critic C22I-W6). Revision 22 tested
    # the exported string against the repository string with a shell `case`,
    # so an absolute path OUTSIDE the repository that RESOLVED inside it
    # passed every arm and the value went on to an `rm -rf`. CG7 already
    # records that a symbolic link defeats a path exemption, and the
    # conversion guard resolves; these two scripts did not take that lesson.
    if [ -d "$ROSTER_TARGET_DIR" ]; then
        ROSTER_REAL=$(cd "$ROSTER_TARGET_DIR" && pwd -P)
    else
        ROSTER_REAL=$(cd "$(dirname "$ROSTER_TARGET_DIR")" 2>/dev/null && pwd -P)/$(basename "$ROSTER_TARGET_DIR")
    fi
    REPO_REAL=$(cd "$REPO" 2>/dev/null && pwd -P)
    case "$ROSTER_REAL" in
        "$REPO_REAL"|"$REPO_REAL"/*)
            printf 'FAIL: ROSTER_TARGET_DIR must sit outside %s; the harness is fail-closed.\n' "$REPO"
            exit 2
            ;;
    esac
    export ROSTER_TARGET_DIR
else
    export ROSTER_TARGET_DIR="$SCRATCH/target"
    trap 'rm -rf "$ROSTER_TARGET_DIR"' EXIT INT TERM
fi

BAD=0

# Plant one shape and run the roster over the result.
# $1 shape id, $2 expected exit code, $3 python mutation source.
shape() {
    local id="$1" want="$2" code="$3" copy="$SCRATCH/copies/$1.md" out="$SCRATCH/copies/$1.out"
    if ! SHAPE_SOURCE="$DOCUMENT" SHAPE_TARGET="$copy" python3 -c "$code"; then
        printf '%-12s PLANT FAILED\n' "$id"
        BAD=$((BAD + 1))
        return
    fi
    bash "$HERE/roster_compile.sh" "$copy" "$SCRATCH/ws" "$REPO" > "$out" 2>&1
    local got=$?
    local line
    line=$(grep -m1 -E '^(duet|error|FAIL)' "$out" | cut -c1-150)
    if [ "$got" = "$want" ]; then
        printf '%-12s exit %s  OK   %s\n' "$id" "$got" "$line"
    else
        printf '%-12s exit %s  BAD  expected %s; %s\n' "$id" "$got" "$want" "$line"
        BAD=$((BAD + 1))
    fi
}

PLANT='
import os
old, new = OLD, NEW
text = open(os.environ["SHAPE_SOURCE"], encoding="utf-8").read()
assert text.count(old) == 1, text.count(old)
open(os.environ["SHAPE_TARGET"], "w", encoding="utf-8").write(text.replace(old, new, 1))
'

plant() {
    printf 'OLD = %s\nNEW = %s\n%s' "$1" "$2" "$PLANT"
}

printf 'BASE\n'
bash "$HERE/roster_compile.sh" "$DOCUMENT" "$SCRATCH/ws" "$REPO" > "$SCRATCH/copies/base.out" 2>&1
BASE=$?
grep -E '^(ROSTER|RECORDED)' "$SCRATCH/copies/base.out"
printf 'BASE         exit %s\n' "$BASE"
if [ "$BASE" != "0" ]; then
    printf 'FAIL: the baseline roster run is not green, so no shape is meaningful.\n'
    exit 1
fi

# PP25 shape 1: an `Eq` derive removed where no container demands `Eq`, so the
# `nursery` lint `derive_partial_eq_without_eq` fires rather than a type error.
shape PP25-eq 1 "$(plant \
  "'#[derive(Debug, Clone, PartialEq, Eq, Error)]\npub enum AgentError'" \
  "'#[derive(Debug, Clone, PartialEq, Error)]\npub enum AgentError'")"

# PP25 shape 2: a name leaves the section 1.9 Drop block, so substitution S8
# emits no `Drop` impl for it and `missing_copy_implementations` fires on an
# all-`Copy` handle (critic WR-12).
shape PP25-drop 1 "$(plant \
  "'HotplugSubscription\nMidiInputHandle'" \
  "'MidiSink\nMidiInputHandle'")"

# PP25 shape 3: the `RenderJob` declaration deleted, which shrinks the roster's
# denominator. Revision 13 exited 0 on this shape (critic C-3).
shape PP25-floor 1 "$(plant \
  "'pub struct RenderJob {'" \
  "'pub struct RenderJobProbeRenamed {'")"

# The three planted gate defects. Each one is a defect the repository lint
# policy refuses, and PG25 is the rule that catches it inside this document.
shape GATE-allow 1 "$(plant \
  "'#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]\npub enum Accidental'" \
  "'#[allow(dead_code)]\n#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]\npub enum Accidental'")"

shape GATE-unwrap 1 "$(plant \
  "'        assert!(value.is_finite(), \"a \`Finite\` constant must be finite\");'" \
  "'        let _probe = Some(value).unwrap();\n        assert!(value.is_finite(), \"a \`Finite\` constant must be finite\");'")"

shape GATE-as 1 "$(plant \
  "'    pub const fn get(self) -> f64 { self.0 }'" \
  "'    pub const fn get(self) -> f64 { let _probe = self.0 as f32; self.0 }'")"

# PP25 shape 4: the impl floor (concern N-3). Revision 14 deleted
# `impl Global for DuetTokens {}` and the roster exited 0 with both impl
# counts one lower, because each one came from the same parse the deletion
# shrank. The `impl-sites` block is the second source, so the same plant is
# now red.
shape PP25-impl 1 "$(plant \
  "'\nimpl Global for DuetTokens {}\n'" \
  "'\n'")"

# Three gate defects at three sites revision 15 wrote, so the plant sits in
# text no earlier revision held.
shape GATE15-allow 1 "$(plant \
  "'#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum ChainSource'" \
  "'#[allow(dead_code)]\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum ChainSource'")"

shape GATE15-unwrap 1 "$(plant \
  "'        f.debug_struct(\"GraphConfigurator\").finish_non_exhaustive()'" \
  "'        let _probe = Some(1_u8).unwrap();\n        f.debug_struct(\"GraphConfigurator\").finish_non_exhaustive()'")"

shape GATE15-as 1 "$(plant \
  "'        f.debug_struct(\"ChainSetHandle\").finish_non_exhaustive()'" \
  "'        let _probe = 1_i64 as u16;\n        f.debug_struct(\"ChainSetHandle\").finish_non_exhaustive()'")"

# Three gate defects planted at the declarations REVISION 17 wrote, so the
# probe set tracks the revision and not only the sites of two revisions ago.
# `GraphConfigurator::backend` closed critic C16-7, `ConfigError::NothingToHandOff`
# closed C16-3, and `MeterReader::caps` and `holds` closed C16-8.
shape GATE17-allow 1 "$(plant \
  "'    backend: Box<dyn AudioBackend>,'" \
  "'    #[allow(dead_code)]\n    backend: Box<dyn AudioBackend>,'")"

shape GATE17-unwrap 1 "$(plant \
  "'        f.debug_struct(\"GraphConfigurator\").finish_non_exhaustive()'" \
  "'        let _probe = Option::<u8>::None.unwrap_or_default();\n        let _second = Some(2_u8).unwrap();\n        f.debug_struct(\"GraphConfigurator\").finish_non_exhaustive()'")"

shape GATE17-as 1 "$(plant \
  "'            live: 0,'" \
  "'            live: 0_i64 as u16,'")"

# A MIXED impl block: one body beside one bodiless signature (concern N17-4).
# The whole block leaves the roster today and no counter moves, so the roster
# refuses the shape rather than dropping it.
shape PP25-mixed 1 "$(plant \
  "'    pub fn apply(&mut self, command: TransportCommand) -> Result<TransportState, TransportError>;'" \
  "'    pub fn probe(&self) -> u8 { 0 }\n    pub fn apply(&mut self, command: TransportCommand) -> Result<TransportState, TransportError>;'")"

# THE THREE `ROSTER_TARGET_DIR` SHAPES (critic C21-W4, C22I-W6). The value
# reaches an `rm -rf`, so the guard validates it, and the third shape is the
# one revision 22 did not refuse: an absolute path outside the repository that
# RESOLVES inside it. Each shape runs `roster_compile.sh` with one exported
# value and asserts exit 2 and the line it prints.
env_shape() {
    local id="$1" value="$2" want="$3" out="$SCRATCH/copies/$1.out"
    ROSTER_TARGET_DIR="$value" bash "$HERE/roster_compile.sh" \
        "$DOCUMENT" "$SCRATCH/ws" "$REPO" > "$out" 2>&1
    local got=$?
    local line
    line=$(grep -m1 -E '^FAIL' "$out" | cut -c1-150)
    if [ "$got" = "2" ] && printf '%s' "$line" | grep -q "$want"; then
        printf '%-12s exit %s  OK   %s\n' "$id" "$got" "$line"
    else
        printf '%-12s exit %s  BAD  expected 2 and %s; %s\n' "$id" "$got" "$want" "$line"
        BAD=$((BAD + 1))
    fi
}

env_shape PP25-relative "target" "must be an absolute path"
env_shape PP25-inside "$REPO/target" "must sit outside"
ln -sfn "$REPO" "$SCRATCH/link-target"
env_shape PP25-symlink "$SCRATCH/link-target" "must sit outside"

# The recorded-text compare of the PG25 row (critic C17-7). Every shape above
# wrote its own `.out` file, and this step asks whether each fragment the
# section 1.9 PG25 cell records appears in one of them. `probe_roster.sh` runs
# the compiler, so `probe_roster.sh` owns this compare.
python3 "$HERE/probe_roster_text.py" "$DOCUMENT" "$SCRATCH/copies"
FRAGBAD=$?

printf 'ROSTER PROBES BAD: %s     FRAGMENTS BAD: %s\n' "$BAD" "$FRAGBAD"
[ "$BAD" = 0 ] || exit 1
[ "$FRAGBAD" = 0 ] || exit 1
exit 0
