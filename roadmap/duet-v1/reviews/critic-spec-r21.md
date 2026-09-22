# Engineering Critic, external specification review of revision 21

**Document under review.** The frozen copy at
`/private/tmp/claude-501/-Users-james-Developer-duet/cb581a6d-1ce5-4cb9-8b4e-948690e49339/scratchpad/frozen-r21/duet-v1/`.
`architecture.md` md5 at the START of my run: `1fd9efa3b89837c80a904a48ce1e68c7`.
`architecture.md` md5 at the END of my run: see the last line of this report.

**I wrote no repository file. I ran no git command.** Every plant went into a throwaway copy under
my own scratch directory. One shared cargo target served every roster run.

**Verdict: NOT READY FOR PLAN AUTHORING.** The blocking list is at the end.

---

## 1. What I ran: `run_all_gates.py` on the frozen copy

I ran the section 1.9 command, verbatim, with one addition I state here. I exported
`ROSTER_TARGET_DIR` to one shared path, because the brief's disk rule requires one shared cargo
target. Section 1.9 grants that ownership to a caller. Without it `run_all_gates.py` gives
`roster_compile.sh` and `probe_roster.sh` a scratch directory each, so the whole `gpui` tree builds
twice.

```
cd roadmap/duet-v1
PYTHONDONTWRITEBYTECODE=1 ROSTER_TARGET_DIR=<shared> python3 tools/run_all_gates.py \
    architecture.md <scratch outside the repository> /Users/james/Developer/duet
```

**Result: exit 0. `GATES RUN: 13     GATES BAD: 0     MODE: FULL`.**

The verbatim 1121-line transcript is at
`/private/tmp/claude-501/-Users-james-Developer-duet/cb581a6d-1ce5-4cb9-8b4e-948690e49339/scratchpad/r21-gates-full.txt`.
Every gate header and every counter line of that transcript follows.

