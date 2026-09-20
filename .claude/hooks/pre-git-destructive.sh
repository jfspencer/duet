#!/usr/bin/env bash
# Pre-Bash hook: refuse destructive git operations and pull request merges.
#
# Blocks (exit 2):
#   - `git push --force` WITHOUT `--force-with-lease`.
#   - `git commit --no-verify`.
#   - `gh pr merge`, and a `gh api` call that reaches a pull request merge
#     route or a write to the git ref of main.
#   - `git merge` / `git pull` that lands a non-main ref on main.
#   - `git push` whose destination resolves to main, `--all` and `--mirror`
#     included.
#
# Advises (exit 0 with stderr WARNING):
#   - `git reset --hard`.
#
# THE LIMIT OF THIS HOOK. It runs inside Claude's Bash tool only. A human shell
# is unaffected. It reads argv, so a body that argv does not carry is invisible
# to it: `eval "..."`, `bash -c "..."`, a `bash <<MARKER` body, and a refspec
# whose source is a command substitution (the segmenter splits that line, so
# `git push origin $(...):main` loses its destination). A git global option that
# takes a SEPARATE value and is not in the list below shifts the verb out of
# reach. This is a guard against an accidental act, not a security boundary.
# Every refusal states this limit, because a guard that overstates its reach is
# worse than one that states it.
#
# KNOWN FALSE POSITIVE, accepted deliberately. A redirect heredoc body is raw
# text on the line, so a body that only DOCUMENTS a blocked command is read as
# argv and refused. An earlier version of this file stripped those bodies and
# that strip was a total bypass: one line carrying the marker pattern in a
# comment or inside quotes deleted every later line, and all five refusals then
# did nothing. A cry-wolf cost is not a bypass, so the strip is gone. Write
# documentation with a file-write tool rather than a heredoc.
#
# Banned by policy but NOT blocked here:
#   - `git push --no-verify`. Root CLAUDE.md bans --no-verify on push too, but
#     that half is a SOCIAL rule with no mechanism: this hook does not inspect
#     `git push` for it, and no quality gate is wired to pre-push anyway (the
#     only pre-push hook is a git-lfs shim). Do not add a pre-push gate here on
#     the strength of this comment; that is an operator decision, unmade.
#   - `git rebase`, `git cherry-pick`, `git am` onto main. They rewrite LOCAL
#     main only. Refusing them would cry wolf on ordinary work. This is accepted
#     debt and NOT an argument that the result cannot be published: the push
#     rules below close the push paths they can read from argv, and the header
#     limits above name the ones they cannot.
#
# Every detector is TOKEN-level, never a substring scan. An earlier substring
# version fired on a `sed` whose PATTERN held the phrase. A gate that cries wolf
# on prose is how a team learns to bypass it. The negative cases in
# tools/claude-hooks/__e2e__/ hold that line.
#
# Triggered by .claude/settings.json on PreToolUse + Bash.

set -euo pipefail

__HOOK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=_lib-tokenize.sh
source "$__HOOK_DIR/_lib-tokenize.sh"

if ! command -v jq >/dev/null 2>&1; then
  exit 0
fi

INPUT=$(cat 2>/dev/null || true)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null || true)
PAYLOAD_CWD=$(echo "$INPUT" | jq -r '.cwd // empty' 2>/dev/null || true)

if [[ -z "$COMMAND" ]] || ! echo "$COMMAND" | grep -qE '\b(git|gh)\b'; then
  exit 0
fi

# __claude_hook_split_commands ends a segment at every newline and ignores quote
# state, while __claude_hook_tokenize carries a quoted run across a newline. The
# segmenter runs first, so each line of a MULTI-LINE quoted argument would be
# classified as its own command, and prose that only names a blocked command
# inside such an argument would be refused. That is the shape this fleet emits
# when it files a report through the plan store. Joining the newlines inside a
# quoted run keeps the whole argument in one segment and one token.
__claude_hook_join_quoted_newlines() {
  awk '
    BEGIN { in_single = 0; in_double = 0; out = ""; }
    {
      line = $0;
      n = length(line);
      for (i = 1; i <= n; i++) {
        c = substr(line, i, 1);
        if (in_single) {
          out = out c;
          if (c == "\x27") { in_single = 0; }
        } else if (in_double) {
          if (c == "\\" && i < n) { out = out c substr(line, i + 1, 1); i++; }
          else { out = out c; if (c == "\"") { in_double = 0; } }
        } else {
          if (c == "\x27") { in_single = 1; out = out c; }
          else if (c == "\"") { in_double = 1; out = out c; }
          else if (c == "\\" && i < n) { out = out c substr(line, i + 1, 1); i++; }
          else { out = out c; }
        }
      }
      if (in_single || in_double) { out = out " "; }
      else { print out; out = ""; }
    }
    END { if (out != "") print out; }
  '
}

