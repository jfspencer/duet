# Engineering Critic, specification review, revision 17

**Document under review.** The frozen copy at
`/private/tmp/claude-501/-Users-james-Developer-duet/cb581a6d-1ce5-4cb9-8b4e-948690e49339/scratchpad/frozen-r17/duet-v1/architecture.md`,
12658 lines.

**Frozen architecture.md md5: `c77ba0484e92109825d8f5927db7b11f`.**

**Scope.** The frozen `architecture.md`, the six records under `adr/`, the six files under `tools/`,
`design-contract.md`, `product-requirements.md`, and `research/`. Cross-checked against the
repository `CLAUDE.md`, `Cargo.toml`, `.cargo/config.toml`, `clippy.toml`, `deny.toml`,
`scripts/dod.sh`, `.github/workflows/ci.yml`, and `crates/duet/src/`. Pinned sources read under
`/Users/james/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`.

**I wrote no repository file. I ran no git command.** One shared cargo target was used and deleted.
Free disk stayed above 110 GB for every run.

**Verdict: NOT READY.**

---

## What I ran

| Run | Result |
|---|---|
| `placement_check.py` over the frozen document | `exit 0`. Baseline green. 36 blocks, 395 declarations, every counter clean. |
| `probe_run.py`, the whole placement probe set | `exit 0`. 302 rows. `PROBES BAD: 0     RECORDED FRAGMENTS BAD: 0`. |
| `conversion_check.py` over the repository tree | `exit 0`. `MEMBERS: 3   FILES: 6   FINDINGS: 0   EXPECTED MEMBERS: 3   EXPECTED FILES: 5`. |
| `probe_conversion.py`, the whole conversion probe set | `exit 0`. 24 shapes. `CONVERSION PROBES BAD: 0`. |
| `roster_compile.sh` over the frozen document | `exit 0`. `ROSTER CLIPPY: clean`, `ROSTER SIZES: 419 measured`, `RECORDED ROWS: 37 checked`, `SIZE BAD: 0`. |
| `probe_roster.sh` over the frozen document | `exit 0`. 12 shapes plus the baseline. `ROSTER PROBES BAD: 0`. |
| My own probes, MINE-C1 to C4, MINE-P1 to P8, MINE-W1 to W2 | Four guard defeats proven. See Critical 6, Critical 7, Warning 6, Warning 8. |
| My own three planted gate defects, at two sites | Three green at the first site, three red at the second. See Concern 4. |

**The three headline repairs hold.** Critical 1 of revision 16 is closed: the roster guard is green
and section 1.9 records the run that produced it. The Critic's own five-crate CG1b shape now exits 2
by name. All twelve conversion probes and all 302 placement probe rows are correct.

---

# Closure check: every row of the revision-16 review

Seventeen Criticals, eighteen Warnings, eleven Concerns. Appendix C.20 carries a row for 26 of the
46. The nine numbered rows with no appendix row are marked with a dagger.

