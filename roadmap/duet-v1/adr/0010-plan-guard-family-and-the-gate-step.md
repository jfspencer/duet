# ADR-0010: The plan guard family: one message vocabulary, where each guard runs, and the gate step

## Status

Accepted. It answers escalations M0-1, M0-2, M0-4, M0-5, and T1-5. Chunk M90 lands every code half.

## Context

Chunk M0 ported fourteen plan tools to six `cargo xtask` guards. Four defects of the family survived
the port, and each one is about WHERE a guard runs or WHAT its output means.

1. **The word `FAIL:` named two states.** `tools/xtask/src/` holds 49 `"FAIL:` string sites: 5 in
   `check_closure.rs`, 8 in `check_conversions.rs`, 3 in `check_manifests.rs`, 11 in
   `check_placement.rs`, 4 in `check_plan_graph.rs`, 17 in `check_roster.rs`, and 1 in `main.rs`.
   **Five of the 49 reach exit 1 and the other 44 are fail-closed.** A reader could not tell "the
   guard could not decide" from "the input breaks a rule". The first draft of this record wrote
   "five paths and a finding on four", which inverted the denominator and miscounted the numerator.
2. **`check-closure` reads a store a runner does not carry.** Rule CL1c takes its second source from
   the plan store under `~/.claude/plan-dbs/`. Revision 23 promised a `plan-lint` job that runs it
   "against the store the workflow restores", and no chunk owned that restore. M0 removed the nine
   job lines rather than ship a job that is red for ever.
3. **`check-roster` has no probe group.** Every shape of PG25 compiles the whole `gpui` dependency
   tree: one measured run reached 4.0 GB and several minutes cold.
4. **`scripts/dod.sh` runs no plan guard.** Nine commits of chunk T1 passed the local hook, and the
   `plan-lint` job then refused on `check-placement` rule VR1. Section 14 argued against a gate line
   on the ground that it would make every commit depend on one roadmap markdown file.

A fifth defect sat beside them: the `typos` step of `scripts/dod.sh` keeps a fallback that stops a
finding from failing the gate, and section 14 rung one claimed the opposite.

## Decision

### 1. The family speaks two words, and each one names an exit code

**`FAIL:` marks exit 2 and nothing else.** Exit 2 is fail closed: the guard could not decide. At
exit 1 a guard prints `FINDING: ` when the finding carries no rule tag, and the rule tag when the
rule has a name. A blanket `FINDING: ` prefix over every exit-1 line was refused: specification
section 1.9 records more than sixty tagged lines, each tag names the rule that fired, and a bare
prefix would delete that name at no gain.

Four sites change, and section 1.9 holds the table.

- `check_plan_graph`, two files state one chunk id: `FINDING:` at **exit 1**. Rule 1 of the guard is
  that every id is unique, so two files that state one id break a rule the guard decided.
- `check_plan_graph`, the directory holds no chunk file: `FAIL:` at **exit 2**. It is a zero
  denominator, and rule CG1b part 1 of section 2.3 already gives that answer for a member set.
- `check_placement`, the candidate set is empty: `FAIL:` at **exit 2**. The line says the parse is
  broken, so the run measured nothing about the document.
- `check_roster`, three verdict paths: `FINDING:` at **exit 1**, with the same text after the word.

The prototypes under `roadmap/duet-v1/tools/` keep their historical text and the specification
states the divergence, exactly as it already does for the roster exit class. The Rust ports are the
rules of record; PG29 reads the prototypes for rule ids alone.

**The dividing line is the SUBJECT of the failure, and one sentence states it.** Exit 1 is a breach
of a rule the guard measures about the DOCUMENT it judges. Exit 2 is a failure of the guard's OWN
input: its register, its parse, or the transform table it applies before it measures anything.

**The two roster SUBSTITUTION sites keep exit 2, and that sentence is why.** `check_roster.rs`
refuses a run when the section 1.9 substitution block and the `SUBSTITUTIONS` list the guard
compiles disagree, and when a substitution line states a number. Both breaches are decidable, and
neither one is about the document the guard judges: **the substitution block is the guard's own
transform table**, which it applies to turn the section 15 declarations into a compilable workspace.
A disagreement means the workspace the guard is about to write is not the workspace the document
describes, so every counter downstream of it is unattributable. Those are failures of the guard's
input, so they take the same answer as the empty candidate set and the empty plan directory. **They
stay at exit 2 and chunk M90 does not move them**; the first draft of this record left that class
unexamined.

