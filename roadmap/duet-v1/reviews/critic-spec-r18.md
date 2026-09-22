# Engineering Critic, specification review, revision 18

**Document under review.** The frozen copy at
`/private/tmp/claude-501/-Users-james-Developer-duet/cb581a6d-1ce5-4cb9-8b4e-948690e49339/scratchpad/frozen-r18/duet-v1/`,
`architecture.md` at 13102 lines, md5 `60836911cf712458adde050e08e5b6ed`. The md5 is unchanged after
every run below.

**Scope.** The frozen `architecture.md`, the six records under `adr/`, the nine files under
`tools/`, `design-contract.md`, `product-requirements.md`, and `research/`. I cross-checked against
the repository `CLAUDE.md`, `Cargo.toml`, `.cargo/config.toml`, `clippy.toml`, `deny.toml`,
`scripts/dod.sh`, `.github/workflows/ci.yml`, and `crates/duet/src/`.

**I wrote no repository file. I ran no git command that writes.** `git status --porcelain` in
`/Users/james/Developer/duet` shows one pre-existing untracked path and nothing else. One shared
cargo scratch target ran under the scratchpad and I deleted it. Free disk stayed at 112 GB.

**Verdict: NOT READY.**

---

## What I ran

| Run | Result |
|---|---|
| `placement_check.py architecture.md` | `exit 0`. 41 blocks, 409 placed, `PROBE ROWS: 56  PROBE BAD: 0`, `PHASE PAIRS: 24  PAIR EXEMPT: 24  PAIR BAD: 0  TAIL BAD: 0`, `B.1 SITES: 7  2.3 LISTED: 7  SUPPRESSION BAD: 0  ASSERTED ROOTS: 11  ASSERTED BAD: 0` |
| `probe_run.py architecture.md` | `exit 0`. `PROBES BAD: 0  RECORDED FRAGMENTS: 65  FLOOR: 41  RECORDED FRAGMENTS BAD: 0` |
| `conversion_check.py`, from the repository root | `exit 0`. `MEMBERS: 3  FILES: 6  FINDINGS: 0  EXPECTED MEMBERS: 3  EXPECTED FILES: 5` |
| `probe_conversion.py architecture.md` | `exit 0`. `CONVERSION PROBES BAD: 0  RECORDED FRAGMENTS: 28  FRAGMENTS BAD: 0` |
| `roster_compile.sh architecture.md <scratch> <repo>` | `exit 0`. `ROSTER IMPL MIXED: 0`, `ROSTER IMPL BLOCKS: 50  FLOOR: 50`, `ROSTER IMPLS: 22`, `ROSTER CLIPPY: clean` |
| `probe_roster.sh` | `exit 0`. 15 shapes plus the base. Every plant red. `ROSTER FRAGMENTS: 8  FRAGMENTS BAD: 0` |
| `probe_closure.py architecture.md` | `exit 0`. Four shapes. `CLOSURE PROBES BAD: 0` |
| `closure_check.py architecture.md critic-r16.md closure-r16` | `exit 0`. `GENERATED: 46  ROWS: 46  CLOSURE BAD: 0` |
| `closure_check.py architecture.md critic-r17.md closure-r17` | `exit 0`. `GENERATED: 26  ROWS: 26  CLOSURE BAD: 0` |
| My own attacks A1 to A7, B1 to B3, D1 to D7, F1, G1, H1 | Six guard defeats proven. Each one appears below with its command and its exit code. |

**Your three re-review gates pass.** A member at `crates/target/` with one `exclude` line exits 2
by name (my D1). A false number inside a `FAIL` fragment of the PP25 cell exits 1 and names the
line (my baseline of `probe_roster_text.py` against a foreign output directory prints eight
failures). Each of the twelve dropped revision-16 ids now returns a hit in Appendix C.

**Real progress, stated first.** Revision 18 closed thirteen of the twenty revision-16 rows that
the seventeenth review found open or partial, and it closed them at the sections the rows name. The
`impl From<ConfigError> for GatewayError` conversion is declared and ADR 0004 clause 12a now agrees
with the roster. `MasterView` owns its own `cache` field and its own `build` task, and ADR 0006
decision 8 states the two-cache rule and its reason. Chunk M0 owns the whole `ci.yml`. The
`shellcheck` step, the twelve-rule count, `SmallVec<[Ticks; 13]>` with B113, the meter zones with
B114 and B115, and the four timeout ids in section 5.12 are all correct. `probe_roster.sh` runs
fifteen plants and every one is red, and the new mixed-impl refusal works. CG1b now rejects the two
workspace shapes that defeated it in revision 17.

---

# Closure check: the revision-17 review, row by row

I read the section each row names. I did not read the row alone.