| Row | State | Section | Reason |
|---|---|---|---|
| C16-1 roster red, 1.9 records green | **CLOSED** | 1.9 | `roster_compile.sh` and `probe_roster.sh` both exit 0 over this document, and 1.9 records that run. |
| C16-2 CG1b vacuous | **PARTIAL** | 2.3 | The floor is gone and the Critic's five-crate shape exits 2, but the `target` prune and the two literal trees reopen the class (Critical 6, Warning 6). |
| C16-3 phantom `NothingToHandOff` | **CLOSED** | 15.10 | The arm is declared with a doc comment that states why the loop never produces it. |
| C16-4 C15-7 not closed | **CLOSED** | 5.5 step 5 | Step 5 states three outcomes, and the B109 one-slot `pending_configure` caps the re-send. |
| C16-5 two retry periods | **CLOSED** | 1.6 B95 | B95 reads "3 attempts, one per B108", and no site names B7 as the retry period. |
| C16-6 three framework paths | **CLOSED** | 1.3, 1.9, 10.3 | I resolved all three in the pinned `gpui-component` 0.6.4 source. |
| C16-7 no `AudioBackend` owner | **CLOSED** | 5.5 | `GraphConfigurator::backend` is a `Box<dyn AudioBackend>` with both threads named. |
| C16-8 cap publication | **CLOSED** | 7.3, 15.14 | The cap left `MeterReading` and lives in `MeterReader::caps` with `holds` and the B111 deadline. |
| C16-9 `end_to_end` gate test | **CLOSED** | 13.2, 14 | Two tests, two chunks, two selected-test rows, and the `I4 and H4 before J3` link. |
| C16-10 chunk I4 cannot commit | **CLOSED** | 13.2 | I4's write scope holds `crates/duet-core/Cargo.toml` and `Cargo.lock`. |
| C16-11 C3 creates a forbidden file | **CLOSED** | 13.2 | C1's stub list holds `chain/{topology,state,graph,migrate,configure}.rs`. |
| C16-12 three pairs share a phase | **PARTIAL** | 13.3, 13.4, PG31 | The three links exist and the phases are correct, but PG31 reads only written links, so the shape itself stays invisible (Warning 9). |
| C16-13 refusal table keyed wrong | **CLOSED** | 12.4 | `fault_text.rs` holds four tables and the fourth keys on `ConfigError`. |
| C16-14 phantom `finite_to_f32_saturating` † | **OPEN** | none | The name is in a body, a B.1 row and a closure row, and in no declaration (Critical 2). |
| C16-15 Master has no path cache † | **OPEN** | none | `MasterView` declares no cache and no sibling handle, and 10.5 still gives the linear key to `MixView` alone (Critical 3). |
| C16-16 ADR 0004 clause 12a † | **OPEN** | none | The clause still says no second variant exists, and `GatewayError` still holds four (Critical 4). |
| C16-17 no chunk owns the CI file † | **OPEN** | none | No chunk lists the `dod` job of `ci.yml`, and the live file has no `libpipewire-0.3-dev` (Critical 5). |
| C16-W1 PG26d closes a shape | **PARTIAL** | 1.5 PG26e | PG26e covers 26 of 37 roots; the Critic's own edit at `Transport` is still green (Warning 8). |
| C16-W2 recorded output uncompared | **PARTIAL** | `probe_run.py` | 302 placement rows are compared; PP25 and all twelve CG rows are not (Critical 7). |
| C16-W3 `Path` names two types | **CLOSED** | 1.9 | The external table states the one-name rule and the name map carries `gpui_kit::Path`. |
| C16-W4 two false pinned-source claims | **CLOSED** | 1.9, 5.5 | The `Producer` row carries `PartialEq` and `Eq`, and shutdown step 5 names both refusal causes. |
| C16-W5 `MixView` has no build task | **CLOSED** | 15.16 | `MixView::build` is an `Option<Task<()>>` with the reason in its doc comment. |
| C16-W6 three link reasons cross an edge | **CLOSED** | 13.4 | Each of the three rows names the `duet-core` seam and 1.3 rule 6. |
| C16-W7 13.4 omits the meter seam | **CLOSED** | 13.4 | The `C3 before I3`, `I3 before K3` and `I3 before K4` rows exist. |
| C16-W8 three SHOULD rows | **CLOSED** | 13.2 | The SHOULD table has ten rows with a model, runtime and view column each. |
| C16-W9 I3 names a framework type | **CLOSED** | 13.2 | I1 writes `DuetCore::drain_note_entries` with no framework type, and `I1 before K1` covers it. |
| C16-W10 K5 bench outside scope | **CLOSED** | 13.2 | K5's Writes column holds `crates/duet/benches/`, its manifest and the lock. |
| C16-W11 rung two command 4 | **CLOSED** | 14 | `canonical_determinism` is written by T2 in phase 1 and has its own row. |
| C16-W12 SM4 omits the root manifest | **CLOSED** | 13.0 SM4 | SM4 puts `[workspace.lints]` on the policy list and states the split by table. |
| C16-W13 `C2 before C3` scope rule | **CLOSED** | 13.4 | The table heading states the one intra-line exception, and the row's reason is a compile fact. |
| C16-W14 eleven against twelve rules † | **OPEN** | none | Lines 2234 and 2416 still say eleven, and ADR 0001 still says CG1 to CG8 (Warning 1). |
| C16-W15 `shellcheck` unnamed † | **OPEN** | none | A grep of the document for `shellcheck` returns zero hits (Warning 2). |
| C16-W16 `[lints]` gate claim † | **OPEN** | none | Line 9405 still claims it and `scripts/dod.sh` still reads no member manifest (Warning 3). |
| C16-W17 `SmallVec<[Ticks; 12]>` † | **OPEN** | none | Line 2440 still says 12 and B43 still tests to 13 (Warning 4). |
| C16-W18 research contradictions † | **OPEN** | none | The pipewire unsafe claim, the `tokio-util` pin and the `blake3` C build are all unchanged (Warning 5). |
| N16-1 prose-only closures | **CLOSED** | C.20 | Five of them became machines this revision, and the appendix says which. |
| N16-2 undocumented environment seam | **CLOSED** | 2.3 | Both variables are deleted from the production path and from every probe. |
| N16-3 `Arc<Path<Pixels>>` cost | **CLOSED** | 10.5 | The paint cost is stated at the site. |
| N16-4 pinned-source citation | **CLOSED** | 6.6 | The `window.rs:1765` citation sits beside `:2586`. |
| N16-5 `Cargo.lock` writers | **CLOSED** | 13.3 | The per-phase lock writer counts are stated. |
| N16-6 plan-graph details | **CLOSED** | 13.2 | A4's bench path, the pin-ownership decision and the SHOULD prose are all corrected. |
| N16-7 budget `Used by` cells | **CLOSED** | 14, 1.6 | Rung one carries `--workspace --locked` and the `typos` step; B111 and B112 carry four real use sites. |
| N16-8 negative claims | **CLOSED** | 1.9 | The external table states the framework-name rule. |
| N16-9 three policy gaps † | **OPEN** | none | `clippy.toml`, the advisory list and the duplicate pins still have no owner (Concern 1). |
| N16-10 the 5.12 timeout table † | **PARTIAL** | 5.12 | B108 and B105 joined the table; B24, B25, B26 and B85 did not (Concern 2). |
| N16-11 meter zones † | **OPEN** | none | The -12 dB and -3 dB thresholds are still in no 1.6 row (Concern 3). |

**Totals.** Closed 30. Partial 5. Open 11.

---

# CRITICAL

## CRITICAL 1. Appendix C.20 counts the revision-16 review wrong, and nine findings it omits are open

**OBSERVATION.** Appendix C.20 line 12620 states that the sixteenth review "returned thirteen
Criticals, thirteen Warnings, and eight Concerns". The review at `critic-r16.md` returned
**seventeen Criticals, eighteen Warnings, and eleven Concerns**. Its own verdict line says so, and
its blocking list holds sixteen numbered rows.

C.20 carries a row for C16-1 to C16-13 and for C16-W1 to C16-W13. It carries no row for C16-14,
C16-15, C16-16, C16-17, C16-W14, C16-W15, C16-W16, C16-W17, C16-W18, N16-9, N16-10, or N16-11.

I searched the whole frozen directory for each of those twelve ids. Each one returns zero hits:

```
C16-14: 0   C16-15: 0   C16-16: 0   C16-17: 0
C16-W14: 0  C16-W15: 0  C16-W16: 0  C16-W17: 0  C16-W18: 0
N16-9: 0    N16-10: 0   N16-11: 0
```

I then checked each finding against the frozen text, not against its id. **Every one of the nine
numbered findings is still open**, verbatim. Criticals 2 to 5 below and Warnings 1 to 5 below record
each one with its evidence.

**CLAIM.** The closure appendix is not a record of the last review. It drops four Criticals, five
Warnings, and three Concerns, and it states a count that makes the drop invisible.

**ARGUMENT.** Appendix C is the one place a reader checks whether a finding was answered. Its own
standard, at line 12021, is "one row per finding". A reader who audits revision 17 against C.20
reads thirteen and thirteen, finds thirteen and thirteen rows, and concludes the review is closed.
The nine open findings carry no row, so no reader is sent to a section that does not hold the
mechanism. The appendix does not report a wrong closure; it reports the wrong denominator. That is
the vacuous-green shape, applied to a human process rather than to a script.

The failure is systematic and not a slip. The dropped set is exactly the tail of each family:
Criticals 14 to 17, Warnings 14 to 18, Concerns 9 to 11. A truncated read of the review is the
simplest cause, and a truncated read is the one failure the appendix exists to prevent.

**EVIDENCE.** architecture.md:12618-12622 and the C.20 table. `critic-r16.md` headings at lines 543,
578, 604, 632, 967, 986, 1002, 1022, 1036, 1144, 1157, 1165, and its verdict at line 1216.