### 1a. `--check-manifest` proves consistency and the human read proves the text

`MANIFEST_NOTE` is one constant of `tools/xtask/src/check_plan_graph.rs`. The generator WRITES it
and `--check-manifest` compares the file against the same generator, so that subcommand exits 0 for
any value of the constant and proves nothing about the repair. **Chunk M90 step 17 therefore names
the human read as the proof of the text** and the exit code as the proof of consistency alone. A
check whose two sides come from one source is a tautology, and a record that calls it evidence
teaches a reader to stop checking.

### 2. `check-closure` is a review-time, Architect-local command

It runs in no job. A restore step would need a secret no chunk owns. A copy of the store inside the
repository would put the second source back inside the Architect's write scope, which is the one
property rule CL1c exists to deny. A rule that passes when the store is absent is the vacuous green
that rule C22I-W4 already condemns in this document's own words. **The guard therefore runs where
the store lives**: the Architect runs it before a plan revision, and the Engineering Critic re-runs
it over the same store when it verifies a closure. Specification section 14 and rule CL1c both say
so now, and neither promises a restore.

**Every run is RECORDED in the plan store, and rule CL1d binds a closure section to its run.** A
guard that runs in no job and reports through prose alone is a verdict nobody can reproduce, which
is the state rule CL1b refuses in the same section. The party that runs the guard appends one row
with `.claude/plan-coordination/db.sh append roadmap/duet-v1 closure-run "<body>"`, which mints the
key `<simpleflake>-closure-run`. **The body opens with `COMMAND: `, `EXIT: ` and `TREE: `, one per
line, and then holds the whole stdout of the run.** A closure section of revision 24 or later
records the key in the sentence ``The `check-closure` run that verified this block is recorded at
plan-store key `<key>`.``, which carries its own head and never collides with the CL1c review-key
sentence. The guard reads the key back with `db.sh get`, and it fails on an absent sentence, a wrong
suffix, a store that does not answer, and a body whose `EXIT: ` line states anything but `0`. A
section below revision 24 prints `RUN KEY: absent`, which is the forward cut-off CL1b already uses.
**The store is append-only and outside the repository**, so the row is a second source for the
VERDICT in the same way the stored review is a second source for the review text, which is the one
property CL1c exists to hold.

### 3. `check-roster --generate-only` splits PG25 at the compiler edge

One subcommand, one rule id, one flag that names the half that runs. With the flag the guard writes
the scratch workspace, runs no cargo command, prints
`ROSTER COMPILE:   skipped (--generate-only)`, and decides the parse rules and the count rules
alone. Without the flag it does everything it does today. **A gate never takes the flag**: the
`plan-lint` job runs the full command, and `tools/xtask/tests/probes.rs` takes the flag and holds
the cheap shapes.

**The flag DOES remove a rule clause, and this is the honest statement of what it removes.** PG25
has three clauses: a roster the real lint table refuses, a roster below the section 1.5 denominator,
and a roster below the `impl-sites` floor. **The compiler is the only oracle for clause one**, so
the skipped form decides clauses two and three and decides nothing about clause one. The first
draft of this record claimed the flag "removes no rule the skipped form can reach", which is true of
the two clauses it keeps and false of the rule.

**The difference from `--no-store` is three properties and not that claim.** The skipped run prints
`ROSTER COMPILE:   skipped (--generate-only)`, a line no full run prints. The full form is the only
form any job runs. **A PROBE now refuses a `--generate-only` on a `check-roster` line of ANY file
under `.github/workflows/`**, so the rule that keeps the flag out of a job has a red run of its own,
and a committed positive control proves that the probe can go red. A prose rule has no denominator,
and that is exactly how `--no-store` reached a job in the first place.

**The probe reads the DIRECTORY and not one path** (critic C2-7). The rule is headed "every job", so
a denominator of one file leaves a second workflow unguarded from the day it is written, and this
plan adds `soak.yml` and `audio-smoke.yml`. The probe reads `.github/workflows/` with
`std::fs::read_dir` and fails closed on a directory that does not open and on a directory that
yields zero files. **Its stated limit** is that it reads one LINE at a time, so a folded YAML
scalar, a shell variable, and an `env:` entry each defeat it; review holds those.

### 4. The three cheap plan guards run inside `scripts/dod.sh`, path conditional