```
    === placement_check        exit 0  OK      1.9s
    DOCUMENT:        /private/tmp/claude-501/-Users-james-Developer-duet/cb581a6d-1ce5-4cb9-8b4e-948690e49339/scratchpad/frozen-r21/duet-v1/architecture.md
    BLOCKS:          47     BLOCK BAD: 0
    FRAMEWORK NAMES: 56    NAME MAP: 20
    DROP LIST:       80     PRIMITIVES: 17     PRIMITIVE SIZES: 16
    CANDIDATE TYPES: 419   DECLARED: 405   PLACEHOLDERS: 0
    TABLE NAMES:     405     UNDECLARED: 0
    EXTERNAL ROWS:   49     EXTERNAL MISSING: 0
    PLACED:          419
    UNPLACED:        0     DUPLICATED:   0
    MISCLAIMED:      0     FRAMEWORK MISUSE: 0
    COPY MISSING:    0     COPY IMPOSSIBLE:  0
    COPY UNDECIDED:  0     UNKNOWN: 54     JUSTIFIED: 15     UNJUSTIFIED: 0
    DERIVE CLOSURE:  0     DERIVE UNDECIDED: 0
    REACH BAD:       0     CONST REACH BAD: 0
    LIMIT ROWS:      4     LIMIT BAD: 0
    PROBE ROWS:      60     PROBE BAD: 0
    RULE IDS:        59     LOCK PHASES: 16     LOCK BAD: 0     PIN ROWS: 24     PIN BAD: 0
    MEMBER BLOCKS:   47     MEMBER ROWS: 1123     MEMBER BAD: 0
    BUDGET ROWS:     132     USED BY BAD: 0
    EXPECTATIONS:    4     B.1 ROWS: 4     B.1 BAD: 0
    VARIANT SITES:   3     B.1 VARIANT ROWS: 3
    REASON SIZES:    8     REASON BAD: 0
    VR1 ROWS:        70     VR1 BAD: 0
    EQ MISSING:     0     EQ UNDECIDED: 0
    SIZES DECIDED:  356     SIZE UNDECIDED: 39     VARIANT SPREAD BAD: 0     VARIANT EXPECTED: 1
    AUDIO OWNED:     40     AUDIO EXEMPT: 1     AUDIO DEFERRED: 49     HEAP IN AUDIO: 0     DROP IMPLS: 3
    GROW IN AUDIO:   0     LOCK IN AUDIO: 0     AUDIO READ ONLY: 48     ROOT BAD: 0
    AUDIO REACHABLE: 79     REACHABLE LEAVES: 44     CLOSURE BAD: 0
    EDGES PARSED:    59    EDGE CLAIMS BAD: 0
    DEP ROWS:        16    DEP PROVEN: 24    DEP MISSING: 0
    SNAPSHOTS:       2     SNAPSHOT BAD: 0
    REGISTER BAD:    0     FLOOR SLACK: 0     VALUE ROWS: 9     VALUE SKIPPED: 123     VALUE BAD: 0
    TESTS SELECTED:  8     TEST ROWS BAD: 0
    PLAN LINKS:      56     LINK BAD: 0
    PHASE PAIRS:     24     PAIR EXEMPT: 24     PAIR BAD: 0     TAIL BAD: 0
    B.1 SITES:       7     2.3 LISTED: 7     SUPPRESSION BAD: 0     ASSERTED ROOTS: 11     ASSERTED BAD: 0
    === probe_run              exit 0  OK    391.1s
    BASE                   exit 0  the unmodified document
    PP1                    exit 2  OK   no document argument
    PP2                    exit 2  OK   a path that does not exist
    PP3                    exit 2  OK   every ```rust fence renamed
    PP4a                   exit 1  OK   a declared struct ProbeUnplaced with no section 1.5 row
    PP4-droplist           exit 2  OK   the candidate drop list deleted under a kept heading
    PP4-primtraits         exit 2  OK   the primitive trait-set block deleted
    PP4-primsizes          exit 2  OK   the primitive size block deleted
    PP4b                   exit 1  OK   the VoiceId declaration removed from section 15.3
    PP5                    exit 1  OK   a second declaration of Knot
    PP6                    exit 1  OK   Pixels added to the duet-command row of section 1.5
    PP7                    exit 1  OK   sidebar_width retyped to gpui_kit::Px in ModeView
    PP8                    exit 1  OK   SlotSpec::kind retyped to the arm name Reverb
    PP9                    exit 1  OK   the Copy derive removed from ExportSpec
    PP10                   exit 1  OK   port: MidiPortId in MidiRecord and reset: AtomicU32 in MeterState
    PP10b                  exit 1  OK   the DeviceKey declaration removed
    PP11                   exit 1  OK   a sentence that `duet-time` writes `duet-project`
    PP12                   exit 1  OK   `serde` removed from the duet-score row of section 1.2
    PP13                   exit 1  OK   the Copy derive removed from MeterSnapshot
    PP14                   exit 1  OK   probe: AnyView added to Toolbar, which derives Copy in an application row
    PP15                   exit 1  OK   8.3 removed from the Declared in cell of the duet-command row
    PP16                   exit 1  OK   the `soak` row of the selected-test table renamed to `soakx`
    PP17                   exit 1  OK   the SharedString name removed from the justified-unknown table
    PP18                   exit 1  OK   the BTreeMap name removed from its external-verdict row
    PP19                   exit 1  OK   the Default derive removed from Finite in section 2.6a
    PP20                   exit 1  OK   probe_strip: StripId added to duet-engrave::FontMetrics
    PP20b                  exit 1  OK   MAX_STRIPS moved under a `// duet-project` comment
    PP21a                  exit 1  OK   the duet-session::track::Source expectation site renamed in its B.1 row
    PP21b                  exit 1  OK   the duet-score::mark::MarkKind site renamed in the variant_size_differences table
    PP21c                  exit 1  OK   the SlotState expect reason states a size as a literal
    PP21d                  exit 1  OK   a doc comment states a size as a literal
    PP21e                  exit 1  OK   a missing_copy_implementations expectation on a Drop type
    PP22                   exit 1  OK   the NoteId name changed to NoteIdx in its section 3.5 VR1 row
    PP23                   exit 1  OK   the Eq derive removed from Revision in section 15.3
    PP24                   exit 1  OK   the variant_size_differences expectation removed from SlotState in section 5.5
    PP26-a                 exit 1  OK   Equalizer(Box<EqualizerState>) in place of the inline arm, the revision-11 shape
    PP26-b                 exit 1  OK   probe: Arc<Generation> added to ChainState, a heap handle that never grows
    PP26-c                 exit 1  OK   the basedrop::Owned wrapper removed from DiskWriter.samples, which leaves a bare rtrb end
    PP26-d                 exit 1  OK   the basedrop::Owned wrapper removed from DiskReader.requests
    PP26-e                 exit 2  OK   the one exemption row removed, which leaves the Arc on GraphState.resets bare
    PP26b-owned            exit 1  OK   probe_grow: Owned<Vec<u8>> added to DiskReader, which is the Critic shape MINE-A1
    PP26b-shared           exit 1  OK   probe_shared: Shared<Vec<u8>> added to ChainState
    PP26b-fixed            exit 0  OK   CONTROL: probe_fixed: Owned<Box<[f32]>> added to ChainState, which is legal and stays green
    PP26c-mutex            exit 1  OK   probe_lock: Mutex<u32> added to ChainState, which is the Critic shape MINE-A4
    PP26c-rwlock           exit 1  OK   probe_rw: RwLock<u32> added to ChainState, which is the Critic shape MINE-A6
    PP26c-parking          exit 1  OK   probe_pl: parking_lot::Mutex<u32> added to ChainState, which proves the last path segment is the name
    PP26-f                 exit 1  OK   a bare Box<[u8]> in place of the Arc on the one exempt field, which a path-only row let through
    PP26b-below            exit 1  OK   probe: Vec<u8> inside ResetGenerations, one step BELOW the one exempt field
    PP26c-below            exit 1  OK   probe: Mutex<u32> inside ResetGenerations, one step BELOW the one exempt field
    PP26d-swap             exit 1  OK   Transport swapped for ChannelConfig in the audio-owned block, a clean one-for-one swap
    PP26d-marker           exit 1  OK   the audio-owned marker removed from the Transport declaration
    PP26e-swap             exit 1  OK   ChainState swapped for ChannelConfig in the block AND its marker moved, which is the Critic's four-line coordinated edit at a reachable root
    PP26e-leaf             exit 1  OK   the BufferPool row removed from the reachable-leaf block
    PP31-link              exit 1  OK   J1 removed from phase 8, so the J1 before K1 link names a chunk no phase holds
    PP31-line              exit 1  OK   K2 moved into phase 11 beside K3, which SM6 forbids
    PP26f-swap             exit 1  OK   the Critic's four-line coordinated edit at Transport, which PG26d and PG26e both pass
    PP26f-row              exit 1  OK   one audio-asserted row deleted
    PP31b-pair             exit 1  OK   the E1 D2 row deleted from the phase-pair-exempt block
    PP33-decl              exit 1  OK   the finite_to_f32_saturating DECLARATION deleted, with the list and the B.1 row kept
    PP33-word              exit 1  OK   the count word of the section 2.3 list changed from Seven to Six
    PP37-neighbour         exit 1  OK   the B120 row changed from 9 slots to 8 slots, which is the value of a constant beside it
    PP37                   exit 1  OK   the B113 row changed from 13 elements to 12 elements, with the declaration kept
    PP33-b1count           exit 1  OK   the Appendix B.1 complexity count word changed from Six to Nine
    PP34-count             exit 1  OK   the SM6 cost bullet restored to the revision-17 numbers
    PP31b-reason           exit 1  OK   one phase-pair exemption reason that names neither chunk of its own pair
    PP35-sequence          exit 1  OK   the per-phase Cargo.lock writer sequence moved by one at phase four
    PP35-writer            exit 1  OK   one chunk's Writes cell emptied of Cargo.lock, which moves the derived count
    PP36-owner             exit 1  OK   one Appendix B.5 pin owner returned to the revision-19 chunk
    PP36-pin               exit 1  OK   one manifest pin list emptied of the name an appendix row owns
    PP33-row               exit 1  OK   the finite_to_f32_saturating row deleted from the b1-convert block
    PP33-list              exit 1  OK   ticks_to_f64 removed from the section 2.3 list
    PP34-tail              exit 1  OK   K6 moved from phase 14 into phase 15
    PP30a                  exit 1  OK   10.2 removed from the Used by cell of B99
    PP30b                  exit 1  OK   9.9 added to the Used by cell of B99
    PP4-drop               exit 1  OK   a declared struct ProbeUnplaced whose name is then added to the candidate drop list
    PP28                   exit 1  OK   the MAX_PARAMS owner changed to duet-engine in the shared-limit block
    PP29a                  exit 1  OK   the PG13 rule id renamed to PG43 in the probe table
    PP29b                  exit 1  OK   the PP11 cell of the PG11 row renamed to PP41
    PP27:crate-table:deleted                     exit 2  OK   FAIL: the `crate-table` block of this document: it holds 0 rows and the stated minimum is 16; th
    PP27:crate-table:emptied                     exit 2  OK   FAIL: the `crate-table` block of this document: it holds 15 rows and the stated minimum is 16; t
    PP27:crate-table:unmarked                    exit 2  OK   FAIL: the `crate-table` block of this document: the marker line appears 0 times under the headin
    PP27:crate-table:renamed                     exit 2  OK   FAIL: the `crate-table` block of this document: the heading `#### The crate dependency table` ap
    PP27:crate-table:stray                       exit 1  OK     MEMBER:     crate-table: `duet-probe` | A probe | Yes | `serde`: `duet-probe` is no crate of t
    PP27b:crate-table:floor                      exit 1  OK     FLOOR:      crate-table: the block holds 17 rows and the stated floor is 16; a floor below the
    PP27:name-map:deleted                        exit 2  OK   FAIL: the `name-map` block of this document: it holds 0 rows and the stated minimum is 20; the g
    PP27:name-map:emptied                        exit 2  OK   FAIL: the `name-map` block of this document: it holds 19 rows and the stated minimum is 20; the 
    PP27:name-map:unmarked                       exit 2  OK   FAIL: the `name-map` block of this document: the marker line appears 0 times under the heading; 
    PP27:name-map:renamed                        exit 2  OK   FAIL: the `name-map` block of this document: the heading `#### The third-party name map` appears
    PP27:name-map:stray                          exit 2  OK   FAIL: the name map holds a row the guard cannot resolve: ProbeName        probe-crate; the guard
    PP27b:name-map:floor                         exit 1  OK     FLOOR:      name-map: the block holds 21 rows and the stated floor is 20; a floor below the ro
    PP27:edge-list:deleted                       exit 2  OK   FAIL: the `edge-list` block of this document: it holds 0 rows and the stated minimum is 18; the 
    PP27:edge-list:emptied                       exit 2  OK   FAIL: the `edge-list` block of this document: it holds 17 rows and the stated minimum is 18; the
    PP27:edge-list:unmarked                      exit 2  OK   FAIL: the `edge-list` block of this document: the marker line appears 0 times under the heading;
    PP27:edge-list:renamed                       exit 2  OK   FAIL: the `edge-list` block of this document: the heading `#### The internal edge list` appears 
    PP27:edge-list:stray                         exit 1  OK     MEMBER:     edge-list: duet-probe           -> duet-time: `duet-probe` is no crate of the othe
    PP27b:edge-list:floor                        exit 1  OK     FLOOR:      edge-list: the block holds 19 rows and the stated floor is 18; a floor below the r
    PP27:framework-types:deleted                 exit 2  OK   FAIL: the `framework-types` block of this document: it holds 0 rows and the stated minimum is 7;
    PP27:framework-types:emptied                 exit 2  OK   FAIL: the `framework-types` block of this document: it holds 6 rows and the stated minimum is 7;
    PP27:framework-types:unmarked                exit 2  OK   FAIL: the `framework-types` block of this document: the marker line appears 0 times under the he
    PP27:framework-types:renamed                 exit 2  OK   FAIL: the `framework-types` block of this document: the heading `#### Framework types` appears 0
    PP27:framework-types:stray                   exit 1  OK     MEMBER:     framework-types: Ticks: `Ticks` is a type this document declares
    PP27b:framework-types:floor                  exit 1  OK     FLOOR:      framework-types: the block holds 8 rows and the stated floor is 7; a floor below t
    PP27:ownership-table:deleted                 exit 2  OK   FAIL: the `ownership-table` block of this document: it holds 0 rows and the stated minimum is 16
    PP27:ownership-table:emptied                 exit 2  OK   FAIL: the `ownership-table` block of this document: it holds 15 rows and the stated minimum is 1
    PP27:ownership-table:unmarked                exit 2  OK   FAIL: the `ownership-table` block of this document: the marker line appears 0 times under the he
    PP27:ownership-table:renamed                 exit 2  OK   FAIL: the `ownership-table` block of this document: the heading `#### The type ownership table` 
    PP27:ownership-table:stray                   exit 1  OK     MEMBER:     ownership-table: `duet-probe` | `ProbeMember` | 15.1: `duet-probe` is no crate of 
    PP27b:ownership-table:floor                  exit 1  OK     FLOOR:      ownership-table: the block holds 17 rows and the stated floor is 16; a floor below
    PP27:drop-list:deleted                       exit 2  OK   FAIL: the `drop-list` block of this document: it holds 0 rows and the stated minimum is 8; the g
    PP27:drop-list:emptied                       exit 2  OK   FAIL: the `drop-list` block of this document: it holds 7 rows and the stated minimum is 8; the g
    PP27:drop-list:unmarked                      exit 2  OK   FAIL: the `drop-list` block of this document: the marker line appears 0 times under the heading;
    PP27:drop-list:renamed                       exit 2  OK   FAIL: the `drop-list` block of this document: the heading `#### The candidate drop list` appears
    PP27:drop-list:stray                         exit 1  OK     MEMBER:     drop-list: Ticks: the drop list carries a name this document declares and no exter
    PP27b:drop-list:floor                        exit 1  OK     FLOOR:      drop-list: the block holds 9 rows and the stated floor is 8; a floor below the row
    PP27:constants:deleted                       exit 2  OK   FAIL: the `constants` block of this document: it holds 0 rows and the stated minimum is 78; the 
    PP27:constants:emptied                       exit 2  OK   FAIL: the `constants` block of this document: it holds 70 rows and the stated minimum is 78; the
    PP27:constants:unmarked                      exit 2  OK   FAIL: the `constants` block of this document: the marker line appears 0 times under the heading;
    PP27:constants:renamed                       exit 2  OK   FAIL: the `constants` block of this document: the heading `#### Every workspace constant` appear
    PP27:constants:stray                         exit 1  OK     MEMBER:     constants: // duet-probe: `duet-probe` is no crate of section 1.2
    PP27b:constants:floor                        exit 1  OK     FLOOR:      constants: the block holds 79 rows and the stated floor is 78; a floor below the r
    PP27:shared-limits:deleted                   exit 2  OK   FAIL: the `shared-limits` block of this document: it holds 0 rows and the stated minimum is 4; t
    PP27:shared-limits:emptied                   exit 2  OK   FAIL: the `shared-limits` block of this document: it holds 3 rows and the stated minimum is 4; t
    PP27:shared-limits:unmarked                  exit 2  OK   FAIL: the `shared-limits` block of this document: the marker line appears 0 times under the head
    PP27:shared-limits:renamed                   exit 2  OK   FAIL: the `shared-limits` block of this document: the heading `#### Every shared limit and its e
    PP27:shared-limits:stray                     exit 1  OK     MEMBER:     shared-limits: MAX_PROBE duet-time duet-core: `MAX_PROBE` is no constant of sectio
    PP27b:shared-limits:floor                    exit 1  OK     FLOOR:      shared-limits: the block holds 5 rows and the stated floor is 4; a floor below the
    PP27:probe-table:deleted                     exit 2  OK   FAIL: the `probe-table` block of this document: it holds 0 rows and the stated minimum is 60; th
    PP27:probe-table:emptied                     exit 2  OK   FAIL: the `probe-table` block of this document: it holds 59 rows and the stated minimum is 60; t
    PP27:probe-table:unmarked                    exit 2  OK   FAIL: the `probe-table` block of this document: the marker line appears 0 times under the headin
    PP27:probe-table:renamed                     exit 2  OK   FAIL: the `probe-table` block of this document: the heading `#### Every rule, its probe, and the
    PP27:probe-table:stray                       exit 1  OK     MEMBER:     probe-table: PG99 | A probe | PP99 | A probe | `exit 1`: `PG99` is no rule this pl
    PP27b:probe-table:floor                      exit 1  OK     FLOOR:      probe-table: the block holds 61 rows and the stated floor is 60; a floor below the
    PP27:external-verdicts:deleted               exit 2  OK   FAIL: the `external-verdicts` block of this document: it holds 0 rows and the stated minimum is 
    PP27:external-verdicts:emptied               exit 2  OK   FAIL: the `external-verdicts` block of this document: it holds 29 rows and the stated minimum is
    PP27:external-verdicts:unmarked              exit 2  OK   FAIL: the `external-verdicts` block of this document: the marker line appears 0 times under the 
    PP27:external-verdicts:renamed               exit 2  OK   FAIL: the `external-verdicts` block of this document: the heading `#### Every external type, and
    PP27:external-verdicts:stray                 exit 1  OK     MEMBER:     external-verdicts: Ticks: section 1.5 places this name, so no row decides it
    PP27b:external-verdicts:floor                exit 1  OK     FLOOR:      external-verdicts: the block holds 31 rows and the stated floor is 30; a floor bel
    PP27:external-paths:deleted                  exit 2  OK   FAIL: the `external-paths` block of this document: it holds 0 rows and the stated minimum is 66;
    PP27:external-paths:emptied                  exit 2  OK   FAIL: the `external-paths` block of this document: it holds 65 rows and the stated minimum is 66
    PP27:external-paths:unmarked                 exit 2  OK   FAIL: the `external-paths` block of this document: the marker line appears 0 times under the hea
    PP27:external-paths:renamed                  exit 2  OK   FAIL: the `external-paths` block of this document: the heading `#### Where every external name c
    PP27:external-paths:stray                    exit 1  OK     MEMBER:     external-paths: ProbeName     std::probe::ProbeName: `ProbeName` is named in no ot
    PP27b:external-paths:floor                   exit 1  OK     FLOOR:      external-paths: the block holds 67 rows and the stated floor is 66; a floor below 
    PP27:pins:deleted                            exit 2  OK   FAIL: the `pins` block of this document: it holds 0 rows and the stated minimum is 12; the guard
    PP27:pins:emptied                            exit 2  OK   FAIL: the `pins` block of this document: it holds 11 rows and the stated minimum is 12; the guar
    PP27:pins:unmarked                           exit 2  OK   FAIL: the `pins` block of this document: the marker line appears 0 times under the heading; the 
    PP27:pins:renamed                            exit 2  OK   FAIL: the `pins` block of this document: the heading `#### The external crate pins the roster co
    PP27:pins:stray                              exit 1  OK     MEMBER:     pins: probe-crate    1.0.0: the crate is in no section 1.2 dependency cell
    PP27b:pins:floor                             exit 1  OK     FLOOR:      pins: the block holds 13 rows and the stated floor is 12; a floor below the row co
    PP27:recorded-sizes:deleted                  exit 2  OK   FAIL: the `recorded-sizes` block of this document: it holds 0 rows and the stated minimum is 50;
    PP27:recorded-sizes:emptied                  exit 2  OK   FAIL: the `recorded-sizes` block of this document: it holds 49 rows and the stated minimum is 50
    PP27:recorded-sizes:unmarked                 exit 2  OK   FAIL: the `recorded-sizes` block of this document: the marker line appears 0 times under the hea
    PP27:recorded-sizes:renamed                  exit 2  OK   FAIL: the `recorded-sizes` block of this document: the heading `#### Every size the guard record
    PP27:recorded-sizes:stray                    exit 1  OK     MEMBER:     recorded-sizes: ProbeName                          8   8: `ProbeName` is no type t
    PP27b:recorded-sizes:floor                   exit 1  OK     FLOOR:      recorded-sizes: the block holds 51 rows and the stated floor is 50; a floor below 
    PP27:substitutions:deleted                   exit 2  OK   FAIL: the `substitutions` block of this document: it holds 0 rows and the stated minimum is 8; t
    PP27:substitutions:emptied                   exit 2  OK   FAIL: the `substitutions` block of this document: it holds 7 rows and the stated minimum is 8; t
    PP27:substitutions:unmarked                  exit 2  OK   FAIL: the `substitutions` block of this document: the marker line appears 0 times under the head
    PP27:substitutions:renamed                   exit 2  OK   FAIL: the `substitutions` block of this document: the heading `#### Every substitution the roste
    PP27:substitutions:stray                     exit 1  OK     MEMBER:     substitutions: probe probe-name         a probe: the row states no substitution id
    PP27b:substitutions:floor                    exit 1  OK     FLOOR:      substitutions: the block holds 9 rows and the stated floor is 8; a floor below the
    PP27:drop-impls:deleted                      exit 2  OK   FAIL: the `drop-impls` block of this document: it holds 0 rows and the stated minimum is 3; the 
    PP27:drop-impls:emptied                      exit 2  OK   FAIL: the `drop-impls` block of this document: it holds 2 rows and the stated minimum is 3; the 
    PP27:drop-impls:unmarked                     exit 2  OK   FAIL: the `drop-impls` block of this document: the marker line appears 0 times under the heading
    PP27:drop-impls:renamed                      exit 2  OK   FAIL: the `drop-impls` block of this document: the heading `#### Every declaration with a hand-w
    PP27:drop-impls:stray                        exit 2  OK   FAIL: the Drop block holds `ProbeName`, which section 1.5 places nowhere; the guard is fail-clos
    PP27b:drop-impls:floor                       exit 1  OK     FLOOR:      drop-impls: the block holds 4 rows and the stated floor is 3; a floor below the ro
    PP27:impl-sites:deleted                      exit 2  OK   FAIL: the `impl-sites` block of this document: it holds 0 rows and the stated minimum is 55; the
    PP27:impl-sites:emptied                      exit 2  OK   FAIL: the `impl-sites` block of this document: it holds 54 rows and the stated minimum is 55; th
    PP27:impl-sites:unmarked                     exit 2  OK   FAIL: the `impl-sites` block of this document: the marker line appears 0 times under the heading
    PP27:impl-sites:renamed                      exit 2  OK   FAIL: the `impl-sites` block of this document: the heading `#### Every impl block the roster com
    PP27:impl-sites:stray                        exit 1  OK     MEMBER:     impl-sites: ProbeName inherent: `ProbeName` is no declaration of this document
    PP27b:impl-sites:floor                       exit 1  OK     FLOOR:      impl-sites: the block holds 56 rows and the stated floor is 55; a floor below the 
    PP27:heap-names:deleted                      exit 2  OK   FAIL: the `heap-names` block of this document: it holds 0 rows and the stated minimum is 12; the
    PP27:heap-names:emptied                      exit 2  OK   FAIL: the `heap-names` block of this document: it holds 11 rows and the stated minimum is 12; th
    PP27:heap-names:unmarked                     exit 2  OK   FAIL: the `heap-names` block of this document: the marker line appears 0 times under the heading
    PP27:heap-names:renamed                      exit 2  OK   FAIL: the `heap-names` block of this document: the heading `#### Every heap-owning name the audi
    PP27:heap-names:stray                        exit 1  OK     MEMBER:     heap-names: Generation: `Generation` is a type this document declares
    PP27b:heap-names:floor                       exit 1  OK     FLOOR:      heap-names: the block holds 13 rows and the stated floor is 12; a floor below the 
    PP27:grow-names:deleted                      exit 2  OK   FAIL: the `grow-names` block of this document: it holds 0 rows and the stated minimum is 10; the
    PP27:grow-names:emptied                      exit 2  OK   FAIL: the `grow-names` block of this document: it holds 9 rows and the stated minimum is 10; the
    PP27:grow-names:unmarked                     exit 2  OK   FAIL: the `grow-names` block of this document: the marker line appears 0 times under the heading
    PP27:grow-names:renamed                      exit 2  OK   FAIL: the `grow-names` block of this document: the heading `#### Every growable name the audio r
    PP27:grow-names:stray                        exit 1  OK     MEMBER:     grow-names: Generation: `Generation` is a type this document declares
    PP27b:grow-names:floor                       exit 1  OK     FLOOR:      grow-names: the block holds 11 rows and the stated floor is 10; a floor below the 
    PP27:lock-names:deleted                      exit 2  OK   FAIL: the `lock-names` block of this document: it holds 0 rows and the stated minimum is 10; the
    PP27:lock-names:emptied                      exit 2  OK   FAIL: the `lock-names` block of this document: it holds 9 rows and the stated minimum is 10; the
    PP27:lock-names:unmarked                     exit 2  OK   FAIL: the `lock-names` block of this document: the marker line appears 0 times under the heading
    PP27:lock-names:renamed                      exit 2  OK   FAIL: the `lock-names` block of this document: the heading `#### Every lock name the audio rules
    PP27:lock-names:stray                        exit 1  OK     MEMBER:     lock-names: Generation: `Generation` is a type this document declares
    PP27b:lock-names:floor                       exit 1  OK     FLOOR:      lock-names: the block holds 11 rows and the stated floor is 10; a floor below the 
    PP27:primitive-traits:deleted                exit 2  OK   FAIL: the `primitive-traits` block of this document: it holds 0 rows and the stated minimum is 3
    PP27:primitive-traits:emptied                exit 2  OK   FAIL: the `primitive-traits` block of this document: it holds 2 rows and the stated minimum is 3
    PP27:primitive-traits:unmarked               exit 2  OK   FAIL: the `primitive-traits` block of this document: the marker line appears 0 times under the h
    PP27:primitive-traits:renamed                exit 2  OK   FAIL: the `primitive-traits` block of this document: the heading `#### Every primitive trait set
    PP27:primitive-traits:stray                  exit 1  OK     MEMBER:     primitive-traits: probe : Copy Default: `probe` is a sized primitive with no size 
    PP27b:primitive-traits:floor                 exit 1  OK     FLOOR:      primitive-traits: the block holds 4 rows and the stated floor is 3; a floor below 
    PP27:primitive-sizes:deleted                 exit 2  OK   FAIL: the `primitive-sizes` block of this document: it holds 0 rows and the stated minimum is 16
    PP27:primitive-sizes:emptied                 exit 2  OK   FAIL: the `primitive-sizes` block of this document: it holds 15 rows and the stated minimum is 1
    PP27:primitive-sizes:unmarked                exit 2  OK   FAIL: the `primitive-sizes` block of this document: the marker line appears 0 times under the he
    PP27:primitive-sizes:renamed                 exit 2  OK   FAIL: the `primitive-sizes` block of this document: the heading `#### Every primitive size the g
    PP27:primitive-sizes:stray                   exit 1  OK     MEMBER:     primitive-sizes: probe 1 1: `probe` is in no primitive trait set
    PP27b:primitive-sizes:floor                  exit 1  OK     FLOOR:      primitive-sizes: the block holds 17 rows and the stated floor is 16; a floor below
    PP27:justified-unknowns:deleted              exit 2  OK   FAIL: the `justified-unknowns` block of this document: it holds 0 rows and the stated minimum is
    PP27:justified-unknowns:emptied              exit 2  OK   FAIL: the `justified-unknowns` block of this document: it holds 1 rows and the stated minimum is
    PP27:justified-unknowns:unmarked             exit 2  OK   FAIL: the `justified-unknowns` block of this document: the marker line appears 0 times under the
    PP27:justified-unknowns:renamed              exit 2  OK   FAIL: the `justified-unknowns` block of this document: the heading `#### The justified unknowns`
    PP27:justified-unknowns:stray                exit 1  OK     MEMBER:     justified-unknowns: ProbeName: it is neither a framework name nor a declared name
    PP27b:justified-unknowns:floor               exit 1  OK     FLOOR:      justified-unknowns: the block holds 3 rows and the stated floor is 2; a floor belo
    PP27:vr1-table:deleted                       exit 2  OK   FAIL: the `vr1-table` block of this document: it holds 0 rows and the stated minimum is 27; the 
    PP27:vr1-table:emptied                       exit 2  OK   FAIL: the `vr1-table` block of this document: it holds 26 rows and the stated minimum is 27; the
    PP27:vr1-table:unmarked                      exit 2  OK   FAIL: the `vr1-table` block of this document: the marker line appears 0 times under the heading;
    PP27:vr1-table:renamed                       exit 2  OK   FAIL: the `vr1-table` block of this document: the heading `#### Every VR1 derive and its use` ap
    PP27:vr1-table:stray                         exit 1  OK     MEMBER:     vr1-table: ProbeName: no Rust block of this document declares it
    PP27b:vr1-table:floor                        exit 1  OK     FLOOR:      vr1-table: the block holds 28 rows and the stated floor is 27; a floor below the r
    PP27:audio-owned:deleted                     exit 2  OK   FAIL: the `audio-owned` block of this document: it holds 0 rows and the stated minimum is 40; th
    PP27:audio-owned:emptied                     exit 2  OK   FAIL: the `audio-owned` block of this document: it holds 39 rows and the stated minimum is 40; t
    PP27:audio-owned:unmarked                    exit 2  OK   FAIL: the `audio-owned` block of this document: the marker line appears 0 times under the headin
    PP27:audio-owned:renamed                     exit 2  OK   FAIL: the `audio-owned` block of this document: the heading `#### Every audio-owned declaration`
    PP27:audio-owned:stray                       exit 2  OK   FAIL: the audio-owned block holds `ProbeName`, which section 1.5 places nowhere; the guard is fa
    PP27b:audio-owned:floor                      exit 1  OK     FLOOR:      audio-owned: the block holds 41 rows and the stated floor is 40; a floor below the
    PP27:audio-exempt:deleted                    exit 2  OK   FAIL: the `audio-exempt` block of this document: it holds 0 rows and the stated minimum is 1; th
    PP27:audio-exempt:emptied                    exit 2  OK   FAIL: the `audio-exempt` block of this document: it holds 0 rows and the stated minimum is 1; th
    PP27:audio-exempt:unmarked                   exit 2  OK   FAIL: the `audio-exempt` block of this document: the marker line appears 0 times under the headi
    PP27:audio-exempt:renamed                    exit 2  OK   FAIL: the `audio-exempt` block of this document: the heading `#### Every audio-owned field the r
    PP27:audio-exempt:stray                      exit 2  OK   FAIL: the audio-exempt block holds `ProbeName.probe  probe`, which is no `Type.field` path; the 
    PP27b:audio-exempt:floor                     exit 1  OK     FLOOR:      audio-exempt: the block holds 2 rows and the stated floor is 1; a floor below the 
    PP27:audio-reachable-leaf:deleted            exit 2  OK   FAIL: the `audio-reachable-leaf` block of this document: it holds 0 rows and the stated minimum 
    PP27:audio-reachable-leaf:emptied            exit 2  OK   FAIL: the `audio-reachable-leaf` block of this document: it holds 43 rows and the stated minimum
    PP27:audio-reachable-leaf:unmarked           exit 2  OK   FAIL: the `audio-reachable-leaf` block of this document: the marker line appears 0 times under t
    PP27:audio-reachable-leaf:renamed            exit 2  OK   FAIL: the `audio-reachable-leaf` block of this document: the heading `#### Every reachable leaf 
    PP27:audio-reachable-leaf:stray              exit 1  OK     MEMBER:     audio-reachable-leaf: ProbeName: no Rust block of this document declares it
    PP27b:audio-reachable-leaf:floor             exit 1  OK     FLOOR:      audio-reachable-leaf: the block holds 45 rows and the stated floor is 44; a floor 
    PP27:block-members:deleted                   exit 2  OK   FAIL: the `block-members` block of this document: it holds 0 rows and the stated minimum is 47; 
    PP27:block-members:emptied                   exit 2  OK   FAIL: the `block-members` block of this document: it holds 46 rows and the stated minimum is 47;
    PP27:block-members:unmarked                  exit 2  OK   FAIL: the `block-members` block of this document: the marker line appears 0 times under the head
    PP27:block-members:renamed                   exit 2  OK   FAIL: the `block-members` block of this document: the heading `#### Every registered block and i
    PP27:block-members:stray                     exit 2  OK   FAIL: the `block-members` block of this document: `probe-block`: the register does not hold it; 
    PP27:budget-table:deleted                    exit 2  OK   FAIL: the `budget-table` block of this document: it holds 0 rows and the stated minimum is 132; 
    PP27:budget-table:emptied                    exit 2  OK   FAIL: the `budget-table` block of this document: it holds 131 rows and the stated minimum is 132
    PP27:budget-table:unmarked                   exit 2  OK   FAIL: the `budget-table` block of this document: the marker line appears 0 times under the headi
    PP27:budget-table:renamed                    exit 2  OK   FAIL: the `budget-table` block of this document: the heading `#### Every budget and bound` appea
    PP27:budget-table:stray                      exit 1  OK     MEMBER:     budget-table: B999: no other line of this document names it
    PP27b:budget-table:floor                     exit 1  OK     FLOOR:      budget-table: the block holds 133 rows and the stated floor is 132; a floor below 
    PP27:b1-convert:deleted                      exit 2  OK   FAIL: the `b1-convert` block of this document: it holds 0 rows and the stated minimum is 7; the 
    PP27:b1-convert:emptied                      exit 2  OK   FAIL: the `b1-convert` block of this document: it holds 6 rows and the stated minimum is 7; the 
    PP27:b1-convert:unmarked                     exit 2  OK   FAIL: the `b1-convert` block of this document: the marker line appears 0 times under the heading
    PP27:b1-convert:renamed                      exit 2  OK   FAIL: the `b1-convert` block of this document: the heading `#### Conversion suppressions` appear
    PP27:b1-convert:stray                        exit 1  OK     MEMBER:     b1-convert: probe_only_here: no other line of this document names it
    PP27b:b1-convert:floor                       exit 1  OK     FLOOR:      b1-convert: the block holds 8 rows and the stated floor is 7; a floor below the ro
    PP27:b1-complexity:deleted                   exit 2  OK   FAIL: the `b1-complexity` block of this document: it holds 0 rows and the stated minimum is 6; t
    PP27:b1-complexity:emptied                   exit 2  OK   FAIL: the `b1-complexity` block of this document: it holds 5 rows and the stated minimum is 6; t
    PP27:b1-complexity:unmarked                  exit 2  OK   FAIL: the `b1-complexity` block of this document: the marker line appears 0 times under the head
    PP27:b1-complexity:renamed                   exit 2  OK   FAIL: the `b1-complexity` block of this document: the heading `#### Complexity suppressions` app
    PP27:b1-complexity:stray                     exit 1  OK     MEMBER:     b1-complexity: duet-probe::probe::probe_site: `duet-probe` is no crate of section 
    PP27b:b1-complexity:floor                    exit 1  OK     FLOOR:      b1-complexity: the block holds 7 rows and the stated floor is 6; a floor below the
    PP27:b1-copy:deleted                         exit 2  OK   FAIL: the `b1-copy` block of this document: it holds 0 rows and the stated minimum is 4; the gua
    PP27:b1-copy:emptied                         exit 2  OK   FAIL: the `b1-copy` block of this document: it holds 3 rows and the stated minimum is 4; the gua
    PP27:b1-copy:unmarked                        exit 2  OK   FAIL: the `b1-copy` block of this document: the marker line appears 0 times under the heading; t
    PP27:b1-copy:renamed                         exit 2  OK   FAIL: the `b1-copy` block of this document: the heading `#### Expectations for missing_copy_impl
    PP27:b1-copy:stray                           exit 1  OK     MEMBER:     b1-copy: duet-probe::probe::ProbeName: `duet-probe` is no crate of section 1.2
    PP27b:b1-copy:floor                          exit 1  OK     FLOOR:      b1-copy: the block holds 5 rows and the stated floor is 4; a floor below the row c
    PP27:b1-variant:deleted                      exit 2  OK   FAIL: the `b1-variant` block of this document: it holds 0 rows and the stated minimum is 3; the 
    PP27:b1-variant:emptied                      exit 2  OK   FAIL: the `b1-variant` block of this document: it holds 2 rows and the stated minimum is 3; the 
    PP27:b1-variant:unmarked                     exit 2  OK   FAIL: the `b1-variant` block of this document: the marker line appears 0 times under the heading
    PP27:b1-variant:renamed                      exit 2  OK   FAIL: the `b1-variant` block of this document: the heading `#### Expectations for variant_size_d
    PP27:b1-variant:stray                        exit 1  OK     MEMBER:     b1-variant: duet-probe::probe::ProbeName: `duet-probe` is no crate of section 1.2
    PP27b:b1-variant:floor                       exit 1  OK     FLOOR:      b1-variant: the block holds 4 rows and the stated floor is 3; a floor below the ro
    PP27:phase-table:deleted                     exit 2  OK   FAIL: the `phase-table` block of this document: it holds 0 rows and the stated minimum is 16; th
    PP27:phase-table:emptied                     exit 2  OK   FAIL: the `phase-table` block of this document: it holds 15 rows and the stated minimum is 16; t
    PP27:phase-table:unmarked                    exit 2  OK   FAIL: the `phase-table` block of this document: the marker line appears 0 times under the headin
    PP27:phase-table:renamed                     exit 2  OK   FAIL: the `phase-table` block of this document: the heading `#### The phase table` appears 0 tim
    PP27:phase-table:stray                       exit 1  OK     MEMBER:     phase-table: 14 | Z9 | none | none | 0: `Z9` is no chunk of section 13.2
    PP27b:phase-table:floor                      exit 1  OK     FLOOR:      phase-table: the block holds 17 rows and the stated floor is 16; a floor below the
    PP27:selected-tests:deleted                  exit 2  OK   FAIL: the `selected-tests` block of this document: it holds 0 rows and the stated minimum is 8; 
    PP27:selected-tests:emptied                  exit 2  OK   FAIL: the `selected-tests` block of this document: it holds 7 rows and the stated minimum is 8; 
    PP27:selected-tests:unmarked                 exit 2  OK   FAIL: the `selected-tests` block of this document: the marker line appears 0 times under the hea
    PP27:selected-tests:renamed                  exit 2  OK   FAIL: the `selected-tests` block of this document: the heading `#### Every test this section sel
    PP27:selected-tests:stray                    exit 1  OK     MEMBER:     selected-tests: probe_test | Z9 | 0 | `probe.rs` | Plain | A probe: `Z9` is no chu
    PP27b:selected-tests:floor                   exit 1  OK     FLOOR:      selected-tests: the block holds 9 rows and the stated floor is 8; a floor below th
    PP27:snapshot-table:deleted                  exit 2  OK   FAIL: the `snapshot-table` block of this document: it holds 0 rows and the stated minimum is 6; 
    PP27:snapshot-table:emptied                  exit 2  OK   FAIL: the `snapshot-table` block of this document: it holds 5 rows and the stated minimum is 6; 
    PP27:snapshot-table:unmarked                 exit 2  OK   FAIL: the `snapshot-table` block of this document: the marker line appears 0 times under the hea
    PP27:snapshot-table:renamed                  exit 2  OK   FAIL: the `snapshot-table` block of this document: the heading `#### High-rate traffic: latest v
    PP27:snapshot-table:stray                    exit 1  OK     MEMBER:     snapshot-table: Audio | `duet-core` | `ProbeName` | Once per frame: `ProbeName` is
    PP27b:snapshot-table:floor                   exit 1  OK     FLOOR:      snapshot-table: the block holds 7 rows and the stated floor is 6; a floor below th
    PP27:rule-blocks:deleted                     exit 2  OK   FAIL: the `rule-blocks` block of this document: it holds 0 rows and the stated minimum is 32; th
    PP27:rule-blocks:emptied                     exit 2  OK   FAIL: the `rule-blocks` block of this document: it holds 31 rows and the stated minimum is 32; t
    PP27:rule-blocks:unmarked                    exit 2  OK   FAIL: the `rule-blocks` block of this document: the marker line appears 0 times under the headin
    PP27:rule-blocks:renamed                     exit 2  OK   FAIL: the `rule-blocks` block of this document: the heading `#### Which rule reads which block` 
    PP27:rule-blocks:stray                       exit 1  OK     MEMBER:     rule-blocks: PG99 | `drop-list`: `PG99` is no rule this plan implements
    PP27b:rule-blocks:floor                      exit 1  OK     FLOOR:      rule-blocks: the block holds 33 rows and the stated floor is 32; a floor below the
    PP27:closure-r16:deleted                     exit 2  OK   FAIL: the `closure-r16` block of this document: it holds 0 rows and the stated minimum is 46; th
    PP27:closure-r16:emptied                     exit 2  OK   FAIL: the `closure-r16` block of this document: it holds 45 rows and the stated minimum is 46; t
    PP27:closure-r16:unmarked                    exit 2  OK   FAIL: the `closure-r16` block of this document: the marker line appears 0 times under the headin
    PP27:closure-r16:renamed                     exit 2  OK   FAIL: the `closure-r16` block of this document: the heading `#### The revision-16 review, over t
    PP27:closure-r16:stray                       exit 1  OK     MEMBER:     closure-r16: PROBE | A probe | **CLOSED** | 1.9: the row states no finding id
    PP27b:closure-r16:floor                      exit 1  OK     FLOOR:      closure-r16: the block holds 47 rows and the stated floor is 46; a floor below the
    PP27:closure-r17:deleted                     exit 2  OK   FAIL: the `closure-r17` block of this document: it holds 0 rows and the stated minimum is 26; th
    PP27:closure-r17:emptied                     exit 2  OK   FAIL: the `closure-r17` block of this document: it holds 25 rows and the stated minimum is 26; t
    PP27:closure-r17:unmarked                    exit 2  OK   FAIL: the `closure-r17` block of this document: the marker line appears 0 times under the headin
    PP27:closure-r17:renamed                     exit 2  OK   FAIL: the `closure-r17` block of this document: the heading `#### The revision-17 review, over t
    PP27:closure-r17:stray                       exit 1  OK     MEMBER:     closure-r17: PROBE | A probe | **CLOSED** | 1.9: the row states no finding id
    PP27b:closure-r17:floor                      exit 1  OK     FLOOR:      closure-r17: the block holds 27 rows and the stated floor is 26; a floor below the
    PP27:closure-r18:deleted                     exit 2  OK   FAIL: the `closure-r18` block of this document: it holds 0 rows and the stated minimum is 20; th
    PP27:closure-r18:emptied                     exit 2  OK   FAIL: the `closure-r18` block of this document: it holds 19 rows and the stated minimum is 20; t
    PP27:closure-r18:unmarked                    exit 2  OK   FAIL: the `closure-r18` block of this document: the marker line appears 0 times under the headin
    PP27:closure-r18:renamed                     exit 2  OK   FAIL: the `closure-r18` block of this document: the heading `#### The revision-18 review, over t
    PP27:closure-r18:stray                       exit 1  OK     MEMBER:     closure-r18: PROBE | A probe | **CLOSED** | 1.9: the row states no finding id
    PP27b:closure-r18:floor                      exit 1  OK     FLOOR:      closure-r18: the block holds 21 rows and the stated floor is 20; a floor below the
    PP27:closure-r19:deleted                     exit 2  OK   FAIL: the `closure-r19` block of this document: it holds 0 rows and the stated minimum is 45; th
    PP27:closure-r19:emptied                     exit 2  OK   FAIL: the `closure-r19` block of this document: it holds 44 rows and the stated minimum is 45; t
    PP27:closure-r19:unmarked                    exit 2  OK   FAIL: the `closure-r19` block of this document: the marker line appears 0 times under the headin
    PP27:closure-r19:renamed                     exit 2  OK   FAIL: the `closure-r19` block of this document: the heading `#### The revision-19 review, over t
    PP27:closure-r19:stray                       exit 1  OK     MEMBER:     closure-r19: PROBE | A probe | **CLOSED** | 1.9: the row states no finding id
    PP27b:closure-r19:floor                      exit 1  OK     FLOOR:      closure-r19: the block holds 46 rows and the stated floor is 45; a floor below the
    PP27:closure-r20:deleted                     exit 2  OK   FAIL: the `closure-r20` block of this document: it holds 0 rows and the stated minimum is 23; th
    PP27:closure-r20:emptied                     exit 2  OK   FAIL: the `closure-r20` block of this document: it holds 22 rows and the stated minimum is 23; t
    PP27:closure-r20:unmarked                    exit 2  OK   FAIL: the `closure-r20` block of this document: the marker line appears 0 times under the headin
    PP27:closure-r20:renamed                     exit 2  OK   FAIL: the `closure-r20` block of this document: the heading `#### The revision-20 review, over t
    PP27:closure-r20:stray                       exit 1  OK     MEMBER:     closure-r20: PROBE | A probe | **CLOSED** | 1.9: the row states no finding id
    PP27b:closure-r20:floor                      exit 1  OK     FLOOR:      closure-r20: the block holds 24 rows and the stated floor is 23; a floor below the
    PP27:closure-r21-inner:deleted               exit 2  OK   FAIL: the `closure-r21-inner` block of this document: it holds 0 rows and the stated minimum is 
    PP27:closure-r21-inner:emptied               exit 2  OK   FAIL: the `closure-r21-inner` block of this document: it holds 12 rows and the stated minimum is
    PP27:closure-r21-inner:unmarked              exit 2  OK   FAIL: the `closure-r21-inner` block of this document: the marker line appears 0 times under the 
    PP27:closure-r21-inner:renamed               exit 2  OK   FAIL: the `closure-r21-inner` block of this document: the heading `#### The revision-21 inner re
    PP27:closure-r21-inner:stray                 exit 1  OK     MEMBER:     closure-r21-inner: PROBE | A probe | **CLOSED** | 1.9: the row states no finding i
    PP27b:closure-r21-inner:floor                exit 1  OK     FLOOR:      closure-r21-inner: the block holds 14 rows and the stated floor is 13; a floor bel
    PP27:line-map:deleted                        exit 2  OK   FAIL: the `line-map` block of this document: it holds 0 rows and the stated minimum is 16; the g
    PP27:line-map:emptied                        exit 2  OK   FAIL: the `line-map` block of this document: it holds 15 rows and the stated minimum is 16; the 
    PP27:line-map:unmarked                       exit 2  OK   FAIL: the `line-map` block of this document: the marker line appears 0 times under the heading; 
    PP27:line-map:renamed                        exit 2  OK   FAIL: the `line-map` block of this document: the heading `#### Every chunk line and the crate it
    PP27:line-map:stray                          exit 1  OK     MEMBER:     line-map: Z duet-probe: the row names no crate of section 1.2
    PP27b:line-map:floor                         exit 1  OK     FLOOR:      line-map: the block holds 17 rows and the stated floor is 16; a floor below the ro
    PP27:audio-asserted:deleted                  exit 2  OK   FAIL: the `audio-asserted` block of this document: it holds 0 rows and the stated minimum is 11;
    PP27:audio-asserted:emptied                  exit 2  OK   FAIL: the `audio-asserted` block of this document: it holds 10 rows and the stated minimum is 11
    PP27:audio-asserted:unmarked                 exit 2  OK   FAIL: the `audio-asserted` block of this document: the marker line appears 0 times under the hea
    PP27:audio-asserted:renamed                  exit 2  OK   FAIL: the `audio-asserted` block of this document: the heading `#### Every audio-owned root the 
    PP27:audio-asserted:stray                    exit 1  OK     MEMBER:     audio-asserted: ProbeName: no Rust block of this document declares it
    PP27b:audio-asserted:floor                   exit 1  OK     FLOOR:      audio-asserted: the block holds 12 rows and the stated floor is 11; a floor below 
    PP27:phase-pair-exempt:deleted               exit 2  OK   FAIL: the `phase-pair-exempt` block of this document: it holds 0 rows and the stated minimum is 
    PP27:phase-pair-exempt:emptied               exit 2  OK   FAIL: the `phase-pair-exempt` block of this document: it holds 23 rows and the stated minimum is
    PP27:phase-pair-exempt:unmarked              exit 2  OK   FAIL: the `phase-pair-exempt` block of this document: the marker line appears 0 times under the 
    PP27:phase-pair-exempt:renamed               exit 2  OK   FAIL: the `phase-pair-exempt` block of this document: the heading `#### Every same-phase crate e
    PP27:phase-pair-exempt:stray                 exit 1  OK     MEMBER:     phase-pair-exempt: Z9  Y9  a probe pair: `Z9` is no chunk of section 13.2
    PP27b:phase-pair-exempt:floor                exit 1  OK     FLOOR:      phase-pair-exempt: the block holds 25 rows and the stated floor is 24; a floor bel
    PP27:gate-defects:deleted                    exit 2  OK   FAIL: the `gate-defects` block of this document: it holds 0 rows and the stated minimum is 6; th
    PP27:gate-defects:emptied                    exit 2  OK   FAIL: the `gate-defects` block of this document: it holds 5 rows and the stated minimum is 6; th
    PP27:gate-defects:unmarked                   exit 2  OK   FAIL: the `gate-defects` block of this document: the marker line appears 0 times under the headi
    PP27:gate-defects:renamed                    exit 2  OK   FAIL: the `gate-defects` block of this document: the heading `#### Six planted gate defects, and
    PP27:gate-defects:stray                      exit 1  OK     MEMBER:     gate-defects: `ProbeName`, section 1.9: no declaration and no function of this doc
    PP27b:gate-defects:floor                     exit 1  OK     FLOOR:      gate-defects: the block holds 7 rows and the stated floor is 6; a floor below the 
    PP27:fault-messages:deleted                  exit 2  OK   FAIL: the `fault-messages` block of this document: it holds 0 rows and the stated minimum is 16;
    PP27:fault-messages:emptied                  exit 2  OK   FAIL: the `fault-messages` block of this document: it holds 15 rows and the stated minimum is 16
    PP27:fault-messages:unmarked                 exit 2  OK   FAIL: the `fault-messages` block of this document: the marker line appears 0 times under the hea
    PP27:fault-messages:renamed                  exit 2  OK   FAIL: the `fault-messages` block of this document: the heading `#### Every engine fault and the 
    PP27:fault-messages:stray                    exit 1  OK     MEMBER:     fault-messages: ProbeFault: `EngineFault` declares no such arm
    PP27b:fault-messages:floor                   exit 1  OK     FLOOR:      fault-messages: the block holds 17 rows and the stated floor is 16; a floor below 
    PROBES BAD: 0     RECORDED FRAGMENTS: 78     FLOOR: 45     RECORDED FRAGMENTS BAD: 0
    === probe_closure          exit 0  OK      0.9s
    BASE             exit 0  the unmodified block
    PP32-missing     exit 1  OK   one generated id with no row
    PP32-extra       exit 1  OK   one row for a finding the review does not state
    PP32-count       exit 1  OK   a count sentence the review does not generate
    PP32-nosection   exit 1  OK   a CLOSED row that names no section
    PP32-digest      exit 1  OK   a digest sentence that names a file this run did not read
    PP32-state       exit 1  OK   a state word outside the three the guard accepts
    PP32-blank       exit 1  OK   a row emptied of everything it says
    PP32-section     exit 1  OK   a CLOSED row naming a section no heading states
    PP32-duplicate   exit 1  OK   one row duplicated, with the state flipped
    PP32-prose       exit 1  OK   a CLOSED row whose section cell names no section at all
    PP32-moved       exit 1  OK   the digest sentence moved out of the block's own section
    PP32-format      exit 2  OK   a Warning in a shape no form of the parser reads, which the count sentence contradicts
    CLOSURE PROBES BAD: 0     RECORDED FRAGMENTS: 12     FLOOR: 12     FRAGMENTS BAD: 0
    === conversion_check       exit 0  OK      0.1s
    MEMBERS: 3   FILES: 6   FINDINGS: 0   EXPECTED MEMBERS: 3   EXPECTED FILES: 5
    === probe_conversion       exit 0  OK      2.3s
    CP1-a    exit 1  OK   a cast in a benches file
    CP1-b    exit 1  OK   a cast in a source module named target
    CP1-c    exit 1  OK   a cast in a file with an uppercase extension
    CP1-d    exit 2  OK   one path argument
    CP1b-exclude exit 2  OK   the Critic shape: five crates, one excluded, a real cast in it
    CP1b-empty exit 2  OK   a workspace with no member at all
    CP1b-file exit 2  OK   a member whose one target sits outside src, with a cast in src
    CP1b-target exit 2  OK   the Critic shape: a member at crates/target, excluded, with a real cast
    CP1b-nested exit 0  OK   a nested fixture crate the glob does not reach, which is the green control
    CP1b-dot exit 2  OK   the Critic shape: a member whose name opens with a dot, excluded, with a real cast
    CP1b-deep exit 2  OK   a member listed by full path with an excluded sibling one level up
    CP1b-top exit 2  OK   the Critic shape: a member with no slash and an excluded top-level sibling
    CP2      exit 0  OK   every false-hit form and no cast
    CP2b1    exit 1  OK   two strings that spell a comment pair
    CP2b2    exit 1  OK   a string that spells a line comment
    CP2b3    exit 1  OK   a `br` raw string with a bare quotation mark
    CP2b4    exit 1  OK   a `cr` raw string with a bare quotation mark
    CP2b5    exit 1  OK   an `rb` raw string with a bare quotation mark
    CP3      exit 1  OK   one bare cast
    CP3b     exit 1  OK   a comparison pair that holds a cast
    CP4      exit 0  OK   both use forms and a macro pattern
    CP4b     exit 2  OK   a use item with no terminating semicolon
    CP5      exit 0  OK   two qualified path spans
    CP6      exit 1  OK   five suppression forms
    CP7a     exit 0  OK   the sanctioned cast in the exempt file
    CP7b     exit 1  OK   the exempt path as a link to a live module
    CP8a     exit 2  OK   a manifest that does not parse
    CP8b     exit 2  OK   a member file that is not UTF-8
    CP8c     exit 2  OK   a member file the guard cannot open
    CONVERSION PROBES BAD: 0     RECORDED FRAGMENTS: 32     FLOOR: 29     FRAGMENTS BAD: 0
    === closure_check r16      exit 0  OK      0.1s
    REVIEW: critic-spec-r16.md   BLOCK: closure-r16   GENERATED: 46   ROWS: 46   CLOSURE BAD: 0
    === closure_check r17      exit 0  OK      0.1s
    REVIEW: critic-spec-r17.md   BLOCK: closure-r17   GENERATED: 26   ROWS: 26   CLOSURE BAD: 0
    === closure_check r18      exit 0  OK      0.1s
    REVIEW: critic-spec-r18.md   BLOCK: closure-r18   GENERATED: 20   ROWS: 20   CLOSURE BAD: 0
    === closure_check r19      exit 0  OK      0.1s
    REVIEW: critic-spec-r19.md   BLOCK: closure-r19   GENERATED: 45   ROWS: 45   CLOSURE BAD: 0
    === closure_check r20      exit 0  OK      0.1s
    REVIEW: critic-spec-r20.md   BLOCK: closure-r20   GENERATED: 23   ROWS: 23   CLOSURE BAD: 0
    === closure_check r21-inner exit 0  OK      0.1s
    REVIEW: critic-spec-r21-inner.md   BLOCK: closure-r21-inner   GENERATED: 13   ROWS: 13   CLOSURE BAD: 0
    === roster_compile         exit 0  OK    112.2s
    ROSTER CRATES:   17
    ROSTER ITEMS:    405     FLOOR: 405
    ROSTER IMPL MIXED: 0
    ROSTER IMPL BLOCKS: 55     FLOOR: 55
    ROSTER IMPLS:    23
    ROSTER DROPS:    3
    ROSTER CONSTS:   8
    ROSTER SUBS:     8
    ROSTER LOCK:     887 packages in the repository lock, 873 after resolution
    ROSTER LINT:     cargo clippy --workspace --all-targets -- -D warnings
    ROSTER CLIPPY:   clean
    ROSTER SIZES:    429 measured
    RECORDED ROWS:   37 checked     HEAD ROWS: 13     SIZE BAD: 0
    === probe_roster           exit 0  OK     49.4s
    ROSTER CRATES:   17
    ROSTER ITEMS:    405     FLOOR: 405
    ROSTER IMPL MIXED: 0
    ROSTER IMPL BLOCKS: 55     FLOOR: 55
    ROSTER IMPLS:    23
    ROSTER DROPS:    3
    ROSTER CONSTS:   8
    ROSTER SUBS:     8
    ROSTER LOCK:     887 packages in the repository lock, 873 after resolution
    ROSTER LINT:     cargo clippy --workspace --all-targets -- -D warnings
    ROSTER CLIPPY:   clean
    ROSTER SIZES:    429 measured
    RECORDED ROWS:   37 checked     HEAD ROWS: 13     SIZE BAD: 0
    BASE         exit 0
    PP25-eq      exit 1  OK   duet-agent/src/lib.rs:5:24: error: you are deriving `PartialEq` and can implement `Eq`: help: consider deriving `Eq` as well: `PartialEq, Eq`
    PP25-drop    exit 1  OK   duet-midi/src/lib.rs:56:1: error: type could implement `Copy`; consider adding `impl Copy`
    PP25-floor   exit 1  OK   FAIL: the roster holds 404 items and section 1.5 places 405; the first name with no declaration is RenderJob.
    GATE-allow   exit 1  OK   duet-score/src/lib.rs:58:3: error: #[allow] attribute found: help: replace it with: `expect`
    GATE-unwrap  exit 1  OK   duet-time/src/lib.rs:98:22: error: used `unwrap()` on `Some` value
    GATE-as      exit 1  OK   duet-time/src/lib.rs:102:50: error: casting `f64` to `f32` may truncate the value
    PP25-impl    exit 1  OK   FAIL: the roster parsed 54 impl blocks and the `impl-sites` block lists 55; the two are one set and they differ by 1.
    GATE15-allow exit 1  OK   duet-engine/src/lib.rs:117:3: error: #[allow] attribute found: help: replace it with: `expect`
    GATE15-unwrap exit 1  OK   duet-engine/src/lib.rs:183:22: error: used `unwrap()` on `Some` value
    GATE15-as    exit 1  OK   duet-engine/src/lib.rs:131:22: error: casting `i64` to `u16` may truncate the value
    GATE17-allow exit 1  OK   duet-engine/src/lib.rs:171:11: error: #[allow] attribute found: help: replace it with: `expect`
    GATE17-unwrap exit 1  OK   duet-engine/src/lib.rs:183:22: error: used `unwrap_or_default()` on `None` value
    GATE17-as    exit 1  OK   duet-dsp/src/lib.rs:104:19: error: casting `i64` to `u16` may truncate the value
    PP25-mixed   exit 1  OK   FAIL: the impl block for Transport carries a body beside a bodiless signature, so the whole block leaves the roster; write every item as a signature o
    ROSTER FRAGMENTS: 14     FLOOR: 14     FRAGMENTS BAD: 0     TRANSCRIPT LINES: 13     TRANSCRIPT BAD: 0
    GATE FRAGMENTS:   6     FLOOR: 6     GATE BAD: 0
    ROSTER PROBES BAD: 0     FRAGMENTS BAD: 0
    GATES RUN: 13     GATES BAD: 0     MODE: FULL
    EXIT=0
```

