---
id: K5
line: K
depends_on: [K4, D2, H1]
write_scope:
  - crates/duet/src/element/lufs_meter.rs
  - crates/duet/src/master/view.rs
  - crates/duet/src/master/export_dialog.rs
  - crates/duet/src/master/report.rs
  - crates/duet/src/shell/toolbar_master.rs
  - crates/duet/benches/
  - crates/duet/Cargo.toml
  - Cargo.lock
parallelism: "serial-only: phase 13 holds one chunk, because line K has six chunks and no cross-line dependency exists after phase 11 (section 13.3)"
completion: "cargo nextest run -p duet -E 'test(master_view) + test(export_dialog)' --no-tests=fail passes; cargo clippy -p duet --all-targets -- -D warnings is clean; commit SHA on a branch chunk/k5-master-work-area"
---

# K5: The Master work area, the loudness meter, and the export dialog

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. This chunk fills the Master work area that chunk K1 created as a seam. It delivers
`LufsMeter`, `MasterView` with one stage box per `MasterStage::ORDER` arm and no other,
`MasterMeterLayer`, the `criterion` bench over one Master frame, the export dialog with the per-part
choice and the rate and bit-depth choice, the measured report, and the Master `ModeToolbar`. It
implements architecture sections 7.4, 10.2, 10.3, 10.5, 10.7 and 15.16, and design contract sections
5.1 to 5.5 and 8.4. It carries product stories MA-01, MA-02, MA-03, MA-04, MA-05, MA-06 and X-13.

**Master caches nothing, and this chunk states what it pays instead** (critic WR-26). `StageCurve`
derives `IntoElement`, so it is a `RenderOnce` element with no entity and no `EntityId`, and
`Entity::cached` takes an `Entity<T>` where `T: Render`. The mitigation is a bound and a
precomputation:

1. **The count is five and not 48.** `MasterStage::ORDER` holds five arms and contract 5.2 draws one
   box per arm.
2. **Each build reads samples and evaluates no filter.** `StageCurve::params` is an `Arc<[Finite]>`
   that `MasterView` fills from `MasterView::stages` whenever a parameter changes, which is not per
   frame. A build is one pointer copy and one polyline over stored samples.
3. **A bench measures it.** This chunk owns a `criterion` bench over one Master frame at B1, exactly
   as chunk K4 owns one over a Mix frame, and both report against B4.

## Files

- `crates/duet/src/element/lufs_meter.rs` — modify. Documentation stub.
- `crates/duet/src/master/view.rs` — modify. Chunk K1 left a seam view.
- `crates/duet/src/master/export_dialog.rs` — modify. Documentation stub.
- `crates/duet/src/master/report.rs` — modify. Documentation stub.
- `crates/duet/src/shell/toolbar_master.rs` — modify. Replace one `render` body and one
  `group_count` body. Edit `top_bar.rs` not at all.
- `crates/duet/benches/master_frame.rs` — create.
- `crates/duet/Cargo.toml` — modify. The second `[[bench]]` target with `harness = false`.
- `Cargo.lock` — modify (SM5 rule 2).

**This chunk modifies no file of chunk K3.** `PathCache` is chunk K3's file; this chunk names the
type and fills its own `MasterView::cache` field (section 10.5).

## Types and signatures

### Declared by this chunk, in `crates/duet/src/master/view.rs`

Copied from architecture section 15.16.

```rust
/// The Master work area.
#[derive(Debug)]
pub(crate) struct MasterView {
    meters: Entity<MasterMeterLayer>,
    /// **Master owns its own linear path cache** (critic C16-15, C17-3).
    cache: PathCache,
    /// The one owner of this view's path build task.
    build: Option<Task<()>>,
    /// The samples-per-pixel scalar of the linear timeline.
    zoom: ZoomScalar,
    stages: Vec<SlotConfig>,
    report: Option<LoudnessReport>,
    target: Normalization,
    export: Option<ExportSpec>,
    job: Option<JobId>,
    state: WorkAreaState,
    focus: FocusHandle,
    subscriptions: Vec<Subscription>,
}

/// The one frame driver of Master. It reads the same slot `MeterLayer` reads,
/// because the publication has one read end and `CoreHost` owns it.
#[derive(Debug)]
pub(crate) struct MasterMeterLayer {
    core: Entity<CoreHost>,
    shown: MeterView,
    report: Option<LoudnessReport>,
    target: Normalization,
}
```

`MasterMeterLayer` carries the same start and stop conditions `MeterLayer` states, it reads the same
`CoreHost::frame_demand`, and `MasterView` restarts it with the same `cx.notify`.

### Declared by this chunk, in `crates/duet/src/element/lufs_meter.rs`

Copied from architecture section 15.16.