**WHAT SHOULD CHANGE.** Add a row for every one of the twelve ids. Correct the count sentence. Then
close each of the nine numbered findings, because each one still stands.

---

## CRITICAL 2. C16-14 is open. `finite_to_f32_saturating` is still a phantom function

**OBSERVATION.** The name occurs three times in the frozen document and in no declaration:

- line 7688, inside the body of `LogicalPx::to_f32`;
- line 11827, a suppression row of Appendix B.1;
- line 12438, a closure row for finding W-6.

Section 2.3 declares twelve items in `duet-time::convert` and this is not one. The counts still
disagree with each other. Line 2214: "Six functions carry a suppression". Line 11813: "Seven
suppressions in `duet-time::convert`". Line 11898: "Twenty sites are predicted in total: seven
conversions, six complexity refusals, four `missing_copy_implementations` expectations, and three
`variant_size_differences` expectations." Against the section 2.3 count of six, the sum is nineteen.

**CLAIM.** A public method body names a function no chunk writes, and Appendix B.1 holds a
suppression row with no site.

**ARGUMENT.** Chunk T1 writes `crates/duet-time/src/convert.rs` from section 2.3. The function is
not there, so `LogicalPx::to_f32` calls nothing and the chunk cannot compile what the document tells
it to write. Appendix B.1 is the register of every accepted suppression. A row for an absent
function is the exact defect the register exists to stop. No guard covers it: PG21 reads the four
`#[expect]` expectation tables, and the run prints `EXPECTATIONS: 4     B.1 ROWS: 4`, so the
suppression-reason table at line 11827 is outside every rule.

**EVIDENCE.** architecture.md:2214, :7688, :11813, :11827, :11898, :12438. Section 2.3, the whole
`convert` declaration block.

**WHAT SHOULD CHANGE.** Declare the function in section 2.3 and add it to the list at line 2214, or
delete the B.1 row and give `LogicalPx::to_f32` a real body. Correct the total at line 11898.

---

## CRITICAL 3. C16-15 is open. Master has a linear timeline and no path cache it can reach

**OBSERVATION.** `design-contract.md:198` gives the Master work area "Linear timeline above, master
chain below." The section 10.5 owner table still reads:

```
| Linear, in Mix and Master | `(SourceHash, RegionId, ZoomScalar)` | `MixView` | ... |
```

`adr/0006-wrapped-timeline.md:55` still reads "Mix and Master use a linear timeline with one
samples-per-pixel scalar, and `MixView` owns their path cache."

`MasterView` at architecture.md:11536 declares `meters`, `stages`, `report`, `target`, `export`,
`job`, `state`, `focus`, and `subscriptions`. It holds no cache, no `Entity<MixView>`, no zoom
field, and no scroll handle. Line 11096 makes `MixView` and `MasterView` two children of `DuetApp`
with no edge between them.

**CLAIM.** Master is told to paint a timeline from a cache no declaration gives it a path to.

**ARGUMENT.** A GPUI child reaches a sibling only through the parent, and `MasterView` declares
nothing that would carry the handle. Chunk K5 writes `src/master/{view,export_dialog,report}.rs`
only. The cache module belongs to K3. No chunk gives `MasterView` a cache, and no chunk gives
`DuetApp` a forwarding path. The engineer who builds K5 must either invent an owner the design does
not name or ship Master with no timeline, which the design contract requires.

**EVIDENCE.** architecture.md:8219-8222 (the 10.5 table), :11096, :11536-11546; `design-contract.md:198`;
`adr/0006-wrapped-timeline.md:55`; architecture.md:9112 (K5 write scope).

**WHAT SHOULD CHANGE.** Give `MasterView` its own cache field and name the chunk that fills it, or
move the shared cache into `DuetApp` and give both views a read handle. Then correct ADR 0006
decision 8 and the 10.5 owner cell.

---

## CRITICAL 4. C16-16 is open. ADR 0004 clause 12a still states the opposite of the type roster

**OBSERVATION.** `adr/0004-backend-and-threading-contract.md:166-167`:

> `configure` returns `ConfigError`, and `EngineError` wraps it with `#[from]`, so a caller still
> sees the cause and no second variant exists.

`ConfigError` declares `PoolExhausted { kind: SlotKind }`, `RingBudgetExceeded { requested_bytes:
u64 }`, `StripBudgetExceeded { requested: u32 }`, and `ParamBudgetExceeded { requested: u32 }`.
`GatewayError` declares the same four names with the same payloads.

Section 12.4 now states the duplication in prose: "the four budget arms share the wording of the
refusal table's own rows". Line 4966 states one mapping in prose: "`ConfigError::PoolExhausted {
kind }` maps to `GatewayError::PoolExhausted { kind }`."

I searched the whole document for a declared conversion. There is no `impl From<ConfigError> for
GatewayError` and no `#[from]` on any `GatewayError` arm that carries a `ConfigError`.

**CLAIM.** The ADR clause that governs the refusal is false, and the mapping it would need exists
only as a sentence.

**ARGUMENT.** An ADR clause is a design decision a reader relies on. This one says one enum owns the
refusal. Two enums own it, with four shared names. A caller that holds a `GatewayError` and a caller
that holds a `ConfigError` must write two matches over one failure set, and the document declares no
conversion to join them. Revision 17 improved the message half, by giving `fault_text.rs` a fourth
table. It did not touch the type half, and the ADR still asserts a property the roster refutes.

**EVIDENCE.** `adr/0004-backend-and-threading-contract.md:163-169`; architecture.md:4966,
:8686-8695, the `GatewayError` and `ConfigError` declarations in section 15.

**WHAT SHOULD CHANGE.** Declare the conversion from `ConfigError` to `GatewayError` and state which
one a gateway caller sees. Then make clause 12a true, or withdraw its last sentence.

---

## CRITICAL 5. C16-17 is open. No chunk owns the `dod` job that rung one describes

**OBSERVATION.** Line 9415 says rung one "runs on `ubuntu-26.04` and on `macos-26`". Line 9416 says
"The Linux job installs `libpipewire-0.3-dev`, `libasound2-dev`, and `pkg-config`".

The live file `/Users/james/Developer/duet/.github/workflows/ci.yml` line 23 reads
`os: [macos-latest, ubuntu-latest]`. Its Linux package list at lines 34 to 42 holds `libasound2-dev`
and `pkg-config` and no `libpipewire-0.3-dev`.