COMMAND=$(printf '%s\n' "$COMMAND" | __claude_hook_join_quoted_newlines)

# git accepts global options BEFORE the verb. Reading the verb at a fixed offset
# misses `git -C <path> push --force`, which reached exit 0 before this rule
# existed. An unrecognised hyphen-leading token advances by one rather than
# ending the scan, because git owns this option set and adds to it: a scan that
# STOPS on an unknown option fails open on every future git release.
# Sets VERB_IDX and GIT_C_DIR.
__claude_hook_git_verb_idx() {
  local i=$((START_IDX + 1))
  GIT_C_DIR=""
  while [[ $i -lt ${#TOKENS[@]} ]]; do
    case "${TOKENS[$i]}" in
      -C)
        if [[ $((i + 1)) -lt ${#TOKENS[@]} ]]; then GIT_C_DIR="${TOKENS[$((i + 1))]}"; fi
        i=$((i + 2))
        ;;
      -c|--git-dir|--work-tree|--namespace|--exec-path|--super-prefix|--config-env|--attr-source)
        i=$((i + 2))
        ;;
      -*)
        i=$((i + 1))
        ;;
      *)
        break
        ;;
    esac
  done
  VERB_IDX=$i
}

# A leading plus sign marks a forced refspec. It changes nothing about WHERE the
# push lands, so it is removed before the comparison.
__claude_hook_resolves_to_main() {
  local ref="${1#+}"
  case "$ref" in
    main|refs/heads/main|'@{u}'|'@{upstream}') return 0 ;;
    */main) return 0 ;;
  esac
  return 1
}

# The lookup runs on the PreToolUse hot path, so it is memoized. It must never
# abort the hook: under `set -euo pipefail` a bare substitution against a missing
# directory exits 128. An inherited GIT_DIR would silently answer for a different
# repository, so the location variables are cleared. The cap needs `timeout` or
# `gtimeout`; where neither exists the call runs uncapped, which is the lesser
# harm against disabling the rule on every host that ships no coreutils timeout.
__BRANCH_CACHE_DIR=""
__BRANCH_CACHE_VAL=""
__claude_hook_branch_of() {
  local dir="${1:-}"
  if [[ -z "$dir" ]]; then dir="."; fi
  if [[ "$dir" == "$__BRANCH_CACHE_DIR" ]]; then
    printf '%s' "$__BRANCH_CACHE_VAL"
    return 0
  fi
  local out=""
  local cap=""
  if command -v timeout >/dev/null 2>&1; then cap="timeout"
  elif command -v gtimeout >/dev/null 2>&1; then cap="gtimeout"
  fi
  if [[ -n "$cap" ]]; then
    out=$(env -u GIT_DIR -u GIT_WORK_TREE -u GIT_INDEX_FILE \
      "$cap" 2 git -C "$dir" rev-parse --abbrev-ref HEAD 2>/dev/null || true)
  else
    out=$(env -u GIT_DIR -u GIT_WORK_TREE -u GIT_INDEX_FILE \
      git -C "$dir" rev-parse --abbrev-ref HEAD 2>/dev/null || true)
  fi
  __BRANCH_CACHE_DIR="$dir"
  __BRANCH_CACHE_VAL="$out"
  printf '%s' "$out"
}

__claude_hook_limit_note() {
  echo "" >&2
  echo "  LIMIT: this hook runs inside Claude's Bash tool only. A human shell is" >&2
  echo "  unaffected, and an eval or a bash -c body is not inspected." >&2
}

__claude_hook_refuse_pr_merge() {
  echo "=== BLOCKED: pull request merge ===" >&2
  echo "" >&2
  echo "  An agent never merges a pull request. A person merges it." >&2
  echo "  Open the pull request, drive it green, and hand it over." >&2
  echo "  $1" >&2
  __claude_hook_limit_note
  exit 2
}

