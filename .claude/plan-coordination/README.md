# Hypervisor wiring

This directory holds the durable-state plumbing for the **Hypervisor** meta-agent
(`.claude/agents/engineering/hypervisor.md`) and the standalone-runnable
**Orchestrator** / **Automated Orchestrator** that adopt the same plan-scoped
state store.

There is no live agent MESH in this environment. `SendMessage` exists, but it is
an intra-session steering affordance ONLY (resume/steer a subagent you spawned) —
never a coordination substrate and never cross-instance (D38). Every worker is a fresh, run-to-completion
subagent; **continuity is the on-disk LMDB store plus git, never a live agent.**
These files are the on-disk substrate.

## Why LMDB, and why GLOBAL

A single plan is operated on by **concurrent orchestrators/hypervisors** (multiple
Hypervisor/operator sessions, plus per-worktree `db.sh` invocations — separate OS
processes). The store must therefore tolerate
**many writers from many processes**, and it must be reachable from **multiple
worktrees** of the same repo at once.

- **LMDB** is a memory-mapped KV store with **lock-free readers** and a
  **serialized-but-graceful single writer**, with **multi-process locking built
  in** (an OS-level lock LMDB keeps in the env directory). Concurrent writers
  from different processes serialize cleanly instead of erroring — there is no
  `SQLITE_BUSY`-style storm and there are **no external lock files** (no
  `mkdir` locks, no advisory `flock`). We rely entirely on LMDB's own lock.
- **GLOBAL, plan-keyed location.** The env lives at
  `~/.claude/plan-dbs/<plan-key>/`, **outside any worktree**, so every
  worktree / session / orchestrator / hypervisor on the same plan shares one
  store. (The old in-worktree `<plan>/.hypervisor/state.db` is gone — an untracked
  in-worktree file is unreachable from a sibling worktree.)

**How `db.sh` reaches LMDB.** `db.sh` is a thin launcher for the Rust binary
`plan-db` (`tools/plan-db`, a workspace member that links LMDB through the `heed`
crate). It resolves, in order: `plan-db` on `PATH` (installed by
`scripts/bootstrap.sh` with `cargo install --path tools/plan-db`), then
`<main checkout>/target/release/plan-db`, then `target/debug/plan-db`, and as a
last resort `cargo run --release -p plan-db` from the main checkout. The plan key
is derived from git, not from where the binary lives, so every path opens the
SAME store. Install the binary once so the hooks' 20-second store timeout is
never reached by a cold `cargo run`.

## Files

| File | What it does |
|---|---|
| `db.sh` | The single DB tool. Every agent calls `.claude/plan-coordination/db.sh <cmd>`. A launcher for the Rust `plan-db` binary (`tools/plan-db`, LMDB via `heed`). Subcommands below. Rely on LMDB's built-in cross-process locking; no external locks. |
| `check-usage.sh` | On-demand reader for the account-GLOBAL usage window. Emits clean JSON (`five_hour` / `seven_day` `{used_pct, remaining_pct, reset_in_s, pace_delta}`, `context {used_pct}`, `captured_at`, `stale_s`, `source`). Safe under concurrent reads; never writes. |
| `README.md` | This file. |

There is **no `db-init.sh`** — its job moved to `db.sh init`. `PLAN_DB_ROOT` overrides the
`~/.claude/plan-dbs` parent for tests only (see `tools/plan-db/tests/roundtrip.rs`).

## `db.sh` — the access CLI

```sh
.claude/plan-coordination/db.sh resolve <plan-dir>          # print the canonical global env path
.claude/plan-coordination/db.sh init    <plan-dir>          # open/create env; seed current_context + control:signal=run if absent
.claude/plan-coordination/db.sh get     <plan-dir> <key>    # print the value at <key> (empty if absent)
.claude/plan-coordination/db.sh put     <plan-dir> <key> <value>   # DURABLE write to a FIXED key (awaits the commit)
.claude/plan-coordination/db.sh del     <plan-dir> <key>    # delete <key>
.claude/plan-coordination/db.sh scan    <plan-dir> <prefix> [-v]   # print keys with <prefix>; -v emits {"key","value"} JSON per line
.claude/plan-coordination/db.sh len     <plan-dir> <key>    # print {key, bytes, approx_tokens} for one key (NO value)
.claude/plan-coordination/db.sh append  <plan-dir> <suffix> <value>   # mint <flake>-<suffix>, write, print the key
.claude/plan-coordination/db.sh keys    <plan-dir> [prefix] # print {key, bytes, approx_tokens} for ALL keys (NO values)
```