I listed every `.github` entry in the whole chunk table. Chunk M0 owns
`.github/workflows/ci.yml` **(the `plan-lint` job)**. M7 owns `soak.yml`. M8 owns
`audio-smoke.yml`. **No chunk owns the `dod` job.**

**CLAIM.** The first commit that adds the cpal dependency fails the Linux gate, and the file that
would fix it is outside every write scope.

**ARGUMENT.** Section 11.5 states that cpal links both libraries at build time. Chunk M4 adds the
`cpal` pin in phase 4. The Linux runner has no `libpipewire-0.3-dev`, so the build fails, so
`scripts/dod.sh` fails, so the pre-commit hook refuses the commit. The engineer cannot edit
`ci.yml`, because SM4 makes it a policy file and no chunk lists the `dod` job. The plan stalls at
phase 4 with no owner for the fix.

**EVIDENCE.** architecture.md:8462, :8897, :9415-9416; `/Users/james/Developer/duet/.github/workflows/ci.yml:18-42`.

**WHAT SHOULD CHANGE.** Give one chunk the `dod` job of `ci.yml` in its write scope. Name it under
SM4. State the runner labels and the package the plan requires.

---

## CRITICAL 6. The new CG1b member set prunes any directory named `target`, and I reproduced the C16-2 vacuous green

**OBSERVATION.** `tools/conversion_check.py`, `expected_members`, prunes on the bare name:

```python
for directory, children, names in os.walk(base):
    children[:] = [child for child in children if child != "target"]
    if "Cargo.toml" in names:
        found.add(os.path.realpath(directory))
```

The prune runs at every depth and is anchored to nothing. A member directory at `crates/target/` is
therefore never visited and never enters the expected set.

I built the shape. Four members under `crates/` and `tools/`, one of them at `crates/target/` with a
real cast, and one `exclude` line in the root manifest. Production defaults, no environment
variable:

```
=== MINE-C1: crates/target, excluded, holds a real cast ===
MEMBERS: 3   FILES: 3   FINDINGS: 0   EXPECTED MEMBERS: 3   EXPECTED FILES: 3
EXIT=0

=== MINE-C1 control: the same crate renamed to crates/hidden, same exclude line ===
FAIL: crates/hidden holds a Cargo.toml and the scan does not cover it; the guard is fail-closed.
EXIT=2
```

**CLAIM.** The rule that replaced the count floor carries a hole of the same class, and the hole is
the very defect an earlier revision already fixed in the sibling function.

**ARGUMENT.** CG1's own build-directory filter is ANCHORED. `build_directories` compares canonical
paths against the workspace root plus `target` and against each member root plus `target`, and the
docstring says why: revision 11 tested the component name at every depth and lost a source module
named `target` (critic WR-8). Revision 17 wrote the new coverage rule with the unanchored filter the
same guard had already condemned. One `exclude` line plus one directory name gives a clean run over
a crate that holds a cast.

The stated backstop does not cover it. The rule's own limit paragraph names `cargo clippy
--all-targets` in `scripts/dod.sh`. An excluded member is not a workspace member, so `cargo clippy
--workspace` never lints it. For the `exclude` class there is no backstop at all.

`expected_files` carries the identical prune at line 558, so a source module named `target` under a
scanned member's `src/` is outside the expected file set as well. CG1 does scan it today, so that
half is latent rather than live.

**EVIDENCE.** `tools/conversion_check.py`:531-549 (`expected_members`), :551-562 (`expected_files`),
against :195-206 (`build_directories`). architecture.md:2178-2196, the CG1b paragraph. My runs above.

**WHAT SHOULD CHANGE.** Anchor the prune exactly as `build_directories` anchors it: the workspace
root plus `target` and each member root plus `target`, and nothing else. State the anchoring in the
CG1b paragraph of section 2.3, so chunk M0 ports it.

---

## CRITICAL 7. The machine that closes C16-W2 exempts the one row whose false record caused CR-13 and C16-1

**OBSERVATION.** `tools/probe_run.py` line 654:

```python
ROSTER_PROBES = {"PP25"}
```

`text_failures` skips every id in that set. `probe_conversion.py` states in its own docstring that
"no probe here reads the architecture document (section 1.9)", and it reads none. `probe_roster.sh`
compares the exit code alone: it greps one line for display and asserts only `[ "$got" = "$want" ]`.

So the recorded cells of the PP25 row and of all twelve `CP` rows are compared with nothing.

I measured both holes with a run.

```
--- MINE-W1: `ROSTER CRATES:   17` changed to `ROSTER CRATES:   99` in the PP25 row
    exit 0      PROBES BAD: 0     RECORDED FRAGMENTS BAD: 0

--- MINE-W2: a FAIL line in the CG1b row changed to name a crate no run ever printed
    exit 0      PROBES BAD: 0     RECORDED FRAGMENTS BAD: 0
