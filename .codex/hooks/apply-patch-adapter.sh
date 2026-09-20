#!/usr/bin/env bash
# apply-patch-adapter.sh <pre|post>
#
# Translates a Codex `apply_patch` hook payload into the Claude Edit/Write payload
# shape and runs the canonical .claude/hooks scripts against each file operation.
# stdin: {"cwd": "...", "tool_input": {"command": "*** Begin Patch ... *** End Patch"}}
#
#   pre   run the PRE hooks (none registered today); exit 2 on any failure
#   post  run the POST hooks (post-edit-rustfmt.sh); always exit 0
set -euo pipefail

MODE="${1:-}"
if [[ "$MODE" != "pre" && "$MODE" != "post" ]]; then
  echo "usage: apply-patch-adapter.sh <pre|post>" >&2
  exit 2
fi
command -v jq >/dev/null 2>&1 || exit 0

INPUT=$(cat 2>/dev/null || true)
if ! printf '%s' "$INPUT" | jq -e . >/dev/null 2>&1; then
  echo "Codex hook adapter received invalid JSON." >&2
  [[ "$MODE" == "pre" ]] && exit 2 || exit 0
fi

CWD=$(printf '%s' "$INPUT" | jq -r '.cwd // empty')
CWD="${CWD:-$PWD}"
PATCH=$(printf '%s' "$INPUT" | jq -r '.tool_input.command // empty')
GIT_ROOT=$(git -C "$CWD" rev-parse --show-toplevel 2>/dev/null || printf '%s' "$CWD")

PRE_HOOKS=()
POST_HOOKS=(post-edit-rustfmt)

run_hook() { # <name> <payload-json>
  local name="$1" payload="$2"
  printf '%s' "$payload" | CLAUDE_PROJECT_DIR="$GIT_ROOT" bash "$GIT_ROOT/.claude/hooks/$name.sh"
}

# Parse the patch into "<kind>\t<path>\t<added-lines>" records (bash 3.2 safe: no mapfile).
OPS=()
while IFS= read -r rec; do
  [[ -n "$rec" ]] && OPS+=("$rec")
done < <(printf '%s\n' "$PATCH" | awk '
  function flush() { if (kind != "") { printf "%s\t%s\t%s\n", kind, path, added; kind=""; path=""; added="" } }
  /^\*\*\* (Add|Update|Delete) File: / { flush(); kind=$2; path=$0; sub(/^\*\*\* (Add|Update|Delete) File: /, "", path); next }
  /^\*\*\* Move to: / && kind != "" { path=$0; sub(/^\*\*\* Move to: /, "", path); next }
  kind != "" && kind != "Delete" && /^\+/ { line=substr($0, 2); gsub(/\\/, "\\\\", line); gsub(/"/, "\\\"", line); gsub(/\t/, "\\t", line); added = added (added == "" ? "" : "\\n") line; next }
  END { flush() }
')

if [[ ${#OPS[@]} -eq 0 ]]; then
  if [[ "$MODE" == "pre" ]]; then
    echo "Codex hook adapter could not identify a file operation in the apply_patch payload." >&2
    exit 2
  fi
  exit 0
fi

for op in "${OPS[@]}"; do
  IFS=$'\t' read -r KIND FILE ADDED <<<"$op"
  [[ "$FILE" = /* ]] || FILE="$CWD/$FILE"
  if [[ "$KIND" == "Add" ]]; then
    TOOL=Write
    PAYLOAD=$(printf '%s' "$INPUT" | jq -c --arg t "$TOOL" --arg f "$FILE" --arg c "$ADDED" '. + {tool_name:$t, tool_input:{file_path:$f, content:($c|gsub("\\\\n";"\n"))}}')
  else
    TOOL=Edit
    PAYLOAD=$(printf '%s' "$INPUT" | jq -c --arg t "$TOOL" --arg f "$FILE" --arg c "$ADDED" '. + {tool_name:$t, tool_input:{file_path:$f, new_string:($c|gsub("\\\\n";"\n"))}}')
  fi
  if [[ "$MODE" == "pre" ]]; then
    [[ "$KIND" == "Delete" ]] && continue
    for h in "${PRE_HOOKS[@]}"; do
      run_hook "$h" "$PAYLOAD" || exit 2
    done
  else
    for h in "${POST_HOOKS[@]}"; do
      run_hook "$h" "$PAYLOAD" || true
    done
  fi
done
exit 0