---

## 2. My own probes, beyond the recorded harness

I planted at least four probes per guard, in throwaway copies, and three gate defects of my own.

### 2.1 `run_all_gates.py`, the new machine of revision 21

| Probe | What I planted | Result |
|---|---|---|
| G1 | Nothing; `--fast` baseline over a throwaway copy | `exit 0`; `GATES RUN: 11     GATES BAD: 0     MODE: FAST (no roster)` |
| G2 | A background `touch` of `reviews/critic-spec-r20.md` twelve seconds into the run | **`exit 2`**; `FAIL: 1 file(s) ... changed while this run was in progress`, with the path named. **The guard rejects the defect it exists to reject.** |
| G3 | One new review file, `reviews/critic-spec-r22.md`, in the plan tree | `exit 0`; `GATES RUN: 11`. No closure gate ran for it. See WARNING 3. |
| G4 | The tree's own `architecture.md` damaged, a pristine copy passed as the argument | `exit 0`. The tree compare covers the plan directory and never the document. See the Concerns table. |
| G5 | A scratch directory inside the repository, and a missing `Cargo.toml` | `exit 2` both times, each with a named fail-closed line. |

### 2.2 PG37, the new budget-value rule

| Probe | What I planted | Result |
|---|---|---|
| A1 | The B120 row at `8 slots`, the recorded shape | `exit 1`; `VALUE: B120: the row states 8 and the declaration line that cites it writes 4, 9` |
| A2 | The B120 row at **`4 slots`**, which is `REVERB_SLOTS` on the line ABOVE the citing line | **`exit 0`; `VALUE BAD: 0`.** See WARNING 2. |
| A3 | The B113 row at `12 elements`, the nineteenth Critic's shape | `exit 1`; the rule names 13 |
| A4 | Every budget id removed from every Rust and text fence | `VALUE ROWS: 0`, and PG37 itself raised nothing |
| A5 | The `(B120)` tag removed from its one citing line | `VALUE ROWS: 8`, `VALUE BAD: 0`; PG30 answered instead, not PG37 |