```

**CLAIM.** The check written to end recorded results that no run produced exempts the exact rows
where that defect has occurred, twice.

**ARGUMENT.** CR-13 of the revision-12 review was a false PP25 cell. C16-1 of the revision-16 review
was a false PP25 cell again, in the revision that claimed to have ended the class. The revision-17
answer is a text compare that skips PP25 by name. A reviewer who reads `RECORDED FRAGMENTS BAD: 0`
now believes every recorded line in section 1.9 came from a run. Thirteen of them did not have to.

The exemption has a reason in the source: PP25 needs a compiler. The reason explains why
`probe_run.py` cannot check it. It does not explain why `probe_roster.sh`, which does run the
compiler, checks only an exit code. That script's own header names CR-13 and says "This script is
the run". It runs; it does not compare.

**EVIDENCE.** `tools/probe_run.py`:654, :694-716; `tools/probe_roster.sh`:53-63;
`tools/probe_conversion.py`:3-7. architecture.md:12648, the C16-W2 closure row. My runs above.

**WHAT SHOULD CHANGE.** Make `probe_roster.sh` compare each shape's recorded fragments with its own
output, exactly as `probe_run.py` does. Make `probe_conversion.py` read the section 1.9 `CP` rows
and do the same. Then delete `ROSTER_PROBES` from `probe_run.py`, or state at the PG25 site which
machine owns that row's text.

---

# WARNING

## WARNING 1. C16-W14 is open. The conversion guard has twelve rules and the chunk is told eleven

**OBSERVATION.** Line 1126 lists twelve rules: "CG1 to CG8, with CG1b, CG2b, CG3b, and CG4b". The
section 1.9 probe table holds twelve `CG` rows and `RULE_IDS` in `placement_check.py` holds twelve
`CG` ids. Line 2234 still reads "Its contract has eleven rules". Line 2416 still reads "**Eleven
tests in `tools/xtask` cover it, one per rule (DR5).**" `adr/0001-time-kernel-representation.md:66`
still reads "`cargo xtask check-conversions` proves it under CG1 to CG8".

**CLAIM.** Chunk M0 ports eleven tests for twelve rules, so one rule reaches the production gate
with no test.

**ARGUMENT.** DR5 gives each rule exactly one probe. The count that a chunk brief reads is the
count in section 2.3, which says eleven. The rule most likely to be dropped is CG1b, because it is
the rule the last two revisions added. That is the rule of Critical 6 above.

**EVIDENCE.** architecture.md:1126, :2234, :2416, the section 1.9 probe table;
`adr/0001-time-kernel-representation.md:66`.

**WHAT SHOULD CHANGE.** Correct both counts to twelve. Correct ADR 0001 decision 10.

## WARNING 2. C16-W15 is open. `scripts/dod.sh` runs `shellcheck` and the specification never names it

**OBSERVATION.** `/Users/james/Developer/duet/scripts/dod.sh` lines 88 to 90 run `shellcheck -x -S
warning` over the shell trees when the tool is installed. Line 28 of the same file names the step. A
grep of the frozen `architecture.md` for the word `shellcheck` returns **zero hits**. Line 9413
lists "`bash -n` passes on every shell hook". Line 9421 lists the steps and omits it.

**CLAIM.** Chunk M0 rewrites `scripts/dod.sh` and `scripts/bootstrap.sh` from a specification that
does not know one gate step exists.

**EVIDENCE.** `/Users/james/Developer/duet/scripts/dod.sh:13,28,88-90`; architecture.md:9413,
:9421; the M0 write scope at :8897.

**WHAT SHOULD CHANGE.** Add the step to both lists in section 14.

## WARNING 3. C16-W16 is open. Rung one claims a gate step for `[lints] workspace = true`

**OBSERVATION.** Line 9405 still reads: "Every crate is a workspace member and declares `[lints]
workspace = true` and a `description`." `scripts/dod.sh` reads no member manifest for a `[lints]`
table. The `description` half is covered, because `clippy::cargo` denies `cargo_common_metadata`.
The `[lints]` half is covered by nothing.

**CLAIM.** The plan creates fifteen crates. A crate that omits the line sits outside the whole lint
policy and every gate stays green.

**ARGUMENT.** `CLAUDE.md` states the rule and says such a crate "sits silently outside the lint
policy". Rung one asserts the gate checks it. The gate does not. A rung that reports a state it did
not measure is the same class as a probe that records a line no run produced.

**EVIDENCE.** architecture.md:9405; `/Users/james/Developer/duet/scripts/dod.sh`; `CLAUDE.md`, the
crates section.

**WHAT SHOULD CHANGE.** Add the check to `scripts/dod.sh` and name the chunk that writes it, or
delete the claim from rung one.

## WARNING 4. C16-W17 is open. `SmallVec<[Ticks; 12]>` is one element short of the range B43 tests

**OBSERVATION.** Line 922: "| B43 | divisors 2 to 13, spans 1 to 7680 ticks | The tuplet sum test
range |". Line 2440: `pub fn split_tuplet(span: Ticks, parts: NonZeroU8) -> SmallVec<[Ticks; 12]>;`

**CLAIM.** A thirteen-part tuplet allocates on every call, inside the range the test covers.

**ARGUMENT.** The inline capacity is the only reason to use `SmallVec` here. A capacity one below
the tested maximum guarantees the allocation it was chosen to avoid, at the top of the range.

**EVIDENCE.** architecture.md:922, :2440; `adr/0001-time-kernel-representation.md:61-62`.

**WHAT SHOULD CHANGE.** Raise the inline capacity to 13 and state the B id it comes from.

## WARNING 5. C16-W18 is open. Two research contradictions and one pin contradiction remain

**OBSERVATION.**

1. `research/crate-survey.md:10` still says the `pipewire` stream path has `unsafe` buffer handling
   and records "Excluded: no unsafe." `research/linux-macos-platform.md:19` says the stream path is
   safe. `adr/0004-backend-and-threading-contract.md:301` still says "both sources refute" the
   unsafe claim. One source states it.
2. Line 1714 still pins `tokio-util 0.7.18`. `research/crate-survey.md:122` records `0.7.19`.
3. `research/crate-survey.md:117` still records that `blake3` "on x86 the build needs a C compiler
   unless the `pure` feature is on". Appendix B.3 line 11930 states no feature.
   `adr/0003-history-model.md:74` rejects `git2` because "It links a C build."

**CLAIM.** A closure rests on a "both sources" claim that one source contradicts, and the plan
rejects one crate for a property it then accepts in another.

**EVIDENCE.** As cited.

**WHAT SHOULD CHANGE.** Correct `crate-survey.md` or correct the ADR, and say which is now right.
Correct the `tokio-util` pin. Give `blake3` `features = ["pure"]`, or state why the C build is
accepted here and refused for `git2`.

## WARNING 6. CG1b derives its member trees from two literals, and the stated backstop cannot see an excluded member

**OBSERVATION.** `tools/conversion_check.py` line 528: `MEMBER_TREES = ("crates", "tools")`. The
rule reads two directory names and never the root manifest's own `members` array. The rule's limit
paragraph names `cargo clippy --all-targets` as the backstop.

I built a workspace whose root manifest globs a third tree and excludes one member of it:

```
=== MINE-C3: members = ["crates/*", "tools/*", "apps/*"], exclude = ["apps/duet-cli"], cast inside ===
MEMBERS: 2   FILES: 2   FINDINGS: 0   EXPECTED MEMBERS: 2   EXPECTED FILES: 2
EXIT=0
```

**CLAIM.** One line added to the root manifest takes a whole tree outside the coverage rule, and the
backstop the document names does not cover it.

**ARGUMENT.** `cargo clippy --workspace` operates on workspace members. An `exclude`d package is not
a member, so clippy never lints it. For every member the workspace excludes, the stated backstop is
absent. The limit paragraph therefore claims a safety net that does not exist for the one class the
whole rule was written for.

**EVIDENCE.** `tools/conversion_check.py`:526-529; architecture.md:2192-2196, the limit paragraph.
My run above.

**WHAT SHOULD CHANGE.** Derive the member trees from the root manifest's `members` globs. If the two
literals stay, correct the limit paragraph: name clippy as a backstop for a non-excluded member
only, and say plainly that an excluded member has none.

## WARNING 7. CG1b over-approximates the root manifest globs, so a nested fixture crate is a false red

**OBSERVATION.** The CG1b paragraph says "Every directory under `crates/` and under `tools/` that
holds a `Cargo.toml`, at any depth, is a member this workspace holds, **because the root manifest
globs both trees**." The root manifest globs `crates/*` and `tools/*`, which is exactly one level.

```
=== MINE-C2: a nested fixture crate the crates/* glob does not make a member ===
FAIL: crates/duet-a/tests/fixture holds a Cargo.toml and the scan does not cover it; the guard is fail-closed.
EXIT=2
```

**CLAIM.** The stated justification is false, and the over-approximation fires on a normal Rust
layout.

**ARGUMENT.** A `trybuild` fixture crate, a compile-fail case, and a nested example crate each sit
under a member and each carry a `Cargo.toml` the workspace excludes. The guard reports a coverage
failure over every one of them, with no way to answer it except to move the fixture. The rule is
fail-closed in the safe direction, so this is a Warning and not a Critical. It is still a guard that
will stop a commit for a shape no rule forbids.

**EVIDENCE.** `tools/conversion_check.py`:531-549; `/Users/james/Developer/duet/Cargo.toml:12`;
architecture.md:2185-2189. My run above.

**WHAT SHOULD CHANGE.** Derive the expected set from the manifest globs, or restrict the walk to one
level and say so. Correct the sentence that claims the globs reach any depth.

## WARNING 8. PG26e leaves eleven roots, and the revision-16 Critic's own disarm is still green

**OBSERVATION.** PG26e derives the audio-owned closure from the declaration bodies below
`GraphState`. I confirmed the derivation is correct and the stated residual is exact. The guard
reaches 26 of the 37 roots. The eleven it does not reach are, by my own computation from the guard's
own functions:

```
Cycle CycleOutcome EngineFault MeterSnapshot MidiRecord NoteEntry
ParamSnapshot RecordState Transport TransportSnapshot TransportState
```

That list matches the document's list at line 791 exactly.

I then planted the revision-16 Critic's four-line coordinated edit at `Transport`, together with a
`Vec<u8>` on the swapped-out type:

```
--- MINE-P2 (block swap + marker moved + Vec<u8> on Transport): exit 0
    AUDIO OWNED: 37   HEAP IN AUDIO: 0   GROW IN AUDIO: 0   ROOT BAD: 0   CLOSURE BAD: 0
--- MINE-P2b (the same Vec<u8>, no block or marker edit): exit 1
    GROW: duet-engine::Transport holds Vec<u8> through probe_heap
```

**CLAIM.** Guard hole 1 is narrowed and not closed. A four-line edit still disarms all three TH1
rules on eleven of the thirty-seven roots, `Transport` among them.

**ARGUMENT.** The document is honest about this. The PG26d site states its limit, the PG26e site
names the eleven roots, and the PP26e cell records that the same edit at `Transport` is green. I
raise a Warning and not a Critical for that reason. It is still a Warning and not a Concern: the
remaining eleven include the live transport, the meter publication, and the cycle types, which are
the values on the audio path whose allocation rule matters most. `Transport` is the exact type the
revision-16 probe used, so the documented probe and the open hole sit on the same type.

**EVIDENCE.** `tools/placement_check.py`:686-745; architecture.md:774-794, :1260. My runs above, and
my recomputation of the eleven from the guard's own `audio_reachable`.

**WHAT SHOULD CHANGE.** Give the eleven a second derived source. The two publications and the two
MIDI record types reach the audio thread as a published value or a function argument, and both
shapes are declared in this document. A rule that walks the `triple_buffer` input types and the
`GraphRunner` argument types would cover most of the eleven.

## WARNING 9. PG31 reads only the links the document writes, and it exempts two whole lines from SM6

**OBSERVATION.** `link_phase_audit` iterates `link_rows(source)`. Its limit paragraph says so: "It
reads the links the document WRITES. A crate edge that binds and that section 13.4 does not carry is
outside it."

`line_phase_audit` carries a second, undocumented exemption:

```python
if chunk.startswith("M") or chunk.startswith("T"):
    continue
```

SM6 at architecture.md:8835 states the rule with no exemption: "two chunks of one line never run in
the same phase."

I probed both halves. Two `T` chunks put in one phase were caught, but by the link rule and not by
SM6. Two `C` chunks in one phase were caught by SM6, as the control.

```
--- MINE-P5 (T2 and T3 in phase 2): exit 1   LINK: T2 before T3: T2 is in phase 2 and T3 is in phase 2
--- MINE-P5b (C1 and C2 in phase 5): exit 1   LINK: C1 and C2: one line, both in phase 5 (SM6)
```

**CLAIM.** The rule named as the closure of C16-12 cannot detect the shape C16-12 described, and the
SM6 half of it does not implement the rule it cites.

**ARGUMENT.** C16-12 was three chunk pairs sharing a phase across a crate edge **with no link row
for any of them**. PG31 reads rows. A new pair in that state adds no row, so PG31 stays green. The
document states the limit, which makes this a Warning. The `M` and `T` exemption is worse, because
it is in the machine and in no line of the document. SM6's own reason, one member-manifest writer
per phase, applies to the `T` line as hard as to any other. The exemption is covered today only
because the `T` line happens to carry dense forward links.

**EVIDENCE.** `tools/placement_check.py`:2702-2732, :2734-2751; architecture.md:8835-8836,
:796-810, :1261. My runs above.

**WHAT SHOULD CHANGE.** Delete the `M` and `T` skip, or state the exemption and its reason under
SM6. For the link half, derive the binding edges from the declarations the chunks write, or state
the residual next to the C16-12 closure row rather than only at the rule site.

## WARNING 10. The SM6 paragraph is stale against the sixteen-phase table

**OBSERVATION.** architecture.md:8849-8851:

> **The plan is longer and narrower than revision 5's.** Thirteen phases replace ten, and the widest
> parallel count falls from eleven to eight. The last three phases hold one chunk each ...

The phase table holds sixteen rows, phase 0 to phase 15. The widths are 1, 2, 4, 5, 7, 6, 4, 5, 4,
4, 3, 2, 1, 1, 1, 0. The widest is **seven**, at phase 4. The last three phases hold one, one, and
zero.

**CLAIM.** Three numbers in the paragraph that justifies SM6 are wrong, in the revision that added
the sixteenth phase.

**ARGUMENT.** The paragraph is the cost argument for SM6. A reader who weighs the rule weighs it
against "thirteen phases" and "eight wide". The real plan is sixteen phases and seven wide, so the
cost is understated and the parallelism is overstated. Line 9317 states the tail correctly, so the
document disagrees with itself two hundred lines apart. PG27 checks the row count of the phase table
and no rule reads this prose.

**EVIDENCE.** architecture.md:8849-8851 against the `phase-table` block and :9313, :9317.

**WHAT SHOULD CHANGE.** Rewrite the three numbers from the table. State the phase count in one place
and cite it elsewhere.

---

# CONCERN

## CONCERN 1. N16-9 is open. Three policy gaps still have no owner

`clippy.toml` is on no chunk write scope and on no SM4 policy list. `pedantic` denies
`clippy::doc_markdown`, and the plan's vocabulary adds `PipeWire`, `CoreAudio`, `CoreMIDI`,
`MusicXML`, `SMuFL`, `Bravura`, `Wayland`, and `RF64`. The document names no `doc-valid-idents` rule
and no backticking rule. The `deny.toml` advisory ignore list is scoped to `gpui-kit` transitives,
the plan adds about thirty crates, and no chunk owns a new RUSTSEC ignore. Appendix B.5 still says
the `clap` and `tracing` pins are already in the root manifest while chunk M7 is told to pin them.

## CONCERN 2. N16-10 is partly open. Four timeout ids sit outside the table ADR 0004 names

`adr/0004:219` says section 5.12 holds the table of every cross-boundary wait. The 5.12 table now
names B15 to B23, B27, B95, B105, B108, and B110. The Appendix B.5 mechanism table adds B24, B25,
B26, and B85. Those four have a mechanism and no row in the one table the ADR names. B110 bounds an
allocation and not a wait, and it is in the timeout table.

## CONCERN 3. N16-11 is open. The design contract's three meter zones have no B row

`design-contract.md:758-759` gives the level meter three colour zones at -12 dB and -3 dB. B106
carries the scale and no zone threshold. `MeterLayer` and `LevelMeter` must paint the zones and
neither threshold is a row of section 1.6, which line 829 requires.

## CONCERN 4. The roster compiles 22 of 50 impl blocks, and a body beside a signature escapes PG25

`full_impls` in `roster_compile.sh` drops a whole `impl` block when any one item has no body. I
measured the frozen document: 50 impl blocks parse, 22 compile, 28 are dropped. My first three
planted gate defects went into `impl Transport`, which carries a bodiless signature, and **all three
were green**:

```
GATE-MINE-1-index EXIT=0  ROSTER CLIPPY:   clean
GATE-MINE-2-print EXIT=0  ROSTER CLIPPY:   clean
GATE-MINE-3-missingdocs EXIT=0  ROSTER CLIPPY:   clean
```

The same three defects at `impl Debug for ChainSetHandle`, which is fully bodied, were all red:

```
GATE-MINE-index EXIT=1  error: binding to `_` prefixed variable with no side-effect
GATE-MINE-panic EXIT=1  error: used `panic!()` or assertion in a function that returns `Result`
GATE-MINE-print EXIT=1  error: use of `println!` in `Debug` impl
```

No impl block in the frozen document is mixed today, so nothing escapes now. The day an author
writes one body beside one signature, that body leaves PG25 and no counter moves. The `missing_docs`
plant could never be red, because the profile relaxes that lint, and the document states the
relaxation at line 689. That plant was my error and not a guard defect.

## CONCERN 5. `RECORDED FRAGMENTS BAD` prints no denominator

`probe_run.py` prints `RECORDED FRAGMENTS BAD: 0` and never prints how many fragments it compared. I
measured it from the library: the section 1.9 cells hold 151 backtick fragments and the harness
compares **59**. The rest fail the `OUTPUT_HEAD` test and count as prose. If that regular expression
ever stops matching, the line reads the same and the run stays green. Print the compared count and
fail below a floor derived from the row count.

## CONCERN 6. The fragment oracle pools every shape's output, so a misattributed line passes

`text_failures` builds one pool from every shape's output and asks only whether the fragment appears
somewhere in it. The source states the limit. The effect is that a recorded line moved from the row
that produces it to a row that does not is green. The check proves that some run printed the string,
never that this probe's run printed it.

## CONCERN 7. A chunk moved into phase 15 passes every rule

Phase 15 is documented as the human acceptance gate that writes no file. I moved `K6` into it and
emptied phase 14. The run is green:

```
--- MINE-P8-chunk-moved-to-last-phase: exit 0     PLAN LINKS: 56     LINK BAD: 0
```

No rule states that the last phase holds no chunk, and the prose at line 9313 that says so is read
by nothing.

## CONCERN 8. A symbolic link to a source directory is outside CG1 and CG1b

`os.walk` does not follow a symbolic link to a directory, in `source_files` and in `expected_files`
alike. A `pub mod linked;` whose directory is a link outside the member is compiled by rustc and
scanned by neither half of the guard:

```
=== MINE-C4: crates/duet-a/src/linked -> ../../../outside/linked, with a cast inside ===
MEMBERS: 2   FILES: 2   FINDINGS: 0   EXPECTED MEMBERS: 2   EXPECTED FILES: 2
EXIT=0
```

CG7 was hardened against exactly this attack surface in revision 11, so the author already treats a
symbolic link in `src/` as hostile input. `cargo clippy` is a real backstop here, because the member
is not excluded, so this is a Concern. State the limit beside the `#[path]` limit CG1 already names.

## CONCERN 9. The suppression-reason table of Appendix B.1 is read by no rule

PG21 audits four expectation tables, and the run prints `EXPECTATIONS: 4     B.1 ROWS: 4`. The
suppression-reason table at line 11813 holds seven rows and no rule reads it against a declaration.
That is why Critical 2 above survived seventeen revisions. One rule that holds the seven rows and
the section 2.3 declaration list to one set would end the class.

---

# What is genuinely closed, and what passed

I record this so no revision re-opens settled work.

**Closed and closed well.** C16-1: the roster and the roster probe set are both green and section
1.9 records this revision's run. C16-3: `NothingToHandOff` is a declared arm with its own doc
comment. C16-4: section 5.5 step 5 states three outcomes and the B109 one-slot `pending_configure`
caps the re-send. C16-5: B95 reads "3 attempts, one per B108" and no site names B7 as the retry
period. C16-6: I resolved all three framework paths in the pinned sources.
`gpui_component::Theme` exists through `pub use theme::*` at `lib.rs:119`,
`VirtualList` through `lib.rs:122`, and `DropdownMenu` is a trait at `menu/dropdown_menu.rs:12` that
`menu/mod.rs:11` re-exports. C16-7: `GraphConfigurator::backend` is a `Box<dyn AudioBackend>` with
its threads named. C16-8: the cap left `MeterReading` and lives in `MeterReader::caps` with
`holds` and the B111 deadline. C16-9: `end_to_end` and `export_end_to_end` are two tests, two
chunks, two selected-test rows and one `I4 and H4 before J3` link. C16-10, C16-11, C16-13, C16-W3,
C16-W5, C16-W6, C16-W7, C16-W8, C16-W9, C16-W10, C16-W11, C16-W12, C16-W13: each one verified at the
section the row names.

**Guard work that holds.** The placement guard is green and all 302 placement probe rows are red on
plant. All 24 conversion probe shapes are correct. All 12 roster shapes are red on plant, including
the three new GATE17 shapes at declarations this revision wrote. The block register is
double-sourced: I lowered the document's own `rows>=37` marker to 36 and the run exits 2 with
`the marker states rows>=36 and the register states 37`. I tried to weaken a block's membership kind
and the existing rows refuse the weaker kind, so that attack is self-limiting. PG26e's derivation is
correct: I recomputed the reachable set from the guard's own function and the eleven roots outside it
match the document's list exactly.

**Cross-document checks that pass.** Every licence the plan needs is in `deny.toml` except
`Unlicense`, and Appendix B.4 assigns that edit to M0, which owns `deny.toml`. The B95 and B108
arithmetic agrees across 1.6, 5.12, Appendix A and Appendix B.5. The sixteen-phase table, the 56
link rows and the chunk table are consistent under PG31.

---

# Reactive assessment

| Property | Verdict | Why |
|---|---|---|
| Responsive | **PARTIAL** | The pool-handoff retry now carries one period and a three-attempt cap (C16-5 closed). `MixView` owns its build task (C16-W5 closed). `MasterView` must paint a linear timeline from a cache no declaration gives it (Critical 3), so one of the four work areas has no reachable render path. |
| Resilient | **PARTIAL** | `NothingToHandOff` is declared, `fault_text.rs` has a fourth table for `ConfigError`, and the engine owns its backend. `GatewayError` still duplicates four `ConfigError` arms with no declared conversion, and ADR 0004 clause 12a asserts the opposite (Critical 4). A `# Errors` contract still names a function the plan never writes (Critical 2). |
| Elastic | **PASS** | `pending_configure` holds one request, B109 bounds the channel, B105 bounds the handoff ring, and every producer I traced has a bound. No unbounded collection or channel appears in the roster. |
| Message Driven | **PARTIAL** | The engine seam is messages throughout, and the three 13.4 rows that described a direct call now name the `duet-core` seam and 1.3 rule 6. `MasterView` is still told to read a sibling entity's cache, which is shared state across a boundary with no channel (Critical 3). |

---

# Verdict

**NOT READY.** Revision 17 did real engineering. Every guard and every harness is green, the roster
compiles clean, and thirteen of the seventeen Criticals and thirteen of the eighteen Warnings of the
last review are genuinely closed at the sections the appendix names. The direction of travel the
appendix claims is real: CG1b became a derived set, C16-12 became PG31, and C16-W1 became PG26e.

The single biggest risk is **Critical 1**. The closure appendix records the last review as thirteen
Criticals and thirteen Warnings. It returned seventeen and eighteen. Nine numbered findings have no
row, no citation, and no fix, and I verified all nine are open. That is not one missed finding. It
is a broken audit trail on the one document a reader uses to decide whether a review was answered.
Every future review inherits the error, because C.20 is now the record.

Close behind it is **Critical 6**. I reproduced the C16-2 vacuous green against the rule written to
end it, at production defaults, with one `exclude` line and one directory name. The new rule carries
the unanchored path filter that the same guard had already fixed in its sibling function for the
same reason. And **Critical 7**: the machine written to end recorded results that no run produced
exempts the one row where that defect has now occurred twice.

The weakest Reactive property is **Resilient**, for the third revision running. Two failure surfaces
still reach a caller that cannot handle them: two enums that duplicate four refusal variants with no
declared conversion, under an ADR clause that says the duplication does not exist, and a `# Errors`
contract that names a function no chunk writes.

---

# Blocking list

Every Critical blocks plan authoring. Every Warning must close before FINISH. Fix in this order.

1. **Critical 1** — Add a C.20 row for C16-14, C16-15, C16-16, C16-17, C16-W14 to C16-W18, and
   N16-9 to N16-11. Correct the count sentence at line 12620.
2. **Critical 6** — Anchor the `target` prune in `expected_members` and `expected_files` the way
   `build_directories` anchors it. State the anchoring in section 2.3.
3. **Critical 7** — Make `probe_roster.sh` and `probe_conversion.py` compare each row's recorded
   fragments with the run that produced them.
4. **Critical 2** — Declare `finite_to_f32_saturating` or delete its B.1 row and its call site.
   Correct the six, seven and twenty counts.
5. **Critical 4** — Declare the `ConfigError` to `GatewayError` conversion and make ADR 0004
   clause 12a true.
6. **Critical 3** — Give `MasterView` a reachable path cache and correct ADR 0006 decision 8.
7. **Critical 5** — Give one chunk the `dod` job of `ci.yml` and name it under SM4.
8. **Warnings 1 to 5** — The five revision-16 Warnings the appendix dropped.
9. **Warnings 6, 7** — The two remaining CG1b defects.
10. **Warnings 8, 9, 10** — The PG26e residual, the PG31 exemptions, and the stale SM6 paragraph.

**Re-review condition.** Do not send revision 18 for review until three commands pass. First, a
workspace with a crate at `crates/target/` and one `exclude` line exits 2. Second, a false number
planted in the PP25 recorded cell exits non-zero. Third, a grep of the document for each of the
twelve dropped finding ids returns a hit. Each one is one command.