| Row | Row says | I find | Evidence |
|---|---|---|---|
| C17-1 closure appendix drops ids | CLOSED | **PARTIAL** | The rows exist and PG32 refuses a missing row. The guard's denominator is a file no rule identifies (Critical 1), the guard has no port target and no gate (Critical 6), and a blank row passes (Warning 1). |
| C17-2 `finite_to_f32_saturating` phantom | CLOSED | **PARTIAL** | The function is declared at :2303. PG33 does not read a declaration, so the class is open (Critical 3), and the count word is unchecked (Warning 3). |
| C17-3 Master path cache | CLOSED | **CLOSED** | :8398 gives `MasterView` its own `cache` and `build`; ADR 0006 decision 8; B61 states two caches. |
| C17-4 ADR 0004 clause 12a | CLOSED | **CLOSED** | `impl From<ConfigError> for GatewayError` at :11094; the `impl-sites` block holds it; the roster compiles it; clause 12a states the bridge and its crate. |
| C17-5 no chunk owns the `dod` job | CLOSED | **CLOSED** | M0 owns `.github/workflows/ci.yml`, **the WHOLE file**, at :9169. Rung one states `ubuntu-26.04`, `macos-26`, and `libpipewire-0.3-dev`. |
| C17-6 unanchored `target` prune | CLOSED | **CLOSED** | My D1 exits 2 and names `crates/target`. `build_roots` is the one anchored filter. |
| C17-7 text compare exempts PP25 | CLOSED | **PARTIAL** | PP25 is compared now. PP32 is compared by nobody (Critical 5), the new comparer has no floor (Warning 4), it pools every shape (Warning 5), and the row states a false fact about the source (Warning 7). |
| C17-W1 twelve rules, eleven tests | CLOSED | **CLOSED** | :2330 and :2538 both read twelve; ADR 0001 decision 10 reads twelve and twelve. |
| C17-W2 `shellcheck` unnamed | CLOSED | **CLOSED** | :9715 and :9725. |
| C17-W3 `[lints]` gate claim | CLOSED | **CLOSED** | :9700 now states that M0 writes the check into `scripts/dod.sh` and gives the reason. |
| C17-W4 `SmallVec<[Ticks; 12]>` | CLOSED | **CLOSED** | :2562 reads 13; B113 at :1037 states the B43 link. |
| C17-W5 research contradictions | CLOSED | **CLOSED** | ADR 0004:307 names one source and decides; `tokio-util` is 0.7.19 at :1791; `blake3` carries `features = ["pure"]`. |
| C17-W6 CG1b reads two literals | CLOSED | **CLOSED** | `member_globs` reads the root manifest. My D2 exits 2 and names `apps/duet-cli`. |
| C17-W7 CG1b over-approximates | CLOSED | **CLOSED** | My D3 gives a nested fixture crate and the run reports no coverage failure. |
| C17-W8 PG26e leaves eleven roots | CLOSED | **CLOSED** | The `audio-asserted` block holds eleven rows with a reason each. My H1 moves one name in one block and the run exits 1 with two `ASSERTED` lines. |
| C17-W9 PG31 reads written links | CLOSED | **PARTIAL** | PG31b derives the pairs and the `T` skip is gone. Every found pair is exempt and the reasons are unread text (Concerns 1 and 2). |
| C17-W10 the SM6 paragraph is stale | CLOSED | **PARTIAL** | The three numbers are correct today. The row says PG34 holds the phase count and the widest phase. PG34 holds neither (Warning 2). |
| N17-1 three policy gaps | CLOSED | **CLOSED** | M0 owns `clippy.toml` and the ten `doc-valid-idents`; :9195 gives M0 the advisory ignore. |
| N17-2 four timeout ids | CLOSED | **CLOSED** | Section 5.12 holds B24, B25, B26, and B85, and states the B110 exception at :6134. |
| N17-3 meter zones | CLOSED | **CLOSED** | B114 and B115 at :1038 and :1039; section 7.3 states all three zones at :6670. |
| N17-4 the mixed impl block | CLOSED | **CLOSED** | `PP25-mixed` is red in my run. `ROSTER IMPL MIXED: 0` prints on every roster run. |
| N17-5 no fragment denominator | CLOSED | **PARTIAL** | `probe_run.py` prints 65 against a floor of 41. `probe_roster_text.py` prints a count and no floor (Warning 4). |
| N17-6 the pooled oracle | CLOSED | **PARTIAL** | `probe_run.py` compares each row first. `probe_roster_text.py` pools (Warning 5). |
| N17-7 a chunk in the last phase | CLOSED | **CLOSED** | PG34 `tail_phase_audit` refuses a last phase with a width above zero and a last phase that names a chunk. |
| N17-8 a symbolic link | CLOSED | **CLOSED** as stated | :2400 states the limit. My D7 confirms the hole stays. The row promised prose and the prose is there. |
| N17-9 the B.1 table is read by no rule | CLOSED | **PARTIAL** | PG33 reads it against a prose sentence and never against a declaration (Critical 3). |

**Revision-17 totals.** Closed 17. Partial 8. Open 0. One row (N17-8) closes a finding with a
stated limit and not a machine, which is what that row promised.

# Closure check: the revision-16 review

The seventeenth review verified 30 of the 46 rows as closed. I re-read a sample of ten of those and
found each one still true. The table below covers the twenty rows that were open or partial then.

| Row | Row says | I find | Evidence |
|---|---|---|---|
| C16-2 CG1b vacuous green | CLOSED | **PARTIAL** | Two shapes fixed. A member whose directory name opens with a dot is still invisible (Critical 4). |
| C16-12 three pairs share a phase | CLOSED | **PARTIAL** | PG31b exists and derives the pairs. See Concerns 1 and 2. |
| C16-14 phantom function | CLOSED | **PARTIAL** | Declared. The guard does not hold the class (Critical 3). |
| C16-15 Master path cache | CLOSED | **CLOSED** | As C17-3 above. |
| C16-16 ADR 0004 clause 12a | CLOSED | **CLOSED** | As C17-4 above. |
| C16-17 no chunk owns the CI file | CLOSED | **CLOSED** | As C17-5 above. |
| C16-W1 PG26d closes a shape | CLOSED | **CLOSED** | As C17-W8 above. |
| C16-W2 recorded output uncompared | CLOSED | **PARTIAL** | As C17-7 above. Critical 2 and Critical 5 are two live instances. |
| C16-W14 eleven against twelve | CLOSED | **CLOSED** | As C17-W1 above. |
| C16-W15 `shellcheck` | CLOSED | **CLOSED** | As C17-W2 above. |
| C16-W16 the `[lints]` claim | CLOSED | **CLOSED** | As C17-W3 above. |
| C16-W17 `SmallVec` capacity | CLOSED | **CLOSED** | As C17-W4 above. |
| C16-W18 research contradictions | CLOSED | **CLOSED** | As C17-W5 above. |
| N16-9 three policy gaps | CLOSED | **CLOSED** | As N17-1 above. |
| N16-10 the 5.12 timeout table | CLOSED | **CLOSED** | As N17-2 above. |
| N16-11 meter zones | CLOSED | **PARTIAL** | B114 and B115 exist. The row claims a citation at 15.16 that the document does not hold (Concern 5). |
| C16-1, C16-3 to C16-11, C16-13, C16-W3 to C16-W13, N16-1 to N16-8 | CLOSED | **CLOSED** | Verified in revision 17 and re-sampled here. C16-1 is the one exception: section 1.9 now records a roster run that this document does not produce (Critical 2). |