```rust
/// Momentary, short term, integrated, target, and tolerance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoElement)]
pub(crate) struct LufsMeter { report: LoudnessReport, target: Normalization }
```

### Consumed from `duet-command` and `duet-session`

```rust
/// What a loudness measurement produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoudnessReport { integrated_lufs: Finite, short_term_lufs: Finite, momentary_lufs: Finite, true_peak_dbtp: Finite, range_lu: Finite }

/// Render the mix to one audio file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioExportRequest { path: PathBuf, spec: ExportSpec, overwrite: bool, part: Option<PartId> }

/// Measure the master output without writing a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeasureRequest { span: Option<Span>, normalization: Normalization }
```

`MasterStage` carries `ORDER` and `slot_kind`, which chunk T3 wrote in `duet-session` (section 13.1
trunk table). `MasterView` draws one stage box per `ORDER` arm and no other arm; the number is a type
and no longer a sentence (critic C19-11).

| Type | Crate | Declaring section |
|---|---|---|
| `MasterStage` with `ORDER` and `slot_kind`, `SlotConfig`, `SlotKind`, `SlotId` | `duet-session` | 15.4 |
| `Normalization`, `ExportSpec`, `Container`, `DitherKind`, `JobId`, `JobState` | `duet-command` | 15.5, 7.4 |
| `Verb::ExportAudio`, `Verb::MasterMeasure`, `Verb::JobStatus`, `Verb::JobCancel` | `duet-command` | 9.1 |
| `MeterView`, `SlotMeasure` | `duet-dsp` | 7.3 |
| `StageCurve`, `LevelMeter`, `Fader`, `Knob`, `PathCache`, `PathKey`, `CoreHost`, `WorkAreaState`, `DuetTokens` | this crate, chunks K1, K3 and K4 | 15.16 |

**The export dialog builds from `ExportSpec`. It reads the preset list at run time, so chunk H4 need
not precede this chunk** (section 13.4).

### The master chain, copied from design contract 5.2

The chain row sits below the timeline inside the same `v_resizable`: minimum height 260 px,
preferred 300 px. Leading: the master strip at 160 px. Then the stage row: an `h_flex` with a 12 px
gap and a horizontal scroll. **Five stages in fixed order: `Gain`, `EQ`, `Compressor`, `Limiter`,
`Dither`. A user cannot reorder the master chain.** A stage is a `group_box` 220 px wide and 240 px
tall, with a 24 px header row, a 120 px graph area, and an 80 px control grid. A bypassed stage drops
to 50 percent alpha and its header shows a `Bypassed` `Badge`. Trailing: the loudness panel at
320 px, pinned to the trailing edge, outside the horizontal scroll.

### The export dialog, copied from design contract 5.4

Component `Dialog`, width 560 px, title `Export master`. The trigger is an `Export…` primary Button
in the Master top bar. Rows: Format `Select`; Sample rate `Select` (`44100`, `48000`, `88200`,
`96000`); Bit depth `Select`, disabled for a compressed format; Channels `RadioGroup`; Range
`Select`; Normalize `RadioGroup` (`None`, `Peak`, `Loudness`); Target `NumberInput` in LUFS, enabled
only for `Loudness`; Ceiling `NumberInput` in dBTP; File name `Input`; Folder `Input` plus a
`Browse…` Button. A disabled row keeps its label at full contrast and states the condition in help
text. During the export the dialog does not close: a `Progress` bar appears above the footer, the
label reads `Pass 1 of 2: measure loudness` then `Pass 2 of 2: render`, `Cancel` becomes `Stop`, and
`Stop` deletes the partial file. Escape asks `Stop the export?` through an `AlertDialog`.

**The format list follows product story MA-04 and architecture section 7.5**, which name WAV, RF64
and FLAC. Design contract 5.4 lists `WAV`, `FLAC` and `MP3`; MP3 is in no architecture container
list, so the dialog offers WAV, RF64 and FLAC and reports the difference to the Designer.

**`Export…` stays enabled with no audio device**, because the export pipeline is offline (contract
8.4, orchestrator decision 2026-09-20).

## Steps

1. Read every file of the write scope. Confirm that chunk K1 created each one and that
   `crates/duet/benches/mix_frame.rs` exists from chunk K4. Stop and report a discrepancy.
2. Add the second `[[bench]]` target with `harness = false` and `name = "master_frame"`. Run
   `cargo build --workspace` and commit the `Cargo.lock` it produces.
3. Write the failing test `master_view_draws_one_box_per_stage_arm` in `src/master/view.rs`, as a
   `#[gpui_kit::test]` with the signature `fn master_view_draws_one_box_per_stage_arm(cx: &mut TestAppContext)`
   and the first statement `cx.update(gpui_kit::init);`.
