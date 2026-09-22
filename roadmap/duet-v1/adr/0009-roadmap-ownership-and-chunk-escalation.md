# ADR-0009: The Architect owns every path under `roadmap/`, and a chunk that finds its brief wrong stops

## Status

Accepted. It answers escalation M0-3 and it adds rule SM9 to specification section 13.0.

## Context

Chunk M0 found that step 25 of its own brief orders files under
`.claude/skills/gpui-kit/references/gpui/` while the front-matter `write_scope` named only
`SKILL.md`. The two halves of one brief disagreed, and the front matter is the half
`cargo xtask check-plan-graph` reads. M0 added the directory to its own `write_scope` and to its own
files table, and it regenerated `plan-graph.md`.

A scan then found that no chunk of the whole plan names a path that opens `roadmap/`. So
`roadmap/duet-v1/m-00-manifest-phase-0.md` and `roadmap/duet-v1/plan-graph.md` were owned by nobody,
and the one act that repaired a brief was an act the brief did not authorise.

The mechanism, not the substance, is the problem. **A chunk that widens its own `write_scope` makes
the guard circular.** `check-plan-graph` reads the front matter, so a chunk that edits the front
matter proves only what the chunk decided. That is the shape rule CL1c of specification section 1.5
already refuses for a closure record, for the same reason.

One further constraint is a fact of the tooling and it shaped the decision. The `DATA_BLOCKS`
register of `tools/xtask/src/check_placement.rs` states the row floor of every
`<!-- GUARD BLOCK ... -->` block. PG27 refuses a marker that differs from the register and PG27b
refuses a block that holds more rows than its floor. **A row added to a registered block therefore
needs the document and the register in ONE commit**, and the Architect writes no Rust.

## Decision

Rule SM9 of specification section 13.0 states all four parts. This record states the decision and
the reasoning.

1. **The Architect is the one writer under `roadmap/`.** The specification, every chunk file, the
   design contract, the product requirements, this directory, and the plan tools are the
   Architect's. `roadmap/duet-v1/reviews/` is append-only and the Critic is its writer, which rule
   CL1c depends on.
2. **`roadmap/duet-v1/plan-graph.md` is the one generated exception.**
   `cargo xtask check-plan-graph roadmap/duet-v1 --write-manifest` writes it from the chunk front
   matter. A chunk that changes the generator regenerates the file in the same commit and names the
   path in its own `write_scope`. The guard stays sound, because the front matter it reads is
   unchanged: the chunk refreshes a build artifact and edits no brief. Chunk M90 owns the
   `MANIFEST_NOTE` repair, which still credits the Python prototype rather than the Rust generator.
3. **An engineer that finds its own brief wrong STOPS and reports.** It never widens its
   `write_scope`, it never edits its own chunk file, and it never edits the specification. It writes
   one plan-store row at the FIXED key `escalation:<chunk id>-<n>` with
   `.claude/plan-coordination/db.sh put`, and it returns the key upward. The JSON carries `kind`,
   `source`, `title`, `detail`, `tier`, `acknowledged_ts`, `verdict`, and `opened_ts`. The nine rows
   of 2026-09-22 are the worked examples. **A chunk that stops on an escalation is complete when
   every other step passed**; the escalation is the deliverable for the step it could not take.
4. **A repair chunk carries a document edit only when a `tools/xtask` register change makes the two
   atomic.** The Architect writes the exact replacement text into the chunk body and the chunk
   applies it. The Architect still authors every word.
5. **A repair chunk takes an `M` id at or above 90**, so no reader takes it for the manifest chunk
   of a phase. The plan states no phase above 15.
6. **An escalation row has a CLOSE state, and an open row BLOCKS a dependent chunk.** Decision 3
   named the fields `acknowledged_ts` and `verdict` and gave them no writer and no time, so the
   channel carried a message that nobody ever answered: all nine rows of 2026-09-22 still read
   `null` after the act that resolved them. **The Architect writes both fields on the same row, in
   the act that resolves the escalation**, with `.claude/plan-coordination/db.sh put` over the whole
   row. `acknowledged_ts` takes the time the Architect read the row. `verdict` takes one of
   `RESOLVED`, `DEFERRED` or `REFUSED`, plus one sentence that names the section, the ADR, or the
   repair chunk that carries the answer. **A row whose `verdict` is `null` is OPEN, and a chunk that
   depends on an open row does not dispatch.** The Orchestrator reads
   `.claude/plan-coordination/db.sh scan roadmap/duet-v1 escalation:` before it dispatches a phase.
   A completed chunk beside an open row lets a fresh Orchestrator read the plan as finished while
   its principal deliverable is undone, which is the same class as a guard that reports a pass it
   did not earn. Specification section 13.0 rule SM9 part 5 states the rule.
7. **Every `M` chunk of a phase runs before every LINE chunk of that phase, in the order the
   section 13.3 row prints.** Rule SM1 already held the phase manifest apart that way. A repair
   chunk needs the same order for a second reason: it changes behaviour that a line chunk of the
   same phase CONSUMES, and the write scopes are disjoint, so PG31 and `check-plan-graph` are blind
   to that edge. **A section 13.4 link was refused for it**, because SM8 and PG31 refuse a link
   whose predecessor shares a phase with its successor; such a link would cascade the whole phase
   table for an edge SM1 already orders.

## Consequences

Easier:

- The party a guard judges and the party that writes the document the guard reads are two parties.
- A defect in a brief leaves evidence in the plan store, which sits outside the repository and
  outside every engineer's write scope, so it cannot be quietly retyped.
- `plan-graph.md` has an owner and a command, so the generated file and its generator move together.
- An engineer has one clear answer to "my brief is wrong": stop, write one row, return the key.

Harder:

- A brief defect costs one dispatch cycle rather than one edit. That is the price of the guard
  staying sound, and the escalation row makes the cycle cheap.
- The Architect cannot add a row to a registered guard block alone. Every such repair needs a repair
  chunk, and the Architect has to write the replacement text into the chunk body rather than into
  the document. This record names the cost rather than hiding it; part 4 is the mechanism.
- Two ranges of `M` number now exist, and a reader has to know the rule. Section 13.0 states it at
  the site.

## Alternatives rejected

| Alternative | Reason |
|---|---|
| Give each chunk its own file in its `write_scope` | It is exactly the circularity: the guard reads the front matter and the chunk would own the front matter. |
| Give one chunk the whole of `roadmap/` | It would let one engineer rewrite every other brief, and the guard would still read a document that engineer owns. |
| Let a chunk open a pull-request comment instead of a store row | A comment is not durable, no key names it, and a fresh orchestrator with no context cannot find it. The plan store is the coordination substrate this fleet already runs on. |
| Move the row floors out of `check_placement.rs` into the document alone | The floor would then be a number the document states about itself, which is the vacuous shape PG27b exists to refuse. Two sources are the point. |
| Let the Architect edit `tools/xtask` for a floor bump alone | The Architect writes no Rust in this plan, and a one-line exception to that is an exception a later act widens. |