**Revision-16 totals.** Closed 40. Partial 6. Open 0.

---

# CRITICAL

## CRITICAL 1. The closure guard proves that the block matches an input file, never that the input file is the review

**OBSERVATION.** `closure_check.py` takes the review path on the command line. It generates the id
list from whatever file that path names. No rule, no block, and no sentence of the document records
the identity of the review file. Appendix C.20 records the md5 of the DOCUMENT that the sixteenth
review read (`architecture.md:12985`). It records no digest of `critic-r16.md`, and C.21 records no
digest of `critic-r17.md`.

I built a draft copy of `critic-r17.md`. I removed Criticals 6 and 7, Warnings 9 and 10, and
Concerns 8 and 9, which is a truncated tail per family, exactly the revision-17 defect. I then built
the appendix block from that draft's generated list:

```
$ python3 tools/review_ids.py draft.md | tail -1
CRITICALS: 5   WARNINGS: 8   CONCERNS: 7   TOTAL: 20
$ python3 tools/closure_check.py draft-doc.md draft.md closure-r17
REVIEW: draft.md   BLOCK: closure-r17   GENERATED: 20   ROWS: 20   CLOSURE BAD: 0
EXIT=0
$ python3 tools/review_ids.py critic-r17.md | tail -1
CRITICALS: 7   WARNINGS: 10   CONCERNS: 9   TOTAL: 26
```

**CLAIM.** The guard written to end a dropped finding accepts a self-consistent green closure over
20 of 26 findings, at production defaults, in one command.

**ARGUMENT.** C17-1 was a truncated READ of a review. PG32 prevents a truncated TRANSCRIPTION of a
read. Those are two different failures. The guard asserts `rows == generated(input)`. It never
asserts `input == the review`. The denominator is therefore taken on trust from the same person who
writes the rows, which is the one party the appendix exists to check. The document's own limit
paragraph at :843 states that the review file "sits outside the repository", so no gate and no
reader can recover the file the run used. A reader who sees `CLOSURE BAD: 0` in a commit message
learns nothing about which review was closed.

The document already holds the correct pattern one line away: C.20 records the md5 of the frozen
document. The same discipline applied to the review file closes this.

**EVIDENCE.** `tools/closure_check.py`:100-120 (`main` takes `argv[2]` and opens it);
`tools/review_ids.py`:75-100; `architecture.md`:833-843, :12985, :12996-13002. My run above.

**WHAT SHOULD CHANGE.** Record the review file's own digest in each closure block, as prose the
guard reads, and make `closure_check.py` compare it with the digest of the file it was handed.
Refuse a mismatch. State the digest of `critic-r16.md`, `critic-r17.md`, and this review in C.20,
C.21, and C.22.

---

## CRITICAL 2. Section 1.9 records a roster run that this document does not produce, and no machine reads it

**OBSERVATION.** Lines 1440 to 1452 hold a fenced transcript that the prose at :1437 introduces
with "The roster compile on this revision prints:". I ran `roster_compile.sh` over this frozen
document and compared the two.

```
RECORDED AT architecture.md:1440-1452     |  MY RUN
ROSTER CRATES:   17                        |  ROSTER CRATES:   17
ROSTER ITEMS:    395     FLOOR: 395        |  ROSTER ITEMS:    395     FLOOR: 395
(absent)                                   |  ROSTER IMPL MIXED: 0
ROSTER IMPL BLOCKS: 49     FLOOR: 49       |  ROSTER IMPL BLOCKS: 50     FLOOR: 50
ROSTER IMPLS:    21                        |  ROSTER IMPLS:    22
```

The document's own PP25 cell states the clean value. Its planted-failure text reads "the roster
parsed 49 impl blocks and the `impl-sites` block lists 50", and the plant deletes one impl block.
The clean count is therefore 50 and the recorded 49 is wrong. My `probe_roster.sh` run produced the
same 49-against-50 line as a red result.

**CLAIM.** This is CR-13, C16-1, and C17-7 for the third time, in the revision whose closure row
says that every harness compares its recorded text.

**ARGUMENT.** The missing line is `ROSTER IMPL MIXED`, which is the counter that the N17-4 closure
row cites as the mechanism. The section that records the run does not show the counter the closure
row says the run prints. A reader who audits N17-4 at section 1.9 sees no such line.

No machine reads this block. `probe_run.py` reads the section 1.9 probe table cells.
`probe_roster_text.py` reads the PG25 row's `Recorded result` cell alone, by an explicit lookup on
`PP25`. This transcript is neither. The revision therefore added two text comparers and left the one
fenced transcript that has now been false three times outside both of them.

**EVIDENCE.** `architecture.md`:1437-1452 against my `roster_compile.sh` output;
`architecture.md`:1304, the PP25 cell; `tools/probe_roster_text.py`:70-84 (`re.search(r"\bPP25\b",
row[2])`); `tools/probe_run.py`:780-800.

**WHAT SHOULD CHANGE.** Correct the three numbers and add the missing line. Then give this
transcript to `probe_roster_text.py`: read the fenced block that follows the anchor sentence and
compare every line with the run, exactly as the PP25 cell is compared. A transcript of a run belongs
to the harness that performs the run.

---

## CRITICAL 3. PG33 holds Appendix B.1 against a prose sentence and never against a declaration, so the phantom-function class is open