__claude_hook_refuse_push_to_main() {
  echo "=== BLOCKED: push to main ===" >&2
  echo "" >&2
  echo "  An agent never writes remote main. Push the branch and open a pull request." >&2
  echo "  A person merges the pull request." >&2
  echo "  $1" >&2
  __claude_hook_limit_note
  exit 2
}

EFFECTIVE_CWD="$PAYLOAD_CWD"
if [[ -z "$EFFECTIVE_CWD" ]]; then EFFECTIVE_CWD="${CLAUDE_PROJECT_DIR:-.}"; fi

__claude_hook_resolve_dir() {
  case "$1" in
    /*) printf '%s' "$1" ;;
    *) printf '%s/%s' "$EFFECTIVE_CWD" "$1" ;;
  esac
}

while IFS= read -r __seg; do
TOKENS=()
while IFS= read -r __tok; do
  TOKENS+=("$__tok")
done < <(printf '%s\n' "$__seg" | __claude_hook_tokenize)
[[ ${#TOKENS[@]} -gt 0 ]] || continue
__claude_hook_strip_env_vars TOKENS
[[ $START_IDX -lt ${#TOKENS[@]} ]] || continue

# A directory change moves the repository that a later segment acts on. Without
# this the branch lookup answers for the session directory, which in a
# multi-worktree fleet is a different repository on a different branch.
case "${TOKENS[$START_IDX]}" in
  cd|pushd)
    __cd_i=$((START_IDX + 1))
    while [[ $__cd_i -lt ${#TOKENS[@]} ]]; do
      case "${TOKENS[$__cd_i]}" in
        --) __cd_i=$((__cd_i + 1)); continue ;;
        -*) __cd_i=$((__cd_i + 1)); continue ;;
      esac
      break
    done
    if [[ $__cd_i -lt ${#TOKENS[@]} ]]; then
      EFFECTIVE_CWD="$(__claude_hook_resolve_dir "${TOKENS[$__cd_i]}")"
    fi
    ;;
esac

# --- 1. a pull request merge through any gh route  -> BLOCK ---
#
# The scan opens at the segment head, or behind a wrapper that runs its
# argument. Opening it at ANY token would let a search whose terms happen to be
# the verb pair trip the gate, which is the cry-wolf class this file refuses.
__GH_IDX=-1
__k=$START_IDX
while [[ $__k -lt ${#TOKENS[@]} ]]; do
  if [[ "${TOKENS[$__k]##*/}" == "gh" ]]; then
    __GH_IDX=$__k
    break
  fi
  case "${TOKENS[$__k]##*/}" in
    command|xargs|env|nohup|time|sudo|stdbuf|doas) __k=$((__k + 1)) ;;
    -*) __k=$((__k + 1)) ;;
    *) break ;;
  esac
done

# The GitHub CLI accepts a persistent flag before its subcommand, so the
# subcommand is the first BARE word, not the next token.
__claude_hook_gh_next_word() {
  local i=$1
  while [[ $i -lt ${#TOKENS[@]} ]]; do
    case "${TOKENS[$i]}" in
      *=*) case "${TOKENS[$i]}" in -*) i=$((i + 1)); continue ;; esac ;;
      -*) i=$((i + 2)); continue ;;
    esac
    break
  done
  GH_WORD_IDX=$i
}