The step runs `cargo xtask check-placement roadmap/duet-v1/architecture.md`, then
`cargo xtask check-plan-graph roadmap/duet-v1`, then
`cargo xtask check-plan-graph roadmap/duet-v1 --check-manifest`. **It runs only when the change
under test names a path that opens `roadmap/`**. `check-roster` stays in the `plan-lint` job,
because it is the one guard rule that costs a compile.

**The denominator has THREE clauses, and the third one is why the step is reachable at all.** The
first draft of this record defined it as the staged diff plus the working-tree diff against `HEAD`,
and both of those are EMPTY on the two surfaces that matter most: `.githooks/pre-push` runs
`scripts/dod.sh` over a clean tree, and `git commit --amend` diffs against the commit it replaces.
A two-clause step would therefore skip at every push and at every amend, and report a pass it did
not earn. The union is `git diff --cached --name-only`, `git diff --name-only HEAD`, and
`git diff --name-only @{upstream}..HEAD`, with
`git diff --name-only "$(git merge-base origin/main HEAD)"..HEAD` when the branch has no upstream.
**When neither reference resolves, the step RUNS**: a step that cannot compute its denominator does
not skip, which is the fail-closed answer the guard family applies everywhere else. When the union
names no path that opens `roadmap/`, the step prints
`plan guards: skipped (no change under roadmap/)` and the gate continues.

**Two residual limits, stated rather than hidden.** A force update that moves the remote ref
backward leaves clause 3 empty while the push still rewrites history. A long-lived branch that has
already pushed a `roadmap/` change carries that change outside clause 3 on its next push; the
earlier run covered it. The `plan-lint` job reads the whole document on every pull request that
touches `roadmap/` or `tools/xtask/`, and that job is the cover for both.

**The SAME denominator turns rule CG9 on, and the script computes it ONCE** (critic C2-1). The
`converts` step of the gate runs `cargo xtask check-conversions` on every commit with no path
condition, and CG9 reads `architecture.md`. An unconditional CG9 would therefore create the exact
coupling this decision exists to remove: every commit in the repository would depend on one roadmap
markdown file, and an Architect edit to one Appendix B.1 reason cell would turn every commit red
until a repair chunk landed the code half. The step passes
`--appendix roadmap/duet-v1/architecture.md` when the union names a path that opens `roadmap/` OR
names `crates/duet-time/src/convert.rs`, and it passes no argument otherwise; the guard then prints
`REASON TEXTS:    skipped (no --appendix)`. **Both halves of the binding sit in the condition**,
because a `roadmap/`-only condition would skip CG9 on the commit that edits the code half.
**One gap stays and this record names it**: a pull request that touches
`crates/duet-time/src/convert.rs` and no `roadmap/` or `tools/xtask/` path runs no CG9 at the merge,
because widening the `plan-lint` filter would pay the 4.0 GB roster compile on that file. The local
step is the cover, and the Orchestrator adjudicates a red CG9 under SM9: it dispatches a repair chunk
or it reverts the appendix edit. No engineer may clear a red CG9 by editing the appendix.

**CLAUDE.md needs no edit, and this record states why.** A gate is a surface that refuses a COMMIT.
`scripts/dod.sh` is the only one, and a workflow job refuses no commit; it reports a red check that
a reviewer acts on. The sentence in CLAUDE.md was therefore true, and the defect T1-5 named is a
LATE SIGNAL and not a false sentence. The path-conditional step removes the lateness, and
specification section 14 now states the definition of a gate so no reader has to infer it.

### 5. `typos` becomes a pass condition through configuration alone

**This record states NO total, and the reason is structural** (critic C1-3, C2-6). Revision 24 stated
one total, revision 25 stated a second, and a run refuted each. A plain `typos` total stated inside a
checked file cannot be stable, because the sentence that states the partition writes the very tokens
it counts into this file and into two more. Each written token is one more finding. A count no rule
reads is a liability, so this decision rests on the CLASSES and on one reproducible measurement.

**FIVE classes COVER every finding, and the class set is EXHAUSTIVE.** The classes are NOT disjoint:
a finding inside an excluded path can also match a word class, and either rule answers it.
Disjointness is a property no rule reads. Each class carries its own rule.

1. **Third-party SMuFL data**, inside `crates/duet/assets/fonts/bravura_metadata.json`, which this
   repository copies and never authors. **Rule: a path exclusion.**