**OBSERVATION.** `suppression_audit` takes the Site column of the `b1-convert` block and the names
inside one section 2.3 sentence, and compares the two sets. It reads no Rust block. Its own docstring
states the opposite: "Every site must also be a function some Rust block of this document declares."
Section 2.3 at :2308 states the same thing to the reader: "**PG33 holds this list and the Appendix
B.1 suppression-reason table to ONE SET in both directions**, so a row with no declaration and a
declaration with no row are each a red run."

I deleted the declaration and left the prose sentence and the B.1 row in place:

```
$ python3 - ...  # remove the line `pub fn finite_to_f32_saturating(value: Finite) -> f32;`
$ python3 tools/placement_check.py b3.md
B.1 SITES:  7   2.3 LISTED: 7   SUPPRESSION BAD: 0   ...
EXIT=0
```

**CLAIM.** The shape of C16-14 and C17-2, a suppression row for a function no section declares, is
still green under the rule written to end it, and two sections state that it is not.

**ARGUMENT.** The finding itself is closed on content: the function is declared today. The CLASS is
not. The list PG33 reads is a sentence of prose that an author types, so a name can enter the list
without a declaration and satisfy both directions of the rule. The defect survived seventeen
revisions because the B.1 table was outside every rule. It can now survive again because the
sentence is outside every rule. Nothing else catches it: a function is not a type, so no placement
counter moves when the declaration disappears.

**EVIDENCE.** `tools/placement_check.py`, `suppression_audit`, the whole function;
`architecture.md`:2306-2312, :11813 onward, the `b1-convert` block. My run above.

**WHAT SHOULD CHANGE.** Take the third set from the declarations. Hold three sets to one set: the
B.1 Site column, the section 2.3 sentence, and the `#[expect(clippy::as_conversions, ...)]` sites
that the Rust blocks of section 2.3 declare. Refuse a name that any one of the three omits. Then the
docstring and the section 2.3 sentence become true.

---

## CRITICAL 4. CG1b cannot see a workspace member whose directory name opens with a dot, and Cargo can

**OBSERVATION.** `expected_members` calls `glob.glob`. Python's `glob` does not match a leading dot.
Cargo expands a `members` glob with the Rust `glob` crate, whose default `MatchOptions` sets
`require_literal_leading_dot` to false, so Cargo does match one.

I proved that Cargo treats such a directory as a member:

```
$ cat Cargo.toml
[workspace]
members = ["crates/*", "tools/*"]
$ cargo metadata --no-deps --format-version 1 | ...
CARGO MEMBERS: ['fixture', 'duet-a', 'x']        # crates/.fixture IS a member
```

I then added one `exclude` line and put a bare cast inside:

```
=== D4: crates/.fixture excluded, holds `v as f32` ===
MEMBERS: 2   FILES: 2   FINDINGS: 2   EXPECTED MEMBERS: 2   EXPECTED FILES: 2
(no coverage FAIL; the two findings are the two decoy casts in the real members)
```

**CLAIM.** One directory name and one `exclude` line give a clean CG1b run over a crate that holds a
cast. This is the C16-2 and C17-6 class for the third revision running.

**ARGUMENT.** The class recurs because each revision writes its own approximation of Cargo's member
resolution. Revision 11 tested a path component at every depth and lost a module named `target`.
Revision 17 pruned any directory named `target` at any depth. Revision 18 uses a glob library whose
dot rule differs from Cargo's. Each fix repaired one instance and kept the method that produces the
class.

The stated backstop is absent for this shape, by the document's own corrected wording: an excluded
member is not a workspace member, so `cargo clippy --workspace` never lints it.

`expected_files` carries no dot rule of its own, but it walks only the members that
`expected_members` returns, so the hole propagates.

**EVIDENCE.** `tools/conversion_check.py`, `expected_members`, the `glob.glob` call;
`tools/conversion_check.py`, `member_globs`; `architecture.md`, the CG1b paragraph of section 2.3.
My D4 and D4b runs above.

**WHAT SHOULD CHANGE.** Stop approximating the glob. Expand each pattern with `os.scandir` over the
pattern's parent directory, which has no dot rule, or pass `include_hidden=True`. Better, ask Cargo:
run `cargo metadata` once per candidate directory and let Cargo decide what a member is. State in the
CG1b paragraph which library expands the pattern and that its dot rule matches Cargo's.

---

## CRITICAL 5. The PP32 recorded cell is compared by no harness, and the source says the opposite

**OBSERVATION.** `probe_run.py` holds `ROSTER_PROBES = {"PP25", "PP32"}` and skips both ids in
`text_failures`. The comment above that set reads:

> **Neither row is exempt from a compare**, which is the whole of critic C17-7 ... PP32 needs a
> review file, so `probe_closure.py` runs it and compares its own recorded lines.

`probe_roster_text.py` compares the `PP25` row only. `probe_closure.py` compares each shape's first
output line with a `recorded` string that is a literal inside its own `SHAPES` table. It never opens
the section 1.9 probe table.

I planted a false line inside the PP32 recorded cell of section 1.9:

```
planted: "CLOSURE:   C99-777: a line no run of any harness has ever produced"
-- placement_check.py   EXIT=0
-- probe_closure.py     CLOSURE PROBES BAD: 0
-- probe_run.py         PROBES BAD: 0     RECORDED FRAGMENTS: 65   FLOOR: 41   RECORDED FRAGMENTS BAD: 0
```

**CLAIM.** The revision that closed C17-7 created one new exempt row and wrote a comment that
denies it.

**ARGUMENT.** C17-7 was one row exempted from `probe_run.py` and given to nobody. PP32 is one row
exempted from `probe_run.py` and given to a harness that compares against its own literal, not
against the document. The phrase "compares its own recorded lines" is exactly true and is exactly
the defect: its own, and not the document's.

The oracle is also weak on its own terms. `probe_closure.py` compares one line per shape. The PP32
cell records four output lines, three of which no harness reads at all.

**EVIDENCE.** `tools/probe_run.py`:710-722 (the comment and the set), :787;
`tools/probe_closure.py`:133-151; `tools/probe_roster_text.py`:70-84;
`architecture.md`:1313, the PP32 cell. My run above.