if [[ $__GH_IDX -ge 0 ]]; then
  __claude_hook_gh_next_word $((__GH_IDX + 1))
  __gh_sub=""
  if [[ $GH_WORD_IDX -lt ${#TOKENS[@]} ]]; then __gh_sub="${TOKENS[$GH_WORD_IDX]}"; fi

  if [[ "$__gh_sub" == "pr" ]]; then
    __claude_hook_gh_next_word $((GH_WORD_IDX + 1))
    if [[ $GH_WORD_IDX -lt ${#TOKENS[@]} ]] && [[ "${TOKENS[$GH_WORD_IDX]}" == "merge" ]]; then
      __claude_hook_refuse_pr_merge "Every other gh pr verb is allowed."
    fi
  fi

  if [[ "$__gh_sub" == "api" ]]; then
    __gh_write=0
    for ((__i = __GH_IDX + 1; __i < ${#TOKENS[@]}; __i++)); do
      case "${TOKENS[$__i]}" in
        PATCH|POST|PUT|DELETE|-XPATCH|-XPOST|-XPUT|-XDELETE) __gh_write=1 ;;
        --method=PATCH|--method=POST|--method=PUT|--method=DELETE) __gh_write=1 ;;
      esac
    done
    for ((__i = __GH_IDX + 1; __i < ${#TOKENS[@]}; __i++)); do
      case "${TOKENS[$__i]}" in
        */merge|*/merge\?*|*mergePullRequest*)
          __claude_hook_refuse_pr_merge "The route reaches the same merge that gh pr merge performs."
          ;;
        *updateRef*|*createRef*)
          __claude_hook_refuse_push_to_main "The mutation writes a git ref directly."
          ;;
      esac
      if [[ $__gh_write -eq 1 ]]; then
        case "${TOKENS[$__i]}" in
          */git/refs/heads/main|*/git/refs/heads/main\?*|*/git/refs/heads/main/*)
            __claude_hook_refuse_push_to_main "The route writes the git ref of main directly."
            ;;
        esac
      fi
    done
  fi
fi

if [[ "${TOKENS[$START_IDX]}" != "git" ]]; then
  continue
fi

__claude_hook_git_verb_idx
[[ $VERB_IDX -lt ${#TOKENS[@]} ]] || continue
GIT_VERB="${TOKENS[$VERB_IDX]}"

LOOKUP_DIR="$EFFECTIVE_CWD"
if [[ -n "$GIT_C_DIR" ]]; then
  LOOKUP_DIR="$(__claude_hook_resolve_dir "$GIT_C_DIR")"
fi

# --- 2. git commit --no-verify  -> BLOCK ---
#
# Token equality, not substring: a commit body that MENTIONS `--no-verify` must
# pass. A `-m` / `-F` / `-c` / `-C` flag consumes the next token as its value,
# and a cluster of short flags ending in one of those letters does the same.
if [[ "$GIT_VERB" == "commit" ]]; then
  i=$((VERB_IDX + 1))
  while [[ $i -lt ${#TOKENS[@]} ]]; do
    arg="${TOKENS[$i]}"
    case "$arg" in
      -m|-F|--message|--file|-c|-C)
        i=$((i + 2))
        continue
        ;;
      --message=*|--file=*|--reuse-message=*|--reedit-message=*)
        i=$((i + 1))
        continue
        ;;
      --no-verify)
        echo "=== BLOCKED: git commit with hook-bypass flag ===" >&2
        echo "" >&2
        echo "  The --no-verify flag bypasses pre-commit hooks. Banned without explicit human approval." >&2
        echo "  See root CLAUDE.md, Git workflow." >&2
        exit 2
        ;;
    esac
    if [[ "$arg" =~ ^-[a-zA-Z]+[mFcC]$ ]] && [[ "$arg" != -[mFcC] ]]; then
      i=$((i + 2))
      continue
    fi
    i=$((i + 1))
  done
fi

# --- 3. force push without --force-with-lease  -> BLOCK ---
if [[ "$GIT_VERB" == "push" ]]; then
  __force_token=0
  __lease_token=0
  i=$((VERB_IDX + 1))
  while [[ $i -lt ${#TOKENS[@]} ]]; do
    case "${TOKENS[$i]}" in
      --force-with-lease|--force-with-lease=*) __lease_token=1 ;;
      --force|-f) __force_token=1 ;;
    esac
    i=$((i + 1))
  done

  if [[ $__force_token -eq 1 ]] && [[ $__lease_token -eq 0 ]]; then
    echo "=== BLOCKED: force push without --force-with-lease ===" >&2
    echo "" >&2
    echo "  Use --force-with-lease (only on personal branches; never main)." >&2
    echo "  See root CLAUDE.md, Git workflow." >&2
    exit 2
  fi
fi

# --- 4. a merge or a pull that lands a non-main ref on main  -> BLOCK ---
#
# `git pull` is a fetch plus a merge, so it carries the same outcome. Its first
# positional is the remote, not a ref. EVERY ref must resolve to main for the
# command to pass: an octopus merge whose LAST argument is main still lands its
# earlier arguments on main. `merge-tree` and `merge-base` are read-only and are
# load-bearing for the fleet's conflict prediction, so the verb is compared by
# exact equality.
if [[ "$GIT_VERB" == "merge" ]] || [[ "$GIT_VERB" == "pull" ]]; then
  __escape=0
  __ffonly=0
  __continue=0
  __pos_i=0
  __refs_seen=0
  __all_main=1
  for ((__i = VERB_IDX + 1; __i < ${#TOKENS[@]}; __i++)); do
    case "${TOKENS[$__i]}" in
      --abort|--quit) __escape=1; continue ;;
      --continue) __continue=1; continue ;;
      --ff-only) __ffonly=1; continue ;;
      -*) continue ;;
    esac
    __pos_i=$((__pos_i + 1))
    if [[ "$GIT_VERB" == "pull" ]] && [[ $__pos_i -eq 1 ]]; then continue; fi
    __refs_seen=1
    if ! __claude_hook_resolves_to_main "${TOKENS[$__i]}"; then __all_main=0; fi
  done

  __skip=0
  if [[ $__escape -eq 1 ]]; then
    __skip=1
  elif [[ $__continue -eq 0 ]]; then
    [[ $__ffonly -eq 1 ]] && __skip=1
    [[ $__refs_seen -eq 0 ]] && __skip=1
    [[ $__all_main -eq 1 ]] && __skip=1
  fi

  if [[ $__skip -eq 0 ]]; then
    __branch="$(__claude_hook_branch_of "$LOOKUP_DIR")"
    if [[ -z "$__branch" ]]; then
      echo "=== WARNING: merge-into-main check skipped ===" >&2
      echo "" >&2
      echo "  The branch of $LOOKUP_DIR could not be read, so this rule did not run." >&2
    elif [[ "$__branch" == "main" ]]; then
      echo "=== BLOCKED: merge into main ===" >&2
      echo "" >&2
      echo "  An agent never merges work into main. A person merges the pull request." >&2
      echo "  To refresh main, use --ff-only. To leave a merge already started, use --abort." >&2
      echo "  The branch was read from $LOOKUP_DIR." >&2
      __claude_hook_limit_note
      exit 2
    fi
  fi
fi

# --- 5. a push whose destination resolves to main  -> BLOCK ---
#
# The destination is refused whatever the source is. Allowing a bare `main`
# refspec because its source is also `main` leaves a bypass: check out main,
# reset it hard onto a feature branch, then push. That path uses no merge verb,
# so only the destination can close it. A destination of HEAD is resolved
# through the branch, because on main HEAD IS main. The agent must open a pull
# request in any case, and the protect-main ruleset refuses a direct push
# server-side, so this refusal costs the agent nothing.
if [[ "$GIT_VERB" == "push" ]]; then
  __pos_count=0
  __i=$((VERB_IDX + 1))
  while [[ $__i -lt ${#TOKENS[@]} ]]; do
    case "${TOKENS[$__i]}" in
      --all|--mirror)
        __claude_hook_refuse_push_to_main "This flag publishes every local ref, main included."
        ;;
      -o|--push-option|--repo|--receive-pack|--exec)
        __i=$((__i + 2))
        continue
        ;;
      -*)
        __i=$((__i + 1))
        continue
        ;;
    esac
    __pos_count=$((__pos_count + 1))
    __dst="${TOKENS[$__i]##*:}"
    __dst="${__dst#+}"
    if [[ "$__dst" == "HEAD" ]] || [[ "$__dst" == "@" ]]; then
      if [[ "$(__claude_hook_branch_of "$LOOKUP_DIR")" == "main" ]]; then
        __claude_hook_refuse_push_to_main "The branch is main, so HEAD resolves to main."
      fi
    elif __claude_hook_resolves_to_main "$__dst"; then
      __claude_hook_refuse_push_to_main "The destination ref is main."
    fi
    __i=$((__i + 1))
  done

  if [[ $__pos_count -le 1 ]]; then
    if [[ "$(__claude_hook_branch_of "$LOOKUP_DIR")" == "main" ]]; then
      __claude_hook_refuse_push_to_main "The branch is main, so this push writes remote main."
    fi
  fi
fi

# --- 6. git reset --hard  -> WARN (advisory) ---
if [[ "$GIT_VERB" == "reset" ]]; then
  __HAS_HARD=0
  for ((__j = VERB_IDX + 1; __j < ${#TOKENS[@]}; __j++)); do
    if [[ "${TOKENS[$__j]}" == "--hard" ]]; then
      __HAS_HARD=1
      break
    fi
  done
  if [[ $__HAS_HARD -eq 1 ]]; then
    echo "=== WARNING: git reset --hard is destructive ===" >&2
    echo "" >&2
    echo "  Ensure the user authorized this. Lost commits cannot be recovered" >&2
    echo "  without the reflog." >&2
  fi
fi

done < <(printf '%s\n' "$COMMAND" | __claude_hook_split_commands)

exit 0