2. **Append-only records**, under `roadmap/duet-v1/reviews/` and `roadmap/duet-v1/research/`, which
   rule CL1c needs nobody to rewrite. **Rule: a second path exclusion.**
3. **The generated protocol word** `Criticals`, in each of its three cases, which
   `check_closure.rs` writes into prose and which no identifier holds. **Rule: three word entries.**
4. **The SMuFL glyph-name identifiers** `ArticAccent`, `ArticStaccato` and `note32ndUp`.
   **Rule: three identifier entries.** `[default.extend-identifiers]` allows each whole name and
   allows nothing else; a word entry for the bare prefix of the first two would hide a real
   misspelling of the word "arctic" anywhere in the repository for ever.
5. **The domain term** `tuplets`. **Rule: one word entry.**

**The one number a reader can reproduce**: the configuration that chunk M90 step 13 mandates exits 0
and prints nothing over the whole tree. That measurement counts nothing, so it cannot change under
its own statement. The magnitude of a plain run is about 500 at the time of this revision; that
figure is an order of magnitude, no rule reads it, and no later author may restate it as a measured
fact.

**A sixth named set is a SUBSET of class two and not a class of its own.** Five findings under the
two excluded directories are genuine prose typos, and this record KNOWINGLY accepts them. Two are a
plural spelling of the word `data`, in `research/linux-macos-platform.md` and
`reviews/critic-spec-r15.md`. Three are truncations in `reviews/critic-spec-r21.md`, which should
read `other`, a two-letter word, and `close`. **Each one is named by its correction and never by its
misspelling**, because `typos` reads this file too. A repair at the site is a defect, because rule
CL1c rests on a review file that nobody rewrites: the closure guard compares the file with a copy
the Critic appended to the plan store, so a spell fix inside the file breaks that compare and puts
the second source back inside the Architect's own hand. **No prose repair follows this decision.**
Chunk M90 replaces the content of the root `typos.toml`, which EXISTS today at 244 bytes, and it
removes the fallback and the `--exclude` flag from `scripts/dod.sh`.

## Consequences

Easier:

- A reader of any guard output knows from the first word whether the guard decided.
- A plan-document author gets a placement finding in one second rather than one push and one
  continuous-integration round trip.
- An engineer who edits no plan document pays nothing for the new gate step.
- PG25 gains a probe group that runs in about one second, so the rule stops being the one rule with
  no probe.
- `typos` fails the gate, so a real typo in new prose is caught at the commit that writes it.

Harder:

- `check-closure` has no machine that runs it on a pull request. Review holds that line, and the
  specification says so plainly rather than naming a job that does not exist.
- The `plan-lint` job still carries the roster compile, so a plan change costs a multi-minute job.
- The two excluded directories hide a future `typos` finding under them. That is correct while both
  are append-only, and it stops being correct the day a chunk edits one. Section 14 states the
  limit.
- The gate step reads the git diff, so a gate run outside a git work tree skips it. Both hooks and
  the continuous-integration checkout are inside one.

## Alternatives rejected

| Alternative | Reason |
|---|---|
| Prefix every exit-1 line with `FINDING: ` | It deletes the rule name from more than sixty recorded lines, and every one would have to be retyped in section 1.9 for no new information. |
| Keep `FAIL:` at exit 1 for prototype fidelity | The prototypes are a historical reference that PG29 reads for rule ids. A word that names two states is worth more than fidelity to a file no gate runs. |
| Upload the plan store as a workflow artifact | It needs a secret and an owner, and the operator forbade a secret. |
| Commit a copy of the store to the repository | It puts rule CL1c's second source back inside the Architect's write scope, so the rule proves nothing. |
| Let `check-closure` exit 0 when the store is absent | It is the vacuous green the document already refuses twice by name. |
| A second `check-roster-generate` subcommand | PG25 is one rule and DR5 gives one rule one probe. Two names would drift. |
| Run the cheap plan guards on every commit | It couples every commit in the repository to one roadmap markdown file, which is the objection section 14 already recorded. |
| Correct the CLAUDE.md sentence instead | The sentence is true under the definition section 14 states. The defect was a late signal, and a rule change would not have made the signal earlier. |
| Repair the 119 roadmap `typos` findings as prose | 116 of them are one protocol word and a domain term, and the rest sit in append-only records that nobody may edit. The repair is a configuration file. |