**WHAT SHOULD CHANGE.** Make `probe_closure.py` read the PP32 row of the section 1.9 probe table and
compare every recorded output fragment of that cell with its own runs, as `probe_run.py` does for the
other 41 rows. Print the compared count and a floor. Then the comment is true.

---

## CRITICAL 6. PG32 has no port target, no owner, and no gate

**OBSERVATION.** `tools/closure_check.py`:3-5 says the script is a "Prototype for `cargo xtask
check-closure <document> <review> <block>`, which chunk M0 ports to
`tools/xtask/src/check_closure.rs`."

A grep of the whole frozen `architecture.md` for `check_closure` and for `check-closure` returns
**zero hits**. M0's write scope at :9169 lists `tools/xtask/src/{check_conversions,check_placement,
check_roster}.rs` and no fourth file. The `plan-lint` job at :9772 runs `check-placement`,
`check-roster`, and `cargo nextest run -p xtask --test probes`. It runs no closure guard. Rung one
at :9700-9718 names no closure guard. `review_ids.py`, `closure_check.py`, and `probe_closure.py`
appear in no chunk's write scope.

**CLAIM.** The guard that answers the worst finding of the last review is never run by any gate, and
no chunk can create the file it is meant to become.

**ARGUMENT.** This is the C16-17 and C17-5 shape applied to the plan's own guard: a file that the
plan requires and that no write scope holds. The probe PP32 does reach CI, because
`tools/xtask/tests/probes.rs` ports every section 1.9 probe and `probe_closure.py` builds its own
throwaway review. So CI will prove forever that the guard works, over a review CI invents, while the
guard is never applied to Appendix C. A probe that runs without its guard is a test of a mechanism
nothing uses.

Combined with Critical 1, PG32's green is a claim by one person, over a file nothing identifies,
from a tool no gate runs.

**EVIDENCE.** `tools/closure_check.py`:3-5; `architecture.md`:9169, :9772, :9700-9718, and the zero
grep count for `check_closure`.

**WHAT SHOULD CHANGE.** Decide one of two things and state it. Either add
`tools/xtask/src/check_closure.rs` to M0's write scope, commit the review files under
`roadmap/duet-v1/reviews/`, and add the command to the `plan-lint` job. Or delete the port sentence
from `closure_check.py`, state in PG32 that the guard is a review-time tool that no gate runs, and
say who runs it and when.

---

# WARNING

## WARNING 1. A closure row may say nothing at all and stay green

**OBSERVATION.** `audit` checks four things about a row: the id is generated, the id is not extra,
the count sentence matches, and a row whose State cell starts with `CLOSED` has a non-empty Section
cell. It checks no vocabulary for the State cell and no content for any other cell.

```
A1: `| C17-3 |  |  |  |`                      -> GENERATED: 26  ROWS: 26  CLOSURE BAD: 0  EXIT=0
A2: `| C17-4 | x | **CLOSED** | 99.99 |`      -> CLOSURE BAD: 0  EXIT=0   (no section 99.99 exists)
A3: `| C17-5 | x | **PENDING** |  |`          -> CLOSURE BAD: 0  EXIT=0
```

**CLAIM.** A finding can still be emptied out of the record. The row count and the count sentence do
not move.

**ARGUMENT.** A1 is the important one. PG32 forbids the removal of a row and permits the removal of
everything the row says. A truncated read cannot produce that shape, so the primary C17-1 failure
mode is genuinely closed, and that is why this is a Warning and not a Critical. A deliberate
evasion, and an author who runs out of time on the last five rows, both produce it. A3 shows the
State cell accepts any word, so a whole block marked `PENDING` with empty Section cells is a green
run over an appendix that asserts nothing.

A2 is weaker, because the rule's own limit at :840 says the guard decides that a row EXISTS and not
that the closure is true. The limit paragraph still promises the reader that "the row names the
section so the check is one lookup". A section number the document does not hold defeats the one
lookup.

**EVIDENCE.** `tools/closure_check.py`, `audit`, the whole function; `architecture.md`:838-843. My
A1, A2, and A3 runs.

**WHAT SHOULD CHANGE.** Require a non-empty Finding cell, a non-empty State cell, and a State cell
from a fixed set of three words. Require a section reference that a heading of this document holds.
Each one is a lookup the guard already has the data for.

## WARNING 2. PG34 holds the tail of the phase table and nothing else, and two places say it holds three numbers

**OBSERVATION.** `tail_phase_audit` reads the `phase-table` block and makes exactly two checks: the
last width is zero, and the last phase names no chunk. Its own docstring says more: "The rule reads
the `phase-table` block and fails when the phase count, the widest phase, or the last phase's width
is stated anywhere in the document with a different number". The C17-W10 closure row at :13086 says
"1.5 PG34, which holds the phase count, the widest phase, and the tail to the table".

I restored the revision-17 defect in the SM6 cost paragraph:

```
B1: "Sixteen phases replace ten ... falls from eleven to seven"
 -> "Thirteen phases replace ten ... falls from eleven to eight"
    placement_check.py  TAIL BAD: 0   EXIT=0
```

**CLAIM.** C17-W10's closure row names a mechanism that does not exist. The three numbers are
correct today by hand, not by machine, and they were wrong by hand in the last revision.

**ARGUMENT.** The document's own SM6 prose at :9046 is careful and honest: "**Every number in this
bullet comes from the phase table**, and PG34 holds the tail of that table". The closure row and the
guard docstring both overstate it. A reader who audits C17-W10 reads the row, believes a machine
holds the count, and does not check the paragraph. That is the exact reliance the appendix exists to
create, spent on a claim that is false.

**EVIDENCE.** `tools/placement_check.py`, `tail_phase_audit`, the docstring against the body;
`architecture.md`:9043-9047, :13086. My B1 run.

**WHAT SHOULD CHANGE.** Add the two checks the docstring already claims: the widest width and the
row count, each compared with every number word in the SM6 bullet. Or correct the docstring and the
C17-W10 row to say that PG34 holds the tail alone.

## WARNING 3. PG33 prints both counts and never compares them with the count word

**OBSERVATION.** The run prints `B.1 SITES: 7     2.3 LISTED: 7`. Section 2.3 opens the list with
"Seven functions carry a suppression:". The regex that finds the sentence captures the count word in
`\b\w+` and discards it.

```
B2: "Seven functions carry a suppression" -> "Six functions carry a suppression"
    SUPPRESSION BAD: 0   EXIT=0