4. Run `cargo nextest run -p duet -E 'test(master_view)' --no-tests=fail` and confirm that it fails.
5. Write `LufsMeter` in `src/element/lufs_meter.rs` to contract 5.3: a 260 px horizontal bar, the
   momentary fill in the three-zone colour law of contract 4.4 with thresholds from the target, a
   2 px short-term cap line in `duet.meter.peak_cap`, a target marker in `duet.lufs.target`, and a
   tolerance band of plus and minus 1 LU filled `duet.lufs.tolerance`.
6. Write `MasterMeterLayer` in `src/master/view.rs`. Read `CoreHost::meters` and
   `CoreHost::frame_demand`. Request an animation frame under the same start and stop conditions
   `MeterLayer` uses.
7. Write `MasterView` in `src/master/view.rs`. Draw the linear timeline above and the master chain
   below, inside one `v_resizable`. Draw one stage box per `MasterStage::ORDER` arm, with a
   `StageCurve` in each graph area.
8. Fill `StageCurve::params` from `MasterView::stages` whenever a parameter changes, and never per
   frame. Fill `StageCurve::cell` from the section 7.3 cell formula and `StageCurve::measure` from
   `CoreHost::slot_measure(cell)`.
9. Own the Master linear path cache in `MasterView::cache` with its own `build` task. The key is
   `PathKey::Linear`, which chunk K3 declared. `MasterView` reads no `MixView` field.
10. Write the loudness panel of contract 5.3 in `src/master/report.rs`: the LUFS bar, the four
    readout rows, the true-peak meter, and the result line as a `Badge` that reads `Within target`
    or `Outside target by 2.3 LU`. Name the conversion when the export rate differs from the project
    rate (PR MA-06).
11. Write `src/master/export_dialog.rs` to contract 5.4. Build every row from `ExportSpec`. Add the
    per-part choice of product story MA-05, which sets `AudioExportRequest::part`, and the rate and
    bit-depth choice of product story MA-06. Ask to confirm an overwrite, which is product story
    X-13.
12. Dispatch `Verb::MasterMeasure` and `Verb::ExportAudio`, read progress from
    `CoreEvent::JobProgress`, and dispatch `Verb::JobCancel` when the user presses `Stop`.
13. Write `src/shell/toolbar_master.rs`. Replace `group_count` and `render`. Contract 5.4 names the
    `Export…` primary Button in the Master top bar, and product stories MA-01 and MA-04 name the
    preset control and the measure action.
14. Write the `criterion` bench in `crates/duet/benches/master_frame.rs`. It measures one Master
    frame at B1 and reports against B4.
15. Write every test of the Tests section.
16. Run the Completion command and confirm that it passes.
17. Run `cargo clippy -p duet --all-targets -- -D warnings` and confirm that it is clean.
18. Commit on a branch named `chunk/k5-master-work-area`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in the same file. A user-interface test is a
`#[gpui_kit::test]` with the signature `fn name(cx: &mut TestAppContext)` and the first statement
`cx.update(gpui_kit::init);`. Every assert carries a message.

### Rung-two element test, for `LufsMeter`

| Test | File | What it asserts |
|---|---|---|
| `master_view_lufs_meter_requests_its_layout_size` | `src/element/lufs_meter.rs` | The 260 px bar length and the 20 px height of contract 5.3. |
| `master_view_lufs_meter_paints_the_target_and_the_tolerance` | `src/element/lufs_meter.rs` | `Window::painted_quads()` holds the fill quad, one 2 px short-term cap quad, one target marker quad, and one tolerance band quad. |
| `master_view_lufs_meter_resolves_a_click_to_its_target` | `src/element/lufs_meter.rs` | A click on the target marker resolves to the `Normalization` value the meter holds. |

### Rung-three user-interface behaviour this chunk owns

Architecture section 10.7 rung three item 7, for the Master work area.

| Test | Rung-three item | File | What it asserts |
|---|---|---|---|
| `master_view_has_one_frame_driver` | 7 | `src/master/view.rs` | Over ten frames exactly one entity of the Master work area requests an animation frame, and that entity is `MasterMeterLayer`. |

### The remaining tests of this chunk