### 2.3 PG32, the closure guard

| Probe | What I planted | Result |
|---|---|---|
| C1 | One CLOSED row flipped to PARTIAL with an empty section cell | `exit 0`. This is the stated rule and not a defect. |
| C2 | One CLOSED row whose Finding cell says nothing true, with a real section number | `exit 0`. PG32 states this limit at its own site. |
| C3 | A seven-edit coordinated drop of one Warning, listed in WARNING 1 | **`exit 0` on every gate.** See WARNING 1. |
| C4 | A block id the register does not hold | `exit 2`, fail-closed |
| C5 | The review heading level changed to `###` | `exit 2`; the count sentence and the heading scan disagree |

### 2.4 The conversion guard

| Probe | What I planted | Result |
|---|---|---|
| CV1 | A real cast in a member of a throwaway workspace | `exit 1`; `CAST: crates/alpha/src/lib.rs: line 2` |
| CV2b | `exclude = ["crates/alpha"]` with the `crates/*` glob kept | `exit 2`; `FAIL: crates/alpha holds a Cargo.toml and the scan does not cover it` |
| CV2c | The `crates/*` glob DELETED, with no `exclude` | `exit 0`; `EXPECTED MEMBERS: 1`. See the Concerns table. |
| CV3 | A member whose one target sits outside `src/`, with a cast inside `src/` | `exit 1`, and the file count rose above the expected set |
| CV4 | A `cfg_attr`-wrapped `allow(clippy::as_conversions)` beside a cast | `exit 1`; both the SUPPRESSION and the CAST were named |
| CV5 | The same unlisted crate reached by a path dependency | `exit 1`; cargo makes it an automatic member and CG1 scans it |