```

**CLAIM.** The count disagreement that formed half of C17-2 is unguarded, in the rule written to
close C17-2.

**ARGUMENT.** C17-2 recorded three disagreeing counts: "Six functions" at :2214, "Seven
suppressions" at :11813, and a twenty-site total that summed to nineteen at :11898. The revision
fixed all three by hand. The guard holds the two sets and prints the two sizes and compares neither
with the word the reader reads first. The next author who adds an eighth suppression will update the
list and may not update the word.

**EVIDENCE.** `tools/placement_check.py`, `suppression_audit`, the `re.search` pattern;
`architecture.md`:2306. My B2 run.

**WHAT SHOULD CHANGE.** Capture the count word, map it to a number, and refuse a word that differs
from the list length. Do the same for the twenty-site total of Appendix B.1.

## WARNING 4. `probe_roster_text.py` prints a denominator and enforces no floor, so zero compared fragments is a clean run

**OBSERVATION.** `main` counts the fragments, reports the missing ones, and returns 1 only when one
is missing. There is no floor. N17-5 asked for a floor and `probe_run.py` has one at
`FLOOR: 41`. The new comparer has none.

```
G1: the PP25 recorded cell rewritten with no `FAIL` and no ": error" fragment
    ROSTER FRAGMENTS: 0     FRAGMENTS BAD: 0     EXIT=0
```

**CLAIM.** The machine built to answer C17-7 reports success over an empty compared set.

**ARGUMENT.** "Compared 0 fragments and found 0 failures" and "compared 8 and found 0" print almost
the same line and mean opposite things. The filter that selects an output fragment is two literal
tests, `startswith("FAIL")` and `": error" in piece`. Any change to the roster's message shapes
silently empties the set. `placement_check.py` does catch the shape I planted, with `PROBE: PG25: the
recorded cell states no exit code`, so a second guard limits this today. That is why it is a Warning.
It does not limit a cell that keeps its exit claim and loses its output fragments.

**EVIDENCE.** `tools/probe_roster_text.py`, `main` and `output_lines`; `tools/probe_run.py`:965-979.
My G1 run.

**WHAT SHOULD CHANGE.** Give `probe_roster_text.py` a floor derived from the shape count, exactly as
`probe_run.py` derives its floor from the row count, and fail below it.

## WARNING 5. `probe_roster_text.py` pools every shape's output, which is the defect the same revision removed from `probe_run.py`

**OBSERVATION.** `main` reads every `.out` file in the output directory, joins them into one string,
and asks whether each fragment appears anywhere in that string. `probe_run.py` no longer does this:
`text_failures` compares a fragment against its own row's output first and reports a pool-only match
with its own reason.

**CLAIM.** N17-6 is closed in one harness and re-opened in the harness this revision wrote.

**ARGUMENT.** The PP25 cell records four failure lines from four named shapes, `PP25-eq`,
`PP25-drop`, `PP25-floor`, and `PP25-impl`. The pool cannot tell which shape produced which line, so
a recorded line attributed to the wrong shape is green. The N17-6 closure row says the residual is
"stated at its own site". `probe_run.py` states it. `probe_roster_text.py` states nothing.

**EVIDENCE.** `tools/probe_roster_text.py`, `main`, the `pool` variable; `tools/probe_run.py`,
`text_failures`, the docstring and the `own` map.

**WHAT SHOULD CHANGE.** Attribute each `.out` file to its shape, compare each recorded fragment
against the shape that the cell says produced it, and fall back to the pool with a stated reason, as
`probe_run.py` does.

## WARNING 6. The heading parser counts a quoted heading, and it refuses a review with an empty family

**OBSERVATION.** `HEADING` matches `^##\s+(CRITICAL|WARNING|CONCERN)\s+(\d+)\b` over the raw text.
It does not remove a fenced code block. A review that quotes a heading from an earlier review inside
a fence contributes that heading to the count.

```
A5:  a fence holding "## CRITICAL 9" in a 7-Critical review
     -> FAIL: the CRITICAL numbers are not 1 to 8: [1,2,3,4,5,6,7,9]   (fail-closed)
A5b: a fence holding "## CRITICAL 8" in a 7-Critical review
     -> CRITICALS: 8 ... TOTAL: 27
     -> closure_check: CLOSURE BAD: 2, one of them "C17-8: the review states it and the block holds no row"
A6:  a review with no CONCERN heading
     -> FAIL: the review states no CONCERN heading; the generator is fail-closed.   EXIT=2
```

**CLAIM.** A normal review formatting choice forces the Architect to write an appendix row for a
finding that does not exist, and a clean family makes the whole review unrecordable.

**ARGUMENT.** A5b is the sharper half. The failure direction is safe, because the guard demands more
rows and not fewer. The consequence is not safe: the only way to reach green is to add a row for
`C17-8`, which is a fabricated closure of a finding no review made. A review of a review quotes
headings; that is what a closure check is for.

A6 is a process defect. The generator refuses an empty family by design, and the docstring states
the reason for CRITICAL. The reason does not hold for CONCERN or WARNING. The effect is that a
review with no Concern cannot be recorded in Appendix C at all, so the reviewer must produce one.
The brief for this very review carries that pressure already: it instructs me to number each family
from one with no gaps. A guard that shapes the finding count of the review it audits is a guard that
has left its subject.