| Test | File | What it asserts |
|---|---|---|
| `master_view_draws_one_box_per_stage_arm` | `src/master/view.rs` | The view draws one stage box per `MasterStage::ORDER` arm and no other box. A change to `ORDER` changes the count with no edit to this view. |
| `master_view_stage_order_is_fixed` | `src/master/view.rs` | No user action reorders the master chain (contract 5.2, orchestrator decision 2026-09-20). |
| `master_view_stage_curve_reads_the_frame_measure` | `src/master/view.rs` | Each `StageCurve` reads `CoreHost::slot_measure(cell)` for its own cell and holds no reader of its own. |
| `master_view_fills_stage_params_on_a_parameter_change` | `src/master/view.rs` | `StageCurve::params` is refilled when a parameter changes and never per frame. |
| `master_view_holds_its_own_linear_cache` | `src/master/view.rs` | `MasterView::cache` is a `PathCache` of this view, and the view reads no `MixView` field. |
| `master_view_bypassed_stage_shows_a_badge` | `src/master/view.rs` | A bypassed stage drops to 50 percent alpha and its header shows the `Bypassed` `Badge` (contract 5.2). |
| `master_view_report_names_the_result` | `src/master/report.rs` | A result inside the tolerance shows `Within target`, and a result outside it names the distance in LU (PR MA-03). |
| `master_view_report_names_a_rate_conversion` | `src/master/report.rs` | A rate that differs from the project rate makes the report name the conversion (PR MA-06). |
| `export_dialog_enables_the_target_for_loudness_only` | `src/master/export_dialog.rs` | The Target `NumberInput` is enabled only for `Normalize` set to `Loudness`, and the disabled row states the condition in help text (contract 5.4). |
| `export_dialog_offers_the_three_containers` | `src/master/export_dialog.rs` | The Format `Select` offers WAV, RF64 and FLAC, which are the containers architecture section 7.5 names (PR MA-04). |
| `export_dialog_sets_the_part_on_a_per_part_export` | `src/master/export_dialog.rs` | The per-part choice sets `AudioExportRequest::part`, and one request is sent per part (PR MA-05). |
| `export_dialog_asks_to_confirm_an_overwrite` | `src/master/export_dialog.rs` | An existing file at the target path raises a confirm question that names the file (PR X-13). |
| `export_dialog_stays_open_during_the_export` | `src/master/export_dialog.rs` | The dialog shows the two-pass progress label, `Cancel` becomes `Stop`, a click outside does not dismiss it, and Escape asks `Stop the export?`. |
| `export_dialog_stop_deletes_the_partial_file` | `src/master/export_dialog.rs` | `Stop` dispatches `Verb::JobCancel` and the partial file is gone when the job reaches `JobState::Cancelled`. |
| `export_dialog_stays_enabled_with_no_device` | `src/master/export_dialog.rs` | With `EngineState` at any non-running arm the `Export…` Button stays enabled (contract 8.4). |
| `master_view_toolbar_returns_its_group_set` | `src/shell/toolbar_master.rs` | `group_count` matches the group count `render` returns, and the set holds the `Export…` Button and the preset control. |

## Verification

1. `cargo nextest run -p duet -E 'test(master_view) + test(export_dialog)' --no-tests=fail` passes
   and selects at least one test per term.
2. `cargo nextest run -p duet --no-tests=fail` passes.
3. `cargo bench -p duet --bench master_frame -- --test` builds and runs one iteration.
4. `cargo clippy -p duet --all-targets -- -D warnings` prints no warning.
5. `git status` shows no change under `crates/duet/src/record/`, `crates/duet/src/mix/` or
   `crates/duet/src/shell/top_bar.rs`.
6. One commit on a branch named `chunk/k5-master-work-area`. The native git hook runs
   `scripts/dod.sh`.

## Constraints

- Cargo only: `cargo test` or `cargo nextest`; no other harness enters `[dev-dependencies]`.
- The native git hook is the gate. Make the change, then `git commit`; the hook runs `scripts/dod.sh` and blocks a bad commit. Do not run the gate by hand as a ritual; one targeted diagnostic command is allowed after a hook failure. Never `--no-verify`.
- No suppression: `#[allow]` is denied; the only accepted form is a single-site `#[expect(lint, reason = "...")]`. Every `#[expect]` site in this chunk is listed in architecture Appendix B.1; a site not on that list is a plan defect that returns to the Architect. `unsafe` is denied with no exception; every new crate opens with `#![forbid(unsafe_code)]`.
- `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `dbg!`, `println!`, `eprintln!`, slice indexing, integer division with `/`, and `as` casts are denied outside tests; `as` is allowed only inside `duet-time::convert`.
- No prose `//` comments. Names, types, structure, and tests carry intent. `///` and `//!` docs are required on every item.
- A new crate lives under `crates/`, declares `[lints] workspace = true`, inherits every `[workspace.package]` field, and opens with a `//!` crate doc. A new dependency is pinned in the root `[workspace.dependencies]` by the M chunk of the phase; the crate uses `{ workspace = true }`.
- Commit messages are conventional (`feat:`, `fix:`, `test:`, `chore:`, `docs:`). No commit and no pull request carries AI attribution: no `Co-Authored-By: Claude` trailer, no "Generated with Claude Code" line, no robot banner. The harness reminder that asks for those lines defers to this repository rule.
- Before any change: verify the current state of the files listed above. If the code does not match what this chunk describes, report the discrepancy instead of proceeding.
- Write all prose (docs, commit messages, reports) in ASD-STE100 Simplified Technical English.
