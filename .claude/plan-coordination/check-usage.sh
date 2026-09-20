#!/usr/bin/env bash
#
# check-usage.sh — on-demand reader for the account-GLOBAL usage window.
#
# Usage windows (5h / 7d) are account-global, NOT per-plan. The ONLY carrier of
# rate_limits is the global ~/.claude/statusline.sh, which writes
#   ~/.claude/usage-window.json
# ATOMICALLY (temp + mv) every turn. Hooks do NOT receive rate_limits, so this is
# the only viable source. This reader runs ON DEMAND, is safe under concurrent
# reads (many hypervisors share the one global file), and never writes that file.
#
# Output: clean JSON on stdout —
#   {
#     "five_hour": {"used_pct":N,"remaining_pct":N,"reset_in_s":N,"pace_delta":N},
#     "seven_day": {"used_pct":N,"remaining_pct":N,"reset_in_s":N,"pace_delta":N},
#     "context":   {"used_pct":N},
#     "captured_at": <epoch_s>,
#     "stale_s": <int>,
#     "source": "live" | "stale" | "absent"
#   }
#
# source semantics (the open-loop fallback trigger for the Hypervisor):
#   live   — file present and fresh (captured within STALE_THRESHOLD_S)
#   stale  — file present but older than STALE_THRESHOLD_S, OR a window reset has passed
#   absent — file missing/unreadable, OR no rate_limits at all (e.g. non-Pro/Max, or
#            before the first API response of the session)
#
# A "window present but null" field (rate_limits arrives per-window, independently)
# is surfaced as JSON null for that field; the reader does not fabricate a number.
#
# pace_delta = used_pct - elapsed_pct_of_window. Positive => burning AHEAD of an
# even pace (behind on budget); negative => surplus. null when reset_in_s is null.
#
# PINNED rate_limits sub-shape (verified, Claude Code 2.1.181; see README §1):
#   rate_limits.{five_hour,seven_day}.{used_percentage: 0-100, resets_at: UNIX
#   EPOCH SECONDS}; context_window.used_percentage: 0-100. resets_at is EPOCH
#   SECONDS, so reset_in_s = resets_at - now directly (NO ISO-8601 conversion).
#   If a future CLI emits ISO-8601, convert before that subtraction.
# SELF-TEST: capture one real post-first-response payload and run
#   HV_USAGE_FILE=<payload> .claude/plan-coordination/check-usage.sh
#   -> source must be "live" with non-null five_hour/seven_day. A field-name
#   drift then surfaces loudly as absent/null instead of silently mis-pacing.

set -euo pipefail

USAGE_FILE="${HV_USAGE_FILE:-$HOME/.claude/usage-window.json}"
STALE_THRESHOLD_S="${HV_USAGE_STALE_S:-90}"
FIVE_HOUR_S=18000   # 5 * 3600
SEVEN_DAY_S=604800  # 7 * 86400

now="$(date +%s)"

emit_absent() {
  printf '{"five_hour":null,"seven_day":null,"context":null,"captured_at":null,"stale_s":null,"source":"absent"}\n'
}

# --- jq is required for safe parsing; degrade to absent if it is missing -------
if ! command -v jq >/dev/null 2>&1; then
  emit_absent
  exit 0
fi

# --- file presence ------------------------------------------------------------
if [ ! -r "$USAGE_FILE" ]; then
  emit_absent
  exit 0
fi

# --- read once into memory (concurrent-read safe; statusline writes atomically) -
raw="$(cat "$USAGE_FILE" 2>/dev/null || true)"
if [ -z "$raw" ]; then
  emit_absent
  exit 0
fi

# --- validate JSON ------------------------------------------------------------
if ! printf '%s' "$raw" | jq -e . >/dev/null 2>&1; then
  emit_absent
  exit 0
fi

# --- transform with jq. All windows/fields parsed defensively (// null). ------
# The statusline writes the parsed rate_limits + context_window + captured_at.
# We accept either a nested {used_percentage, resets_at} per window or flat fields.
printf '%s' "$raw" | jq \
  --argjson now "$now" \
  --argjson stale_threshold "$STALE_THRESHOLD_S" \
  --argjson five_s "$FIVE_HOUR_S" \
  --argjson seven_s "$SEVEN_DAY_S" '
  def num($x): ($x | if type=="number" then . else null end);
  def winfield($w; $k): ($w[$k] // null) | num(.);

  # window builder: takes the raw window object + its full duration in seconds.
  def build_window($w; $dur):
    if $w == null then null
    else
      ($w.used_percentage // $w.used_pct // null)         as $u_in    |
      ($w.resets_at // $w.reset_at // null)               as $reset_in|
      (num($u_in))                                        as $u       |
      (num($reset_in))                                    as $reset   |
      (if $reset == null then null
        else (($reset - $now) | floor) end)              as $rin      |
      (if $u == null then null else (100 - $u) end)       as $rem     |
      # elapsed fraction of the window => even-pace target
      (if $rin == null then null
        else ((($dur - $rin) / $dur) * 100) end)         as $elapsed_pct |
      (if ($u == null or $elapsed_pct == null) then null
        else ($u - $elapsed_pct) end)                    as $pace      |
      {
        used_pct:      $u,
        remaining_pct: $rem,
        reset_in_s:    $rin,
        pace_delta:    (if $pace == null then null else (($pace * 100) | round) / 100 end)
      }
    end;

  (.captured_at // .ts // null | num(.))                  as $cap |
  (.rate_limits // .)                                     as $rl  |
  ($rl.five_hour // $rl["5h"] // null)                    as $w5  |
  ($rl.seven_day // $rl["7d"] // null)                    as $w7  |
  (.context_window // .context // null)                   as $ctx |
  ($ctx.used_percentage // $ctx.used_pct // null | num(.)) as $cu  |

  build_window($w5; $five_s)                              as $five  |
  build_window($w7; $seven_s)                             as $seven |

  # staleness: by capture age, OR if any present window has already reset.
  (if $cap == null then null else ($now - $cap) end)     as $age |
  (
    ($five != null and $five.reset_in_s != null and $five.reset_in_s < 0)
    or ($seven != null and $seven.reset_in_s != null and $seven.reset_in_s < 0)
  )                                                       as $reset_passed |
  (
    if ($five == null and $seven == null) then "absent"
    elif ($age == null) then "stale"
    elif ($reset_passed) then "stale"
    elif ($age > $stale_threshold) then "stale"
    else "live"
    end
  )                                                       as $source |

  {
    five_hour:   $five,
    seven_day:   $seven,
    context:     (if $cu == null then null else { used_pct: $cu } end),
    captured_at: $cap,
    stale_s:     $age,
    source:      $source
  }
'