### 2.5 Three gate defects of my own, against the roster compile

The six recorded gate defects are `#[allow]`, `unwrap` and `as`. Mine are three other denied
classes. Each one ran over a throwaway copy of the document and one shared cargo target.

| Gate defect | Where I planted it | Result |
|---|---|---|
| GD1 | `unsafe { core::ptr::null::<u8>().is_null() }` inside `Finite::from_finite_const` | `exit 1`; `duet-time/src/lib.rs:98:22: error: usage of an `unsafe` block` |
| GD2 | `panic!("probe")` in the same body | `exit 1`; `duet-time/src/lib.rs:98:9: error: only a `panic!` in `if`-then statement` |
| GD3 | A slice index in the `PoolHandle` `Debug` body | `exit 1`; `duet-engine/src/lib.rs:113:22: error: indexing may panic` |

**The roster gate carries the repository lint policy correctly.** `roster_compile.sh` relaxes six
documentation lints and nothing else, so `unsafe_code`, `panic`, `indexing_slicing`, `unwrap_used`
and `allow_attributes` all stay denied inside the generated workspace.


---

## 3. Closure check on every row of the three reviews

`closure_check.py` answered `CLOSURE BAD: 0` for all six blocks, and `GENERATED` equals `ROWS` at
46, 26, 20, 45, 23 and 13. **It agrees with me on every count and on every row shape.** It decides
nothing about the truth of a row, and PG32 states that limit at its own site, so the disagreements
below are not defects in the tool.

**My totals over the 81 rows: 62 CLOSED, 14 PARTIAL, 5 OPEN.** The document records 77 CLOSED and
4 OPEN.

### 3.1 The revision-19 review, 45 rows

**33 are CLOSED.** C19-1, C19-3, C19-5, C19-6, C19-7, C19-8, C19-9, C19-10, C19-11, C19-W1 to
C19-W8, C19-W10 to C19-W17, C19-W19, C19-W20, C19-W21, C19-W24, C19-W25, N19-5, N19-7 and N19-8
each name a mechanism that exists and does what the row claims. I re-ran every probe the row cites
and each one turns the run red.

**Eight are PARTIAL and four are correctly OPEN.**

| Id | My verdict | Section | Reason |
|---|---|---|---|
| C19-2 | PARTIAL | 1.5 PG32 CL1b, `review_ids.py` | The second source is a second sentence of the same file, under the same hand. |
| C19-4 | PARTIAL | 13.1, M0's cell | The cell is truncated by an unescaped `\|\|`, and `tools/xtask/Cargo.toml` is in no write scope. |
| C19-W9 | PARTIAL | 1.5 PG37 | The oracle still passes a wrong value that the adjacent declaration line carries; probe A2. |
| C19-W18 | PARTIAL | 1.6 B125, B126 | Both rows exist, both cite the wrong story, and both call a SHOULD a MUST. |
| C19-W22 | PARTIAL | `research/crate-survey.md` | The `arc-swap` row at line 60 carries no correction note; the note went to `ardour-concepts.md` alone. |
| C19-W23 | PARTIAL | 9.5 rule 8 | Step 6 saves every document with no bound, and the completeness sentence denies the wait. |
| N19-2 | PARTIAL | `placement_check.py`, `run_all_gates.py` | `CLOSURE_SECTIONS` and the review list of `run_all_gates.py` are still literals that track the document. |
| N19-3 | PARTIAL | 1.9, the harness table | `ROSTER_TARGET_DIR` is a second environment seam, it performs `rm -rf`, and no section names it. |
| N19-1 | OPEN, correct | 1.5 PG31b | I rebuilt the pair set independently and found the same 24 pairs. |
| N19-4 | OPEN, correct | 13.0 SM4, SM5 | No rule holds a chunk's write path to the crate it owns. |
| N19-6 | OPEN, correct | 1.5 PG33 | The register reads a suppression's site and not every function that carries one. |
| N19-9 | OPEN, correct | Appendix C | The cross-document work list is still a work list; I closed part of it and report it below. |