**EVIDENCE.** `tools/review_ids.py`, `HEADING` and `ids_of`. My A5, A5b, and A6 runs.

**WHAT SHOULD CHANGE.** Strip fenced code blocks before the headings are read. Allow an empty
WARNING or CONCERN family, and keep the refusal for a review with no heading of any family at all.
State that change in PG32, because it changes the count sentence for a clean review.

## WARNING 7. The C17-7 closure row states a fact about the source that the source refutes

**OBSERVATION.** The C17-7 row at :13076 reads in part: "`ROSTER_PROBES` deleted from
`probe_run.py`". `tools/probe_run.py`:719 reads `ROSTER_PROBES = {"PP25", "PP32"}`. The set is not
deleted. It grew by one member.

**CLAIM.** A closure row states a checkable fact, a reader checks it in one lookup, and the fact is
false.

**ARGUMENT.** The row's own mechanism list is what the PG32 limit paragraph tells the reader to use:
"the row names the section so the check is one lookup". The lookup here returns the opposite of the
row. The substance is arguable, because the ids moved to other harnesses rather than became
compared everywhere. Critical 5 shows one of the two moves did not deliver a compare. A row that
misstates the code loses the reader's ability to tell the two apart.

**EVIDENCE.** `architecture.md`:13076; `tools/probe_run.py`:710-722.

**WHAT SHOULD CHANGE.** Rewrite the cell to say what the code does: `ROSTER_PROBES` names the two
rows another harness owns, and each harness compares the row it owns. Then make that true, per
Critical 5.

## WARNING 8. CG1b's pattern expansion does not reach a sibling of a member that the manifest lists by full path

**OBSERVATION.** `expected_members` builds its pattern set from each `members` entry plus that
entry's parent joined with `/*`. For an entry at depth two, the parent is at depth one, so no
pattern reaches depth one itself.

```
D5: members = ["crates/group/a", "tools/*"], exclude = ["crates/standalone"], cast inside
    MEMBERS: 2  FILES: 2  FINDINGS: 2  EXPECTED MEMBERS: 2  EXPECTED FILES: 2   (no coverage FAIL)
```

**CLAIM.** A second manifest shape hides a whole crate from the rule.

**ARGUMENT.** This is the same class as Critical 4 and it needs a manifest shape the plan does not
use today. The root manifest globs `crates/*` and `tools/*`, so the shape is latent. The plan creates
fifteen crates and the manifest will be edited by M0 through M8. The rule's comment explains the
parent expansion as "the rule CLAUDE.md states", which is a rule about one level under two named
trees, not a property of an arbitrary `members` array.

**EVIDENCE.** `tools/conversion_check.py`, `expected_members`, the `patterns` loop. My D5 run.

**WHAT SHOULD CHANGE.** Fix this with Critical 4's fix: derive the member set from Cargo, not from a
pattern rewrite. If the rewrite stays, state its assumption in the CG1b paragraph: every `members`
entry sits one level under a tree the array also globs.

---

# CONCERN

## CONCERN 1. The 24 exemption reasons are text no rule reads, and one of them names the wrong chunk

`phase_pair_audit` takes `row.split()[:2]` from each `phase-pair-exempt` row and discards the rest.
The reason column is free text. Row 6 reads:

```
C1  N2  C2 adds the `duet-media` entry, one phase later, and 13.4 carries `N1 before C2`
```

The pair is `C1` and `N2`. The reason's subject is `C2`. The intended argument is probably that C1
adds no `duet-media` entry and C2 adds it one phase later, but the row does not say that. A reviewed
exemption whose reason names a different chunk cannot be reviewed. Rewrite the cell and add a rule
that the reason must name both chunks of its own pair. The follow-up belongs in
`roadmap/duet-v1/architecture.md` section 13.3, beside the block.

## CONCERN 2. PG31b rejects nothing today, because every pair it finds is exempt

The run prints `PHASE PAIRS: 24     PAIR EXEMPT: 24     PAIR BAD: 0`. The rule's found set and its
allow-list are the same set. The rule is a ratchet: a 25th pair is red. That is real value and it is
less than the rule site claims. The rule's stated limit is honest about why it cannot decide binding.
SM8 states when an edge binds, and the chunk table states the declarations each chunk writes, so a
machine could decide most of the 24. Record this as accepted debt with the SM8 reference, so the next
revision does not read `PAIR BAD: 0` as a proof.

## CONCERN 3. The rule-blocks registry says PG26f reads three blocks and the rule reads two

Line 1549 reads `| PG26f | audio-owned, audio-asserted, snapshot-table |`. `audio_asserted_audit`
takes the root set, the reachable set, and the asserted set. It never reads `snapshot-table`. The
registry is the reader's map of which block feeds which rule. Correct the row, or make PG26f check
the two `published;` rows against the snapshot table, which the block's own reason text already
claims.

## CONCERN 4. Every budget this revision added counts the closure appendix as a use site

B113's `Used by` cell reads "2.4, C.20, C.21". B114 and B115 read "7.3, C.20, C.21". The appendix
citations are closure rows that mention the id, not sites that use the value. B114 and B115 therefore
have one real use site each. PG30 accepts this because it is a symmetric citation rule and not a use
rule. N16-7 asked for real use sites. Exclude C.20 and C.21 from the `Used by` column, as PG30
already excludes the budget table itself and an ADR reference.

## CONCERN 5. The N16-11 closure row claims a citation that the document does not hold

The row at :13052 reads "1.6 B114 and B115, the two zone thresholds, cited by 7.3 and 15.16". A grep
for `B114` and `B115` returns hits at :1038, :1039, :6670-6672, and the two closure rows. Section
15.16 cites neither, and the `Used by` cells name only 7.3. `USED BY BAD: 0`, so the guard agrees
with the cells and disagrees with the row. Correct the row.

## CONCERN 6. Two filters do one job, and one import is dead