- It is a CLI: writing results to **stdout** is its job. `put` and `append` commit
  the LMDB write transaction before they return, so the call resolves only on a
  **durable commit**.
- `init` is idempotent (create-if-absent, open-if-present) and is the **unified
  entry point** for the whole plan-DB lifecycle: **genesis** (the DB is set up in
  the SAME plan-mode phase that produces the fixed roadmap docs — exiting plan
  mode yields both the roadmap docs and an initialized DB), **resume** (a
  successor re-reads `current_context`), and **legacy backfill** (a roadmap plan
  that predates plan DBs gets its DB spun up on continuation). It leaves an
  existing `current_context` / `control:signal` untouched and only seeds them
  when absent.
- **Two key classes.** `put`/`get`/`del` address the small set of **FIXED**
  well-known keys directly (`current_context`, `control:signal`,
  `orch:<id>:status`, `orch:<id>:heartbeat`, `pacing:current`). The **DYNAMIC**
  bulk (reports, findings, results, checkpoints, work-item events, fluid notes) is
  written with `append <suffix> <value>`, which mints a unique
  `<simpleflake>-<suffix>` key (no shared counter — coordination-free for
  concurrent writers) and prints it. The printed key is the **pointer** a worker
  reports upward.
- **`keys` is the self-describing INDEX.** It enumerates every key with its
  `{bytes, approx_tokens}` (`ceil(bytes/4)`) but NO values — a fresh-session
  Hypervisor digests this to decide what to selectively `get`/`len`. `len`
  size-probes a single key the same way. Neither pulls a verbose body into the
  caller's window.
- **`scan -v` is JSON-per-line and safe to parse.** Each match prints one compact
  JSON record `{"key":...,"value":...}` on its own line, so embedded newlines and
  tabs in a value (full reports are multi-line JSON) are escaped and **never split
  a record** across output lines — the same JSON framing `keys`/`len` use. Parse
  it line-by-line with `jq`; a single key's authoritative raw value still comes
  from `get <key>` (no framing).

### The plan-key resolver (deterministic — all actors must agree)

The env directory is derived **deterministically** so every actor resolves the
**same** store from any worktree:

```
<plan-key> = "<repo-basename>__<roadmap-slug>"   sanitized to [A-Za-z0-9._-]
```

- **`<repo-basename>`** is the basename of the **main checkout's project root**,
  obtained from `git rev-parse --git-common-dir` and taking its **parent's**
  basename. A linked worktree's `--show-toplevel` differs per worktree; the
  **common dir** always points at the main `.git`, so this basename is
  **identical from every worktree** of the repo.
- **`<repo-relative-slug>`** is the plan directory's path **relative to the repo
  root**, with separators flattened (`roadmap/cerebro` -> `roadmap-cerebro`,
  `docs/cerebro` -> `docs-cerebro`). The **full** relative path is used, never the
  leaf basename — so two plans that share a leaf name (`roadmap/cerebro` vs
  `docs/cerebro`) can **never** collide onto one store.

**ALWAYS pass the canonical repo-root-relative roadmap path** (e.g.
`roadmap/<plan>`). A **relative** plan-dir is interpreted relative to the **repo
root**, never the cwd, so it resolves to the **same** store from the main checkout
and from any worktree. **NEVER pass `.`** — from a worktree, `.` is the worktree
dir, which is outside the main repo root and is **rejected with a clear error**
(as is any absolute path that resolves outside the repo root, or the repo root
itself). This prevents two actors who both pass `.` from silently getting
different stores and never seeing each other.

Example: `roadmap/first-light` in this repo resolves to
`~/.claude/plan-dbs/duet__roadmap-first-light/` whether you run `db.sh` from the
main checkout or from any worktree.

## LMDB is a KV store (no SQL)

There is **no SQL** and there are no tables — LMDB is a flat, ordered key/value
space. Lookups are **prefix scans** (`scan <prefix>`, which works because keys
are stored in lexicographic order) plus explicit `idx:*` index keys you write
yourself. The full keyspace is the authoritative schema and lives in
`.claude/agents/engineering/memory-agent.md` — `db.sh` and that file describe the
**identical** keyspace; if you change one, change both.

## File ownership (who writes what)

This is the unambiguous contract for every on-disk artifact in this system:

| Artifact | Location | Written by | Read by |
|---|---|---|---|
| LMDB env (all `current_context` / `control:signal` / `work_item:*` / `pr:*` / `finding:*` / `fluid:*` / `orch:*` / `idx:*` / `<flake>-compaction-frame` keys) | `~/.claude/plan-dbs/<plan-key>/` (GLOBAL) | `db.sh` (any agent; `compaction-frame` rows by the `PreCompact` hook) | `db.sh` (any agent) |
| `usage-window.json` (account-global 5h/7d windows) | `~/.claude/usage-window.json` (GLOBAL) | the global statusline (§1) | `check-usage.sh` |
| `self-ctx.json` (the CURRENT session's own context%) | `${CLAUDE_PROJECT_DIR:-$PWD}/.hypervisor/<session_id>/self-ctx.json` (session-private, per-session subdir) | the statusline (§2) | the session-root agent reads **its own** |
| `role` (POSITIVE owner-session-id identity marker — `hypervisor <owning-session-id>`; the hooks act only when BOTH the per-session subdir name AND the recorded owner match their stdin `session_id`, NEVER the inheritable env) | `${CLAUDE_PROJECT_DIR:-$PWD}/.hypervisor/<session_id>/role` (session-private) | the Hypervisor at INITIALIZE; removed at HALT | the `PreCompact` + `SessionStart` hooks |
| `plan-dir` (the plan address the hooks resolve) | `${CLAUDE_PROJECT_DIR:-$PWD}/.hypervisor/<session_id>/plan-dir` (session-private) | the Hypervisor at INITIALIZE | the compaction hooks |
| `session-id` (THIS session's id) | `${CLAUDE_PROJECT_DIR:-$PWD}/.hypervisor/<session_id>/session-id` (session-private) | the statusline (§2), every render | the Hypervisor INITIALIZE (cross-checks `$CLAUDE_CODE_SESSION_ID` against it, then stamps the `role` owner + `current_context.my_session_id`) |
| `hypervisor-init.json` (one-time bootstrap flag: LMDB usable + statusline emitting 5h/7d + compact-and-continue substrate, D18) | `~/.claude/hypervisor-init.json` (GLOBAL) | the Hypervisor's Bootstrap Gate | the Hypervisor at INITIALIZE |

`db.sh init` creates **only** the LMDB env. It does **not** create
`usage-window.json` or `self-ctx.json` — those are written by the statusline, and
their **absence is a non-fatal WARN + degrade**, never a hard failure.

> **`.hypervisor/<session_id>/` is created LAZILY by the statusline** in the session cwd
> (`${CLAUDE_PROJECT_DIR:-$PWD}/.hypervisor/<session_id>`, scoped by the session's own id so
> MULTIPLE Hypervisors in one cwd never collide) — **NOT by `db.sh`**. It may therefore be
> **absent in a worktree with no statusline-emitting root**, which is exactly why
> `self-ctx.json` absence is a non-fatal WARN + degrade. Note the Hypervisor's fleet
> orchestrators are background subagents with NO statusline (no self-ctx at all —
> bounded from OUTSIDE); only a session ROOT (a Hypervisor, or an operator-launched
> orchestrator) writes self-ctx, and it must run with `CLAUDE_PROJECT_DIR`
> set to **its own session cwd** so the dir it writes is the dir it reads.

## 1. The global statusline usage emit (operator-installed, ONE time)

Usage windows (5h / 7d) are **account-global**, not per-plan. The ONLY carrier of
`rate_limits` is the global statusline payload (verified: hooks do NOT receive
`rate_limits`). So the operator augments `~/.claude/statusline.sh` to ATOMICALLY
write the parsed usage + context to a single global file every Hypervisor reads
on demand. **This path is unchanged by the LMDB migration.**

The statusline receives its session JSON on **stdin**. After its normal work, add
an emit that writes `rate_limits`, `context_window`, and a capture timestamp to
`~/.claude/usage-window.json` using a **temp + mv** atomic write (so a
concurrent reader never sees a half-written file):

```sh
# --- hypervisor usage emit (add near the end of ~/.claude/statusline.sh) ---
# $input holds the full stdin JSON the statusline already read.
hypervisor_dir="$HOME/.claude"
mkdir -p "$hypervisor_dir"
tmp="$(mktemp "$hypervisor_dir/.usage-window.XXXXXX")"
printf '%s' "$input" | jq -c '{
  rate_limits:   (.rate_limits // null),
  context_window:(.context_window // null),
  captured_at:   (now | floor)
}' > "$tmp" 2>/dev/null && mv -f "$tmp" "$hypervisor_dir/usage-window.json" || rm -f "$tmp"
# --- end hypervisor usage emit ---
```

### The verified `rate_limits` sub-shape (PIN — do not drift)

`check-usage.sh` parses these exact fields. The shape was verified against the
Claude Code 2.1.181 statusline JSON:

```jsonc
"rate_limits": {
  "five_hour": { "used_percentage": <0-100>, "resets_at": <UNIX EPOCH SECONDS> },
  "seven_day": { "used_percentage": <0-100>, "resets_at": <UNIX EPOCH SECONDS> }
},
"context_window": { "used_percentage": <0-100> }
```

- `resets_at` is **UNIX EPOCH SECONDS** (not ISO-8601). `check-usage.sh` computes
  `reset_in_s = resets_at - now` directly; if a future CLI ever emits ISO-8601,
  convert it in the emit or in `check-usage.sh` before this subtracts.
- `rate_limits` is **Pro/Max only** and appears **only after the first API
  response** of a session; each window (`five_hour`, `seven_day`) can be
  independently absent. `check-usage.sh` degrades to `source: "absent"` and the
  Hypervisor falls back to the open-loop token budget — never a fabricated number.
- `$input` is whatever variable your statusline captured stdin into (e.g.
  `input="$(cat)"` at the top). Adapt the name to your script.

**Self-test (run after installing the emit):** capture one real post-first-
response statusline payload to a file and run
`HV_USAGE_FILE=/path/to/payload.json .claude/plan-coordination/check-usage.sh`; confirm
`source` is `live` with non-null `five_hour` / `seven_day`. A field-name drift
(e.g. the CLI renaming `used_percentage`) then surfaces loudly as `absent`/null
instead of silently mis-pacing.

## 2. The statusline context emit (session-private self-context)

Only a **session ROOT** has a statusline; a subagent child has none, and a parent
cannot observe a child's context. So the **only** honest self-context signal is a
session writing its **own** `context_window.used_percentage`. There is **ONE
mechanism (the statusline) and ONE path pattern** — a PER-SESSION subdir
`.hypervisor/<session_id>/` so multiple Hypervisors sharing one cwd never clobber:

```sh
# --- hypervisor self-context emit (add to ~/.claude/statusline.sh) ---
# Derive the dir from the session cwd; the statusline runs in the session's cwd. Scope the
# per-session files under .hypervisor/<session_id>/ so MULTIPLE Hypervisors sharing ONE cwd
# never clobber each other.
hypervisor_sid="$(printf '%s' "$input" | jq -r '.session_id // empty' 2>/dev/null)"
hypervisor_base="${CLAUDE_PROJECT_DIR:-$PWD}/.hypervisor"
hypervisor_self_dir="$hypervisor_base"
[ -n "$hypervisor_sid" ] && hypervisor_self_dir="$hypervisor_base/$hypervisor_sid"  # flat = degraded single-session
mkdir -p "$hypervisor_self_dir"
printf '%s' "$input" | jq -c '{ used_percentage: (.context_window.used_percentage // null) }' \
  > "$hypervisor_self_dir/self-ctx.json.tmp" 2>/dev/null \
  && mv -f "$hypervisor_self_dir/self-ctx.json.tmp" "$hypervisor_self_dir/self-ctx.json" \
  || rm -f "$hypervisor_self_dir/self-ctx.json.tmp"
# Record THIS session's id INSIDE its own dir so the Hypervisor's INITIALIZE can cross-check
# $CLAUDE_CODE_SESSION_ID against it before stamping the .hypervisor/<id>/role OWNER (the
# compaction hooks gate on subdir == own session_id AND owner == own session_id).
if [ -n "$hypervisor_sid" ]; then
  printf '%s\n' "$hypervisor_sid" > "$hypervisor_self_dir/session-id.tmp" 2>/dev/null \
    && mv -f "$hypervisor_self_dir/session-id.tmp" "$hypervisor_self_dir/session-id" \
    || rm -f "$hypervisor_self_dir/session-id.tmp"
fi
# Bounded GC (~once/hr via a stamp): a per-session dir is created for EVERY session; prune
# UUID-named subdirs idle >14 days (ended/crashed sessions). The threshold MUST exceed the
# longest legitimate idle — a Hypervisor on WAIT_FOR_RESET can wait out a 7-day usage window
# without rendering — so a paused-but-live session is never reaped. Active sessions stay fresh
# each render; a resumed one re-stamps role/plan-dir at INITIALIZE.
hypervisor_gc_stamp="$hypervisor_base/.gc-stamp"
if [ ! -e "$hypervisor_gc_stamp" ] || [ -z "$(find "$hypervisor_gc_stamp" -mmin -60 2>/dev/null)" ]; then
  find "$hypervisor_base" -mindepth 1 -maxdepth 1 -type d \
    -name '????????-????-????-????-????????????' -mtime +14 -exec rm -rf {} + 2>/dev/null || true
  : > "$hypervisor_gc_stamp" 2>/dev/null || true
fi
# --- end hypervisor self-context emit ---
```

- The path is `${CLAUDE_PROJECT_DIR:-$PWD}/.hypervisor/<session_id>/self-ctx.json` (the
  statusline derives the dir from the session cwd + the session's own id, falling back to
  `$PWD`). It is **session-private and NOT shared** — each session writes and reads its
  **own**, and the `<session_id>` subdir is what isolates concurrent same-cwd Hypervisors.
  The Hypervisor agent finds its own subdir via `$CLAUDE_CODE_SESSION_ID` (cross-checked
  against the statusline-written `session-id` inside it); the hooks via their stdin `session_id`.
  `$CLAUDE_PROJECT_DIR` and the session cwd coincide when the session is launched
  from the project root; an operator-launched orchestrator running as its own session
  root must run with `CLAUDE_PROJECT_DIR` set to **its own session cwd** so the
  dir it writes is the dir it reads (the worktree caveat). The Hypervisor's fleet
  orchestrators are background subagents — no session root, no self-ctx, bounded from OUTSIDE.
- The session-root agent reads its own file to act at its context trigger: the
  Hypervisor at **60%** enters compaction-safe quiescence + checkpoint and
  compacts-and-continues in place (it only `WIND_DOWN`s as a backstop); an
  operator-launched orchestrator running as a root has the same statusline signal but winds
  down at the **operator's discretion** — no fixed self-ctx trigger.
- **Absence = WARN + degrade (non-fatal):** the agent checkpoints continuously
  and winds down conservatively instead of trusting a context number it cannot
  read. An in-session (non-root) orchestrator has **no statusline at all** and is
  bounded from OUTSIDE by the Hypervisor (work-count + wall-clock).
- `.hypervisor/` is **gitignored** in the worktree (it now holds only the ephemeral
  per-session subdirs `<session_id>/` — no DB lives there any more; the statusline bounds
  their growth with an opportunistic GC of UUID-named subdirs idle for >14 days — a threshold
  deliberately wider than the longest legitimate pause, the 7-day usage window).

## 3. State store init + on-demand reads

```sh
# Initialize / resume the global plan-keyed env (idempotent):
.claude/plan-coordination/db.sh init roadmap/<your-plan>
# -> ~/.claude/plan-dbs/<repo>__<your-plan>/  (LMDB env; seeds current_context + control:signal=run)

# Read the operator control channel FIRST every cycle:
.claude/plan-coordination/db.sh get roadmap/<your-plan> control:signal   # run | pause | abort

# On-demand usage read:
.claude/plan-coordination/check-usage.sh        # reads ~/.claude/usage-window.json
HV_USAGE_FILE=/path/to/usage-window.json .claude/plan-coordination/check-usage.sh   # override for testing
```

The Hypervisor runs `check-usage.sh` each PERCEIVE step. `source: live` drives
closed-loop pacing; `source: stale|absent` drives the open-loop fallback
(serialize at N=1, inflate estimates, abort new dispatch on the first 429).

## 4. The operator control channel (`control:signal`)

The operator's RUN / PAUSE / ABORT channel is the **`control:signal` LMDB key**
(it replaces the old `control.json` file). Every long-lived actor reads it
**FIRST, each cycle**:

```sh
.claude/plan-coordination/db.sh get roadmap/<your-plan> control:signal     # run | pause | abort
```

- `run` — proceed normally.
- `pause` — checkpoint and durably sleep; do not dispatch new work.
- `abort` — **wind down**, set `current_context.resume_mode = aborted_by_operator`,
  and **HALT without deleting the store**. A reborn actor sees the abort and
  requires fresh operator confirmation before re-entering the loop.

To abort a run from anywhere (any worktree, any process):

```sh
.claude/plan-coordination/db.sh put roadmap/<your-plan> control:signal abort
```

## 5. Fluid state never lives in markdown

Static docs (the plan graph, specs, ADRs) are markdown under `roadmap/<plan>/` and
are git-tracked. Fluid state (next steps, work items, findings, reports, checkpoints)
lives ONLY in the store, under the keyspace `memory-agent.md` defines. An
orchestrator that writes fluid state to a markdown file has left the protocol.