### 3.2 The revision-20 review, 23 rows

**20 are CLOSED.** C20-1, C20-2, C20-3, C20-5, C20-W1 to C20-W5, C20-W7, C20-W9, C20-W10 and
N20-1 to N20-8 each hold at the section or the tool the row names.

| Id | My verdict | Section | Reason |
|---|---|---|---|
| C20-4 | PARTIAL | B.5, the timeout table | B18 and B116 are correct reports; B119 and B132 bound a blocking `join` on the painting thread. |
| C20-W6 | PARTIAL | C.24, the row itself | An unescaped `\|\|` breaks the row, and its `typos` claim is unreadable. |
| C20-W8 | PARTIAL | B.3 | `proptest` is pinned at 1.11.0 and `criterion` carries no version, against B.3's own rule. |

### 3.3 The revision-21 inner review, 13 rows

**Nine are CLOSED.** C21-1, C21-W2, C21-W3, C21-W4, C21-W5 and N21-1 to N21-4 each hold. I checked
`PlaybackPlan`, `TrackPlayback` and `PlaybackSpan` in section 15.10, the `fault-messages` block at
sixteen rows, and the `fault-arm` membership rule, and each one is as the row states.

| Id | My verdict | Section | Reason |
|---|---|---|---|
| C21-2 | PARTIAL | 9.5 rule 8 | Step 6 has no bound, and step 5's bound sits on the thread TH3 forbids it on. |
| C21-3 | PARTIAL | 5.5, `PoolHandle` | The `PoolHandle` doc comment still names four fixed numbers where the pool holds six. |
| C21-W1 | PARTIAL | 1.5 PG37 | Probe A2 plants `4 slots` for B120 and the run is green, against the rule's own claim. |
| C21-W6 | **OPEN** | 5.5 `ChainState` | The row names latch bits on `ChainState`, and that declaration holds no latch field. |


---

## 4. New findings

## CRITICAL 1. `ChainState` holds no latch bits, so the fault flood the inner review measured is not fixed

**OBSERVATION.** The C21-W6 row cites "5.5 `ChainState`, which holds the latch bits the runner
clears when the condition stops". `ChainState` is declared at architecture.md:4801. Its fields are
`strip`, `writer`, `reader`, `trim`, `pre_fader`, `fader`, `post_fader`, `meter` and `reset_seen`.
No latch field exists. I grepped every use of the word "latch" in the document. Only
`ChainSlot::reported` and `MeterSnapshot::latched` are declared, and neither serves
`PlaybackStarved`, `CaptureOverflow` or `CaptureShortfall`.

**CLAIM.** The closure row names a mechanism the declaration refutes, and the defect it closes is
still live.

**ARGUMENT.** The inner review computed the cost: up to twelve thousand records a second at B1 and
B7. The doc comment at architecture.md:9671 repeats that arithmetic and states the remedy. The
remedy has no home. The same doc comment says the `frames` payload now carries "the count of the
whole run of cycles", and no declared field accumulates that count either. This is the C19-1 class
one revision later: a CLOSED row whose mechanism the artifact does not hold. PG32 cannot see it,
because PG32 checks shape.

**EVIDENCE.** architecture.md:4801 to :4823 is the declaration. architecture.md:9671 to :9690 is
the doc comment. architecture.md:14480 is the closure row. `grep -n -i latch architecture.md`
returns no `ChainState` field.

**WHAT SHOULD CHANGE.** Declare the latch bits and the run counter on `ChainState`. State which
conditions each bit covers. Then mark C21-W6 OPEN until the declaration exists.

## CRITICAL 2. Two quit-path bounds put a timed blocking join on the GPUI foreground thread

**OBSERVATION.** Appendix B.5 gives B119 and B132 one mechanism each: "`JoinHandle::join` behind a
bounded park", on "The quit path, on the GPUI foreground thread through `cx.spawn`". The Feature
column reads "None" for both.

**CLAIM.** TH3 forbids a timed wait on that thread, and the thread table forbids blocking work on
it, so the two rows contradict the two rules that own the thread.

**ARGUMENT.** TH3 reads: "no git walk, no file scan, no import parse, no render, **no timed wait**
... runs on the GPUI foreground thread." The thread table row for the GPUI foreground thread reads
"Never does: Blocking input or output, audio work". A bounded park is a timed wait and
`JoinHandle::join` is a blocking call. The same appendix states the correct shape one row above:
B18 and B116 are reports the CORE measures, "because the core is waiting for a MESSAGE". B131 uses
that shape. B119 and B132 do not. The B95 row states the principle in the plainest words the
document has: "no host, no GPUI task, and no thread that paints is involved". B119 is five seconds
and B132 is a further wait, so a quit on a stalled device freezes the window for both.

**EVIDENCE.** architecture.md:13514 and :13516 are the two rows. architecture.md:6245 is TH3.
architecture.md:6190 is the thread table. architecture.md:6764 and :6770 are the section 5.12 rows.

**WHAT SHOULD CHANGE.** Make both waits a message the core measures, as B131 already is. Let each
worker and the handoff thread send a completion event, and abandon on expiry from the core loop.
Then state the thread of each row, as the B18 row does.

## CRITICAL 3. The quit-path save has no bound, and the completeness sentence denies that it waits

**OBSERVATION.** Section 9.5 rule 8 step 6 reads: "The core runs the atomic save of section 4.10
for every dirty document, and it releases the writer lock." The paragraph below it reads: "Steps 1,
6 and 9 perform no cross-boundary wait."

**CLAIM.** Step 6 is the one step of the quit path that writes files, and the sentence that claims
the path is bounded excludes it by assertion.

**ARGUMENT.** Section 4.10 gives the atomic save five steps and names eight `fsync` calls plus one
per temporary file and one per touched directory. An `fsync` on a network volume is exactly the
wait the design already grants B117 and B118 for on the disk thread, and the B117 and B118 rows say
so: "a blocking file call on macOS and on Linux cannot be interrupted". The core runs step 6, and
in the application the core runs on the GPUI foreground thread. So the quit path performs unbounded
blocking file input and output on the painting thread, between two steps that are bounded. The
ordering makes it worse: step 5 has already abandoned every job worker, so no job thread is left to
carry the save.

**EVIDENCE.** architecture.md:8262 is step 6. architecture.md:8285 is the completeness sentence.
architecture.md:4337 to :4352 is the five-step save. architecture.md:6190 is the thread table row.

**WHAT SHOULD CHANGE.** Give step 6 a budget row and a stated action on expiry. Name the thread
that runs it, and order it before step 5 if it needs a worker. Then correct the completeness
sentence.

## CRITICAL 4. `CaptureInfo` names no track, so no declared message ends one take per armed track

**OBSERVATION.** architecture.md:7034 declares
`pub struct CaptureInfo { start: SuperClock, frames: u64, loop_offset: u64, underruns: u32 }`.
`EngineEvent::CaptureDone(CaptureInfo)` is at architecture.md:11978. Neither holds a `TrackId` and
neither holds `TakeFlags`.

**CLAIM.** PR R-05 is a MUST and the design cannot satisfy it with the declared message.

**ARGUMENT.** PR R-05 reads: "Given several armed tracks, when the user records, then the
application writes one take per armed track." Section 6.4 states that the pass "reports
`CaptureInfo` plus the flags it observed to `duet-core`", and no declared value carries those
flags. With many armed tracks the core receives several identical-shaped events and cannot map one
to a take. Appendix B.5 makes it concrete: the B131 row says the core "counts the
`EngineEvent::CaptureDone` events it still owes", and the B131 expiry action is to mark **every
unfinished take**. A count cannot name an unfinished track; only a set can.

**EVIDENCE.** architecture.md:7034, :11978, :6996, :8250, :13515. product-requirements.md:401.

**WHAT SHOULD CHANGE.** Add `track: TrackId` and the observed `TakeFlags` to `CaptureInfo`. Restate
the B131 rule over the set of owed tracks.

## CRITICAL 5. An unescaped table pipe truncates chunk M0's write scope, so eleven policy files leave the plan

**OBSERVATION.** architecture.md:10068 is chunk M0's row of the section 13.1 table. It parses to
seven cells under a five-column header, because the cell text carries a literal `\|\|` inside
`` `|| printf` ``.

**CLAIM.** M0's write scope and its Completion command are in the dropped columns, and section 13.0
cites that cell as the evidence that M0 owns the policy files.

**ARGUMENT.** A GitHub-flavoured renderer keeps the first five cells of a ragged row and drops the
rest. Column three then ends mid-word and column four is empty. The files that vanish are
`scripts/bootstrap.sh`, `deny.toml`, `.cargo/config.toml`, `.github/workflows/ci.yml`,
`clippy.toml`, `.gitignore`, `.claude/skills/gpui-kit/SKILL.md`, `crates/duet/Cargo.toml`,
`Info.plist`, `assets/fonts/` and `NOTICE`, together with the Completion command. Those are the SM4
policy set, and SM4 is the rule that keeps a policy edit out of an implementation chunk. The same
defect sits at architecture.md:14424, where the C20-W6 closure row parses to six cells under a
four-column header and its raw prose is already garbled inside a backtick span.
**No guard sees either.** Section 13.1 is not a registered block, so DR7 gives it no marker.
`closure_check.py` reads `row[3]`, finds the token "14", and passes CL4 on the truncated cell.

**EVIDENCE.** architecture.md:10068 and :14424. I parsed both rows and counted seven and six cells.

**WHAT SHOULD CHANGE.** Escape both pipes. Add a column-count rule over every section 13 chunk
table and every closure block, because a ragged row is invisible today.

## CRITICAL 6. The session holds unbounded slot and send lists that the fixed runtime cannot accept

**OBSERVATION.** architecture.md:7054 declares `pre_fader: Vec<SlotConfig>` on `Strip`, :7057
`post_fader: Vec<SlotConfig>`, and :7058 `sends: Vec<AuxSend>`. architecture.md:4740 declares
`pre_fader: ArrayVec<SlotSpec, MAX_SLOTS>` on `ChainTopology` and :4742
`sends: ArrayVec<SendSpec, MAX_SENDS>`. B45 is 8 and B46 is 8.

**CLAIM.** No declared error arm refuses a ninth slot, and `configure` has no legal answer.

**ARGUMENT.** `MixError` declares `RoutingCycle`, `BusRoleReserved`, `StripBudgetExceeded`,
`ParamBudgetExceeded`, `MissingStrip`, `MissingParam`, `EmptyCurve` and `DuplicatePoint`.
`ConfigError` declares `ChannelMismatch`, `PoolExhausted`, `RingBudgetExceeded`,
`StripBudgetExceeded`, `ParamBudgetExceeded`, `PoolHandoffBusy` and `HandoffRetryPending`. Neither
enum names a slot count or a send count. So `SessionCommand::StripInsertSlot` accepts a ninth slot
and `configure` must build an `ArrayVec` of capacity eight from nine values. `clippy::panic` and
`clippy::indexing_slicing` are both denied, so silence and truncation are the only remaining
behaviours, and both are wrong. The constant-reach block also withholds `MAX_SLOTS` from
`duet-session`, so the crate that owns the `Vec` cannot read its own bound.

**EVIDENCE.** architecture.md:7054, :7057, :7058, :4740, :4742, :1084, :1085, :1297, and the
`MixError` and `ConfigError` blocks.

**WHAT SHOULD CHANGE.** Add `MixError::SlotBudgetExceeded` and `MixError::SendBudgetExceeded`. Give
`duet-session` the two constants. State the refusal in the `# Errors` contract of
`MixState::validate` and at section 7.1.

## CRITICAL 7. No publication carries a per-slot measurement, so two MUST meters have no source

**OBSERVATION.** The `snapshot-table` block holds six rows. Its two audio-to-core rows publish
`TransportSnapshot` and `MeterSnapshot`. `MeterReading` at architecture.md:7322 holds `peak`,
`rms`, `true_peak` and `hold`, per strip. `StageCurve` at architecture.md:13160 holds
`live: Option<Finite>`, per slot, with no declared producer.

**CLAIM.** PR M-03 and the limiter bar of section 10.3 both ask for a per-slot measurement, and no
boundary of section 5.8 carries one.

**ARGUMENT.** PR M-03 is a MUST: "Given a compressor, when audio passes through it, then the strip
shows the gain reduction as a live meter." Contract 5.2 asks the compressor for a live dot at the
input level and the limiter for a gain-reduction bar. `ParamSnapshot` carries parameter values from
core to audio, which is the wrong direction for a measurement. `MeterSnapshot` is per strip and
holds no slot dimension. So `StageCurve::live` is a field no thread can fill, and one scalar is
being asked to carry two different quantities.

**EVIDENCE.** architecture.md:6318 to :6324, :7322, :13160, :8187. design-contract.md:890.
product-requirements.md:518.

**WHAT SHOULD CHANGE.** Add a per-slot measurement array to `MeterSnapshot`, or declare a second
audio-to-core publication. Give it a budget row and count it in B8. Split `live` into the two
quantities the contract draws.

---

## WARNING 1. Seven coordinated edits hide a Warning from the closure record, and every gate stays green

**OBSERVATION.** I deleted the `## WARNING 6` section of `critic-spec-r21-inner.md`, retyped that
review's own closing count line, dropped the C21-W6 row, retyped the document count sentence,
retyped the digest sentence, lowered the `rows>=` marker, and edited the `placement_check.py`
register. `review_ids.py` printed `WARNINGS: 5   TOTAL: 12   COUNT SENTENCE: agrees`.
`closure_check.py` printed `GENERATED: 12   ROWS: 12   CLOSURE BAD: 0` and exited 0.
`placement_check.py` exited 0.

**CLAIM.** CL1b is not a second source. It is a second sentence of the file the first source reads.

**ARGUMENT.** The C19-2 row says CL1b "reads the review's own count sentence as a second source". A
source is independent when a different party owns it. The Architect owns the document, the path
`reviews/critic-spec-<n>.md`, and both guard registers. CL5 binds the block to the file at that
path, and the same hand owns the path. The nineteenth review measured six edits; revision 21 made
it seven. A count of edits is not the property the rule needs.