`probe_run.py` selects a recorded output fragment with `OUTPUT_HEAD =
^(FAIL|usage:|[A-Z][A-Z0-9 ]*:)`. `probe_roster_text.py` selects one with `startswith("FAIL")` or
`": error" in piece`. The second is narrower, so a counter line such as `ROSTER CRATES: 17` recorded
in the PP25 cell would be compared by the first rule and skipped by the second. No such fragment sits
in that cell today, so this is latent. `closure_check.py` imports `subprocess` and never calls it,
which `cargo clippy` will reject in the ported Rust form. Share one filter between the two harnesses
and delete the import.

---

# Reactive assessment

| Property | Verdict | Why |
|---|---|---|
| Responsive | **PASS** | `MasterView` owns its own `cache` field and its own `build` task, so all four work areas have a reachable render path. Section 5.12 now holds B24, B25, B26, and B85 beside B15 to B23, B27, B95, B105, B108, and B110, and it states why B110 is there. I traced every cross-boundary wait in the table to a bound. |
| Resilient | **PASS** | `impl From<ConfigError> for GatewayError` is declared at :11094, the `impl-sites` block registers it, and the roster compiles it. ADR 0004 clause 12a now states the bridge and its crate. `finite_to_f32_saturating` is declared, so no `# Errors` contract names an absent function. `unsafe` stays denied and no chunk needs it. |
| Elastic | **PASS** | `pending_configure` holds one request, B109 bounds the channel, B105 bounds the handoff ring, and B61 bounds each of the two path caches. I found no unbounded collection and no unbounded channel in the roster. |
| Message Driven | **PASS** | The engine seam is messages throughout. The last shared-state read across a view boundary is gone: ADR 0006 decision 8 gives `MixView` and `MasterView` one cache each and states that a shared cache would need a third owner and a notification path between two views that never render in the same frame. |

**All four properties pass for the first time in this review series.** The design of the system is
sound. Every finding above is about the plan's own guard layer, which is a different subject: it
decides whether a reader can trust that the design is what the appendix says it is.

---

# Verdict

**NOT READY.** Revision 18 did the most engineering of any revision I have reviewed. It closed
seventeen of the twenty-six revision-17 rows completely, it closed thirteen of the twenty
revision-16 rows that were open or partial, and it did so at the sections the rows name. All four
Reactive properties pass. The type roster compiles clean, the fifteen roster plants are red, the
placement guard is green over 409 placed names, and CG1b now rejects both workspace shapes that
defeated it last revision.

The single biggest risk is **Critical 1**. The machine written to end a dropped finding takes its
denominator from a file that nothing identifies. I handed it a truncated copy of the last review and
it reported a complete closure over 20 of 26 findings, at production defaults, in one command. That
is the C17-1 defect reproduced against the rule built to close it. **Critical 6** compounds it: the
guard has no port target, no owner, and no gate, so its green is a hand run by one person over an
unidentified file.

Close behind sit three repeats of the same shape. **Critical 2**: section 1.9 records a roster run
that this document does not produce, for the third time in the series, and no machine reads that
transcript. **Critical 3**: the rule that answers the phantom-function finding never reads a
declaration, so the class stays open while two sections state that it is closed. **Critical 4**: one
directory name whose first character is a dot takes an excluded crate outside CG1b, which is the
third distinct path-matching asymmetry to produce the C16-2 class in three revisions.

The pattern across all six Criticals is one method, not six mistakes. Each new rule is written to
refuse the exact instance the last review produced, and each one takes one input on trust: the
review file, the fenced transcript, the prose list, the glob library, the harness literal, the port
sentence. **The remedy is the same in every case: take the second source from a place the author
does not type.**

The weakest Reactive property is none of the four. The weakest property of this document is the
audit trail, and that is the property the whole of Appendix C exists to hold.

---

# Blocking list

Every Critical blocks plan authoring. Every Warning must close before FINISH. Fix in this order.

1. **Critical 1** — Record each review file's digest in its closure block and make `closure_check.py`
   compare it. Add the digest for `critic-r16.md`, `critic-r17.md`, and this review.
2. **Critical 6** — Give `check_closure.rs` a write scope and a gate line, or state plainly that
   PG32 is a review-time tool and name who runs it.
3. **Critical 2** — Correct the three numbers and the missing line at :1440-1452, then give that
   transcript to `probe_roster_text.py`.
4. **Critical 5** — Make `probe_closure.py` read the PP32 row of section 1.9 and compare every
   recorded fragment of it.
5. **Critical 3** — Give PG33 a third set from the section 2.3 declarations.
6. **Critical 4** — Expand the `members` patterns with a method whose dot rule matches Cargo's, or
   ask Cargo.
7. **Warnings 1, 2, 3** — The empty closure row, the PG34 overclaim, and the unchecked count word.
8. **Warnings 4, 5, 6** — The missing floor, the pooled oracle, and the heading parser.
9. **Warnings 7, 8** — The false `ROSTER_PROBES` claim and the deep-member pattern gap.

**Re-review condition.** Do not send revision 19 for review until four commands pass. First, a
closure run against a review file whose digest the block does not hold exits non-zero. Second, a
`roster_compile.sh` run and the fenced transcript at section 1.9 agree line for line, by a command
that compares them. Third, the deletion of the `finite_to_f32_saturating` declaration, with the B.1
row and the 2.3 sentence left in place, exits non-zero. Fourth, a workspace with an excluded member
at `crates/.fixture` exits 2 by name. Each one is one command.

---

# For the closure block of revision 19

`tools/review_ids.py` parses this review as `CRITICALS: 6   WARNINGS: 8   CONCERNS: 6   TOTAL: 20`,
with the prefix `C18`. Appendix C.22 must hold one row for each of C18-1 to C18-6, C18-W1 to C18-W8,
and N18-1 to N18-6, and this exact count sentence:

    It returned 6 Criticals, 8 Warnings, and 6 Concerns, and this block holds one row for each of the 20.

Per Critical 1, record this review file's own digest in that block as well.