**EVIDENCE.** My probe C3, above. The seventh edit is one line of
`placement_check.py` `DATA_BLOCKS`. `roster_compile.sh` carries a second copy of that register, and
section 1.9 already records the duplicate as a known defect with chunk M0 as its owner.

**WHAT SHOULD CHANGE.** Take the second source from outside the Architect's write scope. The Critic
stores every review with `db.sh append`, so record the store key beside the digest and make the
guard read the stored copy.

## WARNING 2. PG37 accepts a wrong budget value that the neighbouring declaration line carries

**OBSERVATION.** I changed the B120 row from `9 slots` to `4 slots` and ran
`placement_check.py`. The run printed `VALUE ROWS: 9     VALUE BAD: 0` and exited 0. 4 is
`REVERB_SLOTS`, declared on the line immediately above the line that cites B120.

**CLAIM.** The rule narrowed the C21-W1 defect from a block to a three-line window and did not
remove it, and both the rule text and the code comment claim otherwise.

**ARGUMENT.** `budget_value_audit` builds a SET of every capacity in the window of the line before,
the citing line, and the line after, over every Rust and text fence, and passes when the stated
value is anywhere in that set. The PP37 cell records the oracle as `4, 9` for B120, so the document
prints the evidence of its own hole. The code comment states: "so a planted `4 slots`, `8 slots`,
`48 slots` or `2048 slots` is red now". My probe shows `4 slots` is green.

**EVIDENCE.** `tools/placement_check.py:3113` to :3175. architecture.md:1511 is the PP37 cell.
architecture.md:1253 to :1258 is the constant block. My probes A1 and A2.

**WHAT SHOULD CHANGE.** Take the capacity from the citing line alone, or require the id and the
capacity on one line. A doc comment above a constant is one line, so the window can close to one.

## WARNING 3. `run_all_gates.py` holds its review list as a literal, so a new review gets no closure gate

**OBSERVATION.** `run_all_gates.py` builds its closure gates from
`for revision in ("r16", "r17", "r18", "r19", "r20", "r21-inner")`. I copied one review to
`reviews/critic-spec-r22.md` and ran the harness. It printed `GATES RUN: 11     GATES BAD: 0` and
exited 0, and no closure gate ran for the new file.

**CLAIM.** The script that exists to replace a habit carries its own denominator as a habit.

**ARGUMENT.** The module docstring says the script "runs every guard and every harness of this
plan", and section 1.9 repeats it. The set of reviews is readable from the directory. The revision
this review produces will be the seventh, and nothing fails when the list is not extended. Section
1.9's own words apply: "A habit cannot hold that rule and a script can."

**EVIDENCE.** `tools/run_all_gates.py`, the `for revision in (...)` line. My probe G3.

**WHAT SHOULD CHANGE.** Derive the list from `reviews/critic-spec-*.md` and fail when a review file
has no registered block, and when a registered closure block has no review file.

## WARNING 4. `probe_roster.sh` deletes a cargo target its caller owns, at an unvalidated path

**OBSERVATION.** `probe_roster.sh` sets `export ROSTER_TARGET_DIR="${ROSTER_TARGET_DIR:-$SCRATCH/target}"`
and then traps `rm -rf "$ROSTER_TARGET_DIR"` on `EXIT`, `INT` and `TERM`, whatever the source of the
value. My gate run set `ROSTER_TARGET_DIR` to a shared path and the script deleted it.

**CLAIM.** The script breaks the ownership contract section 1.9 states, and it performs `rm -rf` on
a path no rule validates.

**ARGUMENT.** Section 1.9 reads: "A caller that exports `ROSTER_TARGET_DIR` takes that ownership
instead." `roster_compile.sh` honours that rule and sets no trap in the exported branch.
`probe_roster.sh` sets the trap unconditionally. Both scripts refuse a scratch directory inside the
repository; neither validates `ROSTER_TARGET_DIR`. `run_all_gates.py` copies the whole environment
into every child, so the seam reaches the documented command. A mistyped value deletes whatever it
names.

**EVIDENCE.** `tools/probe_roster.sh` lines 47 and 48. `tools/roster_compile.sh` lines 58 to 63.
Section 1.9, the one-target paragraph. My own shared target was deleted by the recorded run.

**WHAT SHOULD CHANGE.** Trap only the target this script created. Refuse a `ROSTER_TARGET_DIR`
inside the repository, exactly as both scripts already refuse a scratch directory there.

## WARNING 5. `tools/xtask/Cargo.toml` is in no write scope, so chunk M0 stops under SM0

**OBSERVATION.** M0 writes five `tools/xtask/src/check_*.rs` modules and `tools/xtask/tests/probes.rs`.
The string `tools/xtask/Cargo.toml` appears nowhere in the document.

**CLAIM.** M0 needs dependencies that neither its pin list nor its write scope allows.

**ARGUMENT.** CL5 makes `check_closure` compute an md5. The live `tools/xtask/Cargo.toml` declares
`anyhow`, `clap`, `serde_json` and `toml`, and no md5 crate sits in `[workspace.dependencies]`.
M0's pin list is `serde`, `serde_json`, `thiserror`, `smallvec` and `proptest`. Every Python
prototype also parses with regular expressions, and `regex` is pinned nowhere. SM1 makes the using
chunk add the `{ workspace = true }` entry to its own member manifest in the same commit, and that
manifest is outside M0's scope. SM0 then makes the engineer report and stop.

**EVIDENCE.** architecture.md:10068 is M0's cell. architecture.md:900 and :923 are CL5 and the
`check_closure.rs` scope. `/Users/james/Developer/duet/tools/xtask/Cargo.toml` and the root
`[workspace.dependencies]`.

**WHAT SHOULD CHANGE.** Add `tools/xtask/Cargo.toml` to M0's write scope and add the digest and
parser pins to its pin list, or state that the port carries a hand-written digest and parser.

## WARNING 6. ADR 0006 gives the linear path cache two owner sets, and B61 counts two caches where it names three

**OBSERVATION.** `adr/0006-wrapped-timeline.md:47` reads: "`RecordView` owns it in Record, and
`MixView` owns it in Mix and Master." Line 55 of the same record reads: "`MixView` holds one and
`MasterView` holds one." The B61 row reads: "**Two caches exist**: `RecordView` holds the wrapped
one and `MasterView` and `MixView` each hold a linear one."

**CLAIM.** One record states two owner sets for one bounded resource, and one budget row contradicts
itself inside its own sentence.

**ARGUMENT.** DR1 requires a revision to rewrite the sentence it changes. Decision 6 stands beside
decision 8, and a reader who follows decision 6 gives Master no cache. Section 10.2 makes `MixView`
and `MasterView` siblings with no edge, so the decision-6 shape cannot be built. The count carries
weight too: B61 bounds 64 MB per cache, so three caches bound 192 MB.

**EVIDENCE.** `adr/0006-wrapped-timeline.md:47` and :55. architecture.md:1100 and :9188 to :9194.
The C16-15 and C17-3 closure rows both say CLOSED and cite decision 8 alone.

**WHAT SHOULD CHANGE.** Rewrite decision 6 to name three owners. Correct the B61 row to state three.

## WARNING 7. Section 1.7, the one rule index, omits four live ids

**OBSERVATION.** Section 1.7 indexes "PG1 to PG36, with PG4b, PG10b, PG20b, PG26b to PG26f, and
PG31b" and "PP1 to PP36". The document states PG27b, PG37, PP27b and PP37 as rules and probes of
their own.

**CLAIM.** The index the document declares as the one index of rule ids is stale by four ids,
including the rule this revision added.

**ARGUMENT.** DR4 makes section 1.7 the index. DR5 says a lettered id is a rule in its own right.
No guard reads section 1.7, so nothing caught it. The same row also calls PG26b, PG26c and PG26d
"the three TH1 rules", and section 5.7 names PG26, PG26b and PG26c.

**EVIDENCE.** architecture.md:1309 and :1311, against :1508 and :1511 and
`tools/placement_check.py:3113`.

**WHAT SHOULD CHANGE.** Extend both ranges and correct the TH1 row. Then give the index a rule that
reads it, or state that it is unverified text.

## WARNING 8. B125 and B126 cite the wrong story and promote a SHOULD to a MUST

**OBSERVATION.** B125 reads "which PR R-07 makes a MUST" and B126 reads "which PR R-09 makes a
MUST". PR R-07 is "Record a layered take (MUST)" and states no pitch deviation. PR R-09 is
"Layered takes and take selection (MUST)" and states no free-space threshold.

**CLAIM.** Both citations name a story that states neither threshold, and both raise a SHOULD to a
MUST.

**ARGUMENT.** The 25-cent criterion is PR R-15, "Pitch track overlay (SHOULD)". The
one-gigabyte criterion is PR R-14, "Disk and performance warnings (SHOULD)". The C19-W18 closure
row calls both "MUST thresholds", so the appendix records a closure over a false premise, and
section 6.3 repeats the R-09 citation.

**EVIDENCE.** architecture.md:1162, :1163, :6944, :14369. product-requirements.md:472 to :479.

**WHAT SHOULD CHANGE.** Cite R-15 and R-14. Delete the word MUST from both rows and from section
6.3. Restate the C19-W18 row.

## WARNING 9. `criterion` carries no version, against Appendix B.3's own rule

**OBSERVATION.** Appendix B.3 states "Each one needs a version" and adds that "the version below is
therefore the source". The `criterion` row carries "**M4 resolves and records it**" in the Version
column. No `criterion` version exists anywhere in the document.

**CLAIM.** The appendix breaks the rule it states one line above, and the C20-W8 closure row claims
a closure over two crates while it pinned one.

**ARGUMENT.** The C20-W8 row says CLOSED and cites "B.3, which gains a Version column and pins
`proptest` at 1.11.0". `proptest` and `criterion` were both named by the finding. The Version cell
and the B.5 State cell now carry the same instruction, which is a duplicate instruction and not a
pin.

**EVIDENCE.** architecture.md:13426, :13431, :13450, :13495 and :14426.

**WHAT SHOULD CHANGE.** Write a `criterion` version in B.3, or restate C20-W8 as PARTIAL.

## WARNING 10. The `arc-swap` row of the crate survey carries no correction note

**OBSERVATION.** `research/crate-survey.md:60` still advises "store only from the UI side" for
`arc-swap`. ADR 0004 decision 12 states that `arc-swap` is a dependency of no Duet crate. The same
supersession carries a dated note in `research/ardour-concepts.md:232`.

**CLAIM.** The preamble rule of this document is broken at one of the four sites it names.

**ARGUMENT.** The preamble reads: "a superseded line that stands with no note is a second answer a
reader can find and follow". Four notes exist and each is dated 2026-09-21. The `arc-swap`
supersession is recorded in `ardour-concepts.md` and not in `crate-survey.md`, which carries its own
`arc-swap` row, so a reader of the survey meets no note.

**EVIDENCE.** `research/crate-survey.md:60`; `research/ardour-concepts.md:232`;
`adr/0004-backend-and-threading-contract.md:161`.

**WHAT SHOULD CHANGE.** Add a dated note to the `crate-survey.md` `arc-swap` row.

## WARNING 11. `Take` has no `muted` field, so `SessionCommand::TakeSetMuted` writes nothing

**OBSERVATION.** `Take` holds `id`, `name`, `regions` and `flags`.
`SessionCommand::TakeSetMuted { take: TakeId, muted: bool }` is declared. `TakeFlag` declares
`Uncalibrated`, `HadShortfall`, `HadOverrun`, `HadDrift` and `MidiPortLost`.

**CLAIM.** A declared command writes a field the aggregate does not hold.

**ARGUMENT.** Section 13.2 states plainly that the command "carries R-10", and chunk K3 gets "a
mute toggle per take". PR R-10 asks that both passes appear and each can be muted. None of the five
flags is a mute. The same note records that revision 16 named both stories and declared no command;
revision 21 added the command and left the state.

**EVIDENCE.** architecture.md:6865, :3474, :10444, :10452, :6960. product-requirements.md:440.

**WHAT SHOULD CHANGE.** Add `muted: bool` to `Take`, or add the mute as a `TakeFlag` arm and say so.

## WARNING 12. Per-part export has no field on any declared type

**OBSERVATION.** `ExportSpec` holds `container`, `sample_format`, `sample_rate`, `dither` and
`normalization`. `AudioExportRequest` holds `path`, `spec` and `overwrite`. Neither names a part.

**CLAIM.** PR MA-05 is assigned to three chunk cells and no declared value can express it.

**ARGUMENT.** PR MA-05 asks the application to write one file per part with the part name and to
show which part it writes now. Section 13.2 maps MA-05 to `ExportSpec` and tells chunk K5 to write
"the per-part choice". The `ExportSpec` doc comment states that the record "fully determines a
render", so the gap is a contradiction and not an omission. This is the C19-W20 class, which the
same review closed for `CreateRequest`.

**EVIDENCE.** architecture.md:7522, :7527, :11631, :10306, :10448. product-requirements.md:595.

**WHAT SHOULD CHANGE.** Add a part selection to `AudioExportRequest`, or move MA-05 out of version
one and delete the three chunk cells.

## WARNING 13. The alignment formula names a monitor term that no type and no budget holds

**OBSERVATION.** Section 6.4 states: "The offset is `LatencyReport::capture` plus
`LatencyReport::playback` plus the monitor chain latency." `LatencyReport` holds `capture`,
`playback` and `block`. `Calibration` holds `round_trip_frames`.

**CLAIM.** The third term has no type, no budget row and no owner.

**ARGUMENT.** DR3 makes section 1.6 the one table of numbers this design chooses, and no row states
a monitor chain latency. The term decides where every take lands on the timeline, and PR R-07 and
PR R-08 are both MUST stories that depend on it. An implementer must invent the value.

**EVIDENCE.** architecture.md:6992, :4426, :4620, and the section 1.6 table.

**WHAT SHOULD CHANGE.** Declare the term and give it a B row, or derive it from the declared slot
latencies and state the derivation at section 6.4.

## WARNING 14. A curve holds an unbounded point list with no budget and no refusal

**OBSERVATION.** `CurvePoints::Beats(Vec<(Ticks, Finite)>)` and
`CurvePoints::Audio(Vec<(SuperClock, Finite)>)` are the two arms. `Curve::new` returns
`MixError::EmptyCurve` and `MixError::DuplicatePoint` only.

**CLAIM.** One collection of this design grows without a bound, a budget row or a refusal.

**ARGUMENT.** PR M-07 asks that write-mode automation records a fader movement during playback. A
ride recorded at frame rate over a long project is bounded by nothing. `thin` exists and no rule
invokes it and no number sizes it. Every other collection carries a B id and a refusal: B86 bounds
strips, B90 bounds parameters, B128 and B129 bound spans. The list also reaches `automation.jsonl`
and the B101 snapshot, so the growth crosses two boundaries that state a size.

**EVIDENCE.** architecture.md:7218 to :7224, :7244, :7267, :3964, :1140.

**WHAT SHOULD CHANGE.** Add a B row for the points one curve may hold. Add
`MixError::CurveBudgetExceeded` and return it from `Curve::new` and `MixState::validate`.

## WARNING 15. `Strip` has no name field, so `BusAdd` collects a value nothing stores

**OBSERVATION.** `Strip` holds no name. `SessionCommand::BusAdd { name: StripName, role: BusRole }`
and `BusAddRequest { name: StripName, role: BusRole }` both carry one. `StripName` is documented as
"A strip name the user reads and edits."

**CLAIM.** Two declared inputs carry a value that no declaration stores and no surface reads.

**ARGUMENT.** Section 7.1 derives every strip header from the role, so the collected name is never
used. No `StripRename` arm exists and `ElementRef` holds only `Note`, `Spanner` and `Mark`, so the
doc comment states an edit path that does not exist. The section 3.5 vocabulary row also lists
`StripName` as a map key inside `Session` and `MixState`, and neither map keys by it.

**EVIDENCE.** architecture.md:7049, :3486, :11652, :11284, :7176, :11630, :3568.

**WHAT SHOULD CHANGE.** Add `name: StripName` to `Strip` with a rename path, or delete the field
from both inputs and correct the doc comment and the section 3.5 row.

## WARNING 16. Chunk C4 mixes platform-specific work with one cross-platform completion command

**OBSERVATION.** C4 delivers "the PipeWire host selection by id" with the cpal backend. Its
Completion is `cargo nextest run -p duet-engine -E 'test(calibration_split) + test(host_select)'
--no-tests=fail`, with no runner named.

**CLAIM.** C4 breaks SM3 rule 3, and nothing proves its Linux half at the end of phase 7.

**ARGUMENT.** SM3 rule 3 asks a platform-specific chunk for a per-platform command and the runner of
each. G2 and G3 obey it and C4 does not. PipeWire host selection is Linux only. `+` is a union in a
nextest filterset, so a macOS run that matches `calibration_split` alone still passes
`--no-tests=fail`. The test that would prove the PipeWire half is `pipewire_smoke`, which is
`#[ignore]` and runs only in `audio-smoke.yml`, which chunk M8 writes one phase later.

**EVIDENCE.** architecture.md:10166, :9880, :10725, :10702.

**WHAT SHOULD CHANGE.** Split C4's Completion into a macOS command and a Linux command and name
each runner. State where `host_select` lives and whether a `cfg` gates it.

## WARNING 17. `SlotConfig` and `AuxSend` each live in two containers of one aggregate

**OBSERVATION.** `MixState` holds `sends: BTreeMap<SendId, AuxSend>` and
`slots: BTreeMap<SlotId, SlotConfig>`. `Strip` holds `pre_fader: Vec<SlotConfig>`,
`post_fader: Vec<SlotConfig>` and `sends: Vec<AuxSend>`.

**CLAIM.** One fact has two owners, and no sentence names which copy wins.

**ARGUMENT.** This document states the rule twice: "two encodings of one state cannot disagree". A
slot edit must write two containers, and a read of one proves nothing about the other.
`MixState::validate` checks cycles, roles and budgets and never the agreement of the two copies.
`MixDocument` then serializes one shape and the document does not say which.

**EVIDENCE.** architecture.md:11320 to :11326, :7054 to :7058, :11340, :7195, :6898, :3492.

**WHAT SHOULD CHANGE.** Make `Strip` hold `Vec<SlotId>` and `Vec<SendId>` in order and leave
`MixState` the one owner of each value. State the choice at section 7.1.

## WARNING 18. `ZoomStep` is persisted and deserialized with no bound and no guarded constructor

**OBSERVATION.** `pub struct ZoomStep(u8)` derives `Deserialize` and offers
`pub const fn new(step: u8) -> Self`. `state/view.json` persists it. The design contract defines
six steps. No clamp and no refusing constructor appears at any of the twenty `ZoomStep` sites.

**CLAIM.** A hand-edited view document can set a zoom step outside the six the contract defines, and
nothing refuses it.

**ARGUMENT.** ADR 0002 decision 4b and section 2.6a put a guarded constructor on the
deserialization path of `Finite` for exactly this reason. A hand edit of a canonical file is a
product path this design already recognises. ADR 0005 decision 16c says a bad view document must
fall back and never block the project, and a step of 200 is a readable document with an unreadable
value.

**EVIDENCE.** architecture.md:8675 to :8681 and :3805; `adr/0002-storage-format.md:42`.

**WHAT SHOULD CHANGE.** Give `ZoomStep` a stated maximum and a refusing constructor, or state the
clamp the view loader applies.

## WARNING 19. B99 opens the staff above the contract default, and no zoom map exists

**OBSERVATION.** `design-contract.md:231` reads: "Default `sp` = 8.0 px. Zoom steps set `sp` to 6,
7, 8, 10, 12, and 16 px." B99 sets the first-open zoom step to 4, and
`pub const FIRST_OPEN_ZOOM: ZoomStep = ZoomStep::new(4);` writes it. Step 4 of that list is 12 px,
which the contract reads as 150 percent.

**CLAIM.** The application opens the staff at 150 percent, and no document states the function that
turns a `ZoomStep` into a staff size.

**ARGUMENT.** `ZoomStep::scalar` returns a `ZoomScalar` for the linear timeline only.
`LayoutOptions::pixels_per_staff_space` is a bare `f32` with no stated derivation, and the string
appears once in the whole document. So the wrapped timeline has no declared zoom mapping and
chunks A1 and K2 must invent one. The neighbouring rows prove the omission is not a style choice:
B94, B97 and B98 each name the contract row that sets the value, and B99 names none.

**EVIDENCE.** `design-contract.md:231`; architecture.md:1138, :1237, :8676 to :8686, :11737.

**WHAT SHOULD CHANGE.** Declare the step-to-`sp` map in one place. Then set B99 to the contract
default, or state in the B99 row why the plan overrides the contract.

---

## Concerns

The inner review of this revision already used the ids `N21-1` to `N21-4` for its own Concerns.
Mine are the Concerns of the EXTERNAL revision-21 review and they collide by id. The collision is
itself the first row.

| Id | Concern | Disposition |
|---|---|---|
| N21-1 | Two reviews of revision 21 both generate `N21-n` ids, so Appendix C will hold two blocks with colliding ids and `review_ids.py` cannot tell them apart. | File. PG32 should key an id on the review file and not on the revision number alone. Record it in section 1.5 PG32. |
| N21-2 | The `PoolHandle` doc comment still reads "B47, B48, B49, and B50 are each a fixed number" where the pool now holds six fixed numbers. It is the fifth site of C21-3 and its conclusion is still true. | File in `roadmap/duet-v1/architecture.md` section 5.5 at the next edit of that block. |
| N21-3 | PG37 decides 9 budget rows of 132 and prints `VALUE SKIPPED: 123`. It has no floor of its own; probe A5 dropped it to 8 and PG30 answered instead. | File. The denominator is honest and readable; a floor would make a silent shrink red. |
| N21-4 | CG1b derives its expected member set from the `members` globs of the manifest under test. Probe CV2c deleted the `crates/*` glob and the run was green with a real cast in `crates/alpha`. | File. An unglobbed crate is built by nothing, so the miss has no runtime cost. State the limit beside the two the rule already states. |
| N21-5 | `run_all_gates.py` never flushes stdout, so a redirected run prints nothing for six minutes and a reader cannot tell it from a hang. | File. One `sys.stdout.flush()` per gate. |
| N21-6 | `run_all_gates.py` accepts any document path and time-checks only the plan directory. Probe G4 ran a pristine copy against a damaged tree. The first gate does print `DOCUMENT:` with the path, so the transcript is honest. | File. Refuse a document outside the plan directory. |
| N21-7 | `CLOSURE_SECTIONS` in `placement_check.py` is still a literal that tracks the document, which N19-2 recorded and `sync_floors.py` does not write. | File with N19-2. |
| N21-8 | The latch paragraph of `EngineFault` sits in the doc comment of `FaultsDropped`, the one arm it does not describe, beside that arm's own sentence. | File. Move it to the enum doc or to the three arms it binds. |
| N21-9 | SM8 says section 13.4 carries one row for every edge that binds. Under the wide reading 27 section 1.3 edges have no row; all 27 are order-correct today. | File. State that the rule covers same-phase edges only. |
| N21-10 | `rust-toolchain.toml` is in no chunk's write scope. The plan adds about thirty pins over a toolchain at 1.98.1, and a pin whose minimum rustc is higher has no owner. | File in section 13.1 against chunk M0. |
| N21-11 | The phase-0 row names "the four xtask guards" and M0's own cell writes five `check_*.rs` modules. | File. DR3 exists to remove exactly this. |
| N21-12 | ADR 0004 decision 22 says the record names four version pins and it names three crates. | File. Delete the count and let the pins be the fact. |
| N21-13 | ADR 0004 decisions 8e, 8e.1 and 8e.2 restate the section 5.5 migration mechanism over about twenty lines, against DR6. Decisions 15 and 16c in the same record apply DR6 correctly. | File. Cut to the decision plus the citation. |
| N21-14 | ADR 0004 decision 8e.3 writes "The handoff ring is capacity one" where B105 owns the number, and DR3 exempts no such form. | File. Replace the words with the B id. |
| N21-15 | `DuetTokens` declares 27 fields and contract 7.2 states 29 tokens. `duet.wave.fill` and `duet.wave.rms` appear nowhere in the architecture, and the WR-20 closure row claims every role is carried. | File. Add the two fields, or state the derivation and correct WR-20. |
| N21-16 | ADR 0004's rejected-alternatives row says the two research files disagree about the `pipewire` crate. `crate-survey.md:10` now records the caller path as safe, so the two files agree. | File. Rewrite the row. |
| N21-17 | A stop during the count-in has no stated take outcome, and PR R-06 is a MUST that asks for no take. | File. State at section 6.3 that a pass with zero captured frames commits no take. |
| N21-18 | Rung one's `shellcheck` bullet is conditional, because `dod.sh` runs the tool only when it is installed. No line names who installs it in CI. | File in section 14 rung one. |
| N21-19 | Chunk K1's write scope says "the eleven stubs under `src/element/`", and SM2 says a directory is not a write scope. The eleven are named about 1,400 lines earlier. | File. Name them in the cell, as the twenty shell files already are. |

**Four earlier Concerns stay open as recorded debt and I confirm each one.** N19-1: I rebuilt the
PG31b pair set independently and found the same 24 pairs, so the rule rejects nothing today. N19-4:
no rule holds a chunk's write path to the crate it owns. N19-6: PG33 reads a suppression's site and
not every function that carries one. N19-9: the cross-document work list is still open, and I
closed part of it in WARNING 6, 8, 9, 10, 18, 19 and in eight Concerns above.

---

## Reactive Assessment

- **Responsive: FAIL.** The quit path saves every document with no bound, and two of its waits are a
  blocking join on the thread that paints.
- **Resilient: FAIL.** No declared field latches a per-track fault, so one starved track floods the
  B34 queue, the B29 channel and every client.
- **Elastic: PARTIAL.** Every ring, channel and pool carries a bound. One automation curve and the
  session slot and send lists grow without one.
- **Message Driven: PARTIAL.** Almost every boundary is a bounded message with a stated overflow
  rule. `CaptureDone` carries no identity, no publication carries a per-slot measurement, and two
  quit-path waits are a join rather than a message.

---

## Verdict

**NOT READY FOR PLAN AUTHORING.**

The guard apparatus is in good order and I say so plainly. All thirteen gates are green over one
tree. `run_all_gates.py` rejects the defect it exists to reject: my probe G2 turned it red with the
changed file named. Every recorded probe reproduced. My three own gate defects, `unsafe`, `panic!`
and a slice index, each turned the roster compile red with the correct lint. The closure guard
answered all nine of its own hostile shapes and four of mine. This is the strongest tooling I have
reviewed in this plan.

**The single biggest risk is that the guards read shape and the defects are in meaning.** Seven of
my seven Criticals are invisible to every rule that runs. PG32 passed a closure row that names a
field its own declaration does not hold. PG35, PG36 and every membership rule passed a chunk row
that a markdown renderer truncates. No rule reads a `Vec` in the session against an `ArrayVec` in
the engine. The plan has built a machine that proves a document is internally well formed, and the
remaining work is to make the document true.

**The weakest Reactive property is Resilient.** The inner review computed that an unlatched
per-track fault is twelve thousand records a second, the document repeats that arithmetic, and the
type that must carry the fix has no field for it.

**Blocking list, in the order I would fix it.**

1. CRITICAL 1. Declare the latch, because the closure record says it exists and it does not.
2. CRITICAL 5. Escape the two pipes, because chunk M0's write scope is unreadable until then.
3. CRITICAL 2, 3 and 4. The quit path and the capture message. They are one seam and three defects.
4. CRITICAL 6 and 7. The session-to-engine bounds and the missing per-slot publication.
5. WARNING 1, 2, 3 and 4. The four guard defects, because every later closure rests on them.
6. WARNING 5. Chunk M0's manifest, because M0 is the first chunk of phase 0.
7. WARNING 6 to 19. None is deferrable; each one is a named contradiction with a one-line fix.

The nineteen Concerns above are filed with the disposition each row states.

---

**`architecture.md` md5 at the END of my run: `1fd9efa3b89837c80a904a48ce1e68c7`. The two agree.**

This review holds 7 Criticals, 19 Warnings, and 19 Concerns.
