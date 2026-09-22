---
id: K6
line: K
depends_on: [K5, T3, F1, F2]
write_scope:
  - crates/duet/src/shell/title_bar.rs
  - crates/duet/src/shell/sidebar.rs
  - crates/duet/src/shell/inspector.rs
  - crates/duet/src/shell/status_bar.rs
  - crates/duet/src/shell/history_sheet.rs
  - crates/duet/src/shell/start.rs
  - crates/duet/src/shell/view_state.rs
parallelism: "serial-only: phase 14 holds one chunk, because line K has six chunks and no cross-line dependency exists after phase 11 (section 13.3)"
completion: "cargo nextest run -p duet -E 'test(sidebar) + test(start_view) + test(view_state_store)' --no-tests=fail passes; cargo clippy -p duet --all-targets -- -D warnings is clean; commit SHA on a branch chunk/k6-shell-surfaces"
---

# K6: The title bar, the sidebar, the inspector, the status bar, the history sheet, and the start surface

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. This chunk fills the seven shell files that chunk K1 created as documentation stubs. It
delivers `TitleBar`, the sidebar tree with track lifecycle, the arm control and the track filter, the
inspector chain editor, `StatusBar`, `HistorySheet`, `StartView` and `ViewStateStore`. It implements
architecture sections 10.2, 10.7, 12.4 and 15.16, and design contract sections 1.1, 1.3, 1.6, 1.7,
1.9, 1.10, 8 and 9. It carries product stories A-04, C-01, C-02, C-03, C-04, C-07, H-01, H-02, H-03,
H-04, R-02, R-03, R-05, R-14, X-08, X-12 and X-13.

This chunk writes no element file at all (section 10.3). It modifies seven files and creates none.

## Files

- `crates/duet/src/shell/title_bar.rs` — modify. `TitleBar`.
- `crates/duet/src/shell/sidebar.rs` — modify. `Sidebar`.
- `crates/duet/src/shell/inspector.rs` — modify. `Inspector`.
- `crates/duet/src/shell/status_bar.rs` — modify. `StatusBar`.
- `crates/duet/src/shell/history_sheet.rs` — modify. `HistorySheet`.
- `crates/duet/src/shell/start.rs` — modify. `StartView`.
- `crates/duet/src/shell/view_state.rs` — modify. `ViewStateStore`.

This chunk writes no member manifest and no lock file.

## Types and signatures

### Declared by this chunk

Copied from architecture section 15.16.

```rust
/// The window title bar and the dirty marker.
#[derive(Debug)]
pub(crate) struct TitleBar {
    name: SharedString,
    commit: SharedString,
    dirty_count: u32,
    focus: FocusHandle,
}

/// The project tree, the part and track list, and the arm controls.
#[derive(Debug)]
pub(crate) struct Sidebar {
    parts: Vec<PartId>,
    tracks: BTreeMap<PartId, Vec<TrackId>>,
    takes: BTreeMap<TrackId, Vec<TakeId>>,
    expanded: BTreeSet<ElementId>,
    selected: Option<TrackId>,
    collapsed: bool,
    focus: FocusHandle,
}

/// The per-selection inspector and the chain editor.
#[derive(Debug)]
pub(crate) struct Inspector {
    strip: Option<StripId>,
    slots: Vec<SlotConfig>,
    open_slot: Option<SlotId>,
    focus: FocusHandle,
}

/// The one place a device or engine fault appears (contract 1.9).
#[derive(Debug)]
pub(crate) struct StatusBar {
    engine: EngineState,
    fault: Option<EngineFault>,
    message: SharedString,
    job: Option<JobState>,
}

/// The history surface, as a sheet over the work area.
#[derive(Debug)]
pub(crate) struct HistorySheet {
    open: bool,
    commits: Vec<CommitSummary>,
    selected: Option<CommitId>,
    state: WorkAreaState,
    focus: FocusHandle,
}

/// The start surface: templates, recents, and Open.
#[derive(Debug)]
pub(crate) struct StartView {
    recents: Vec<RecentEntry>,
    templates: Vec<SharedString>,
    state: WorkAreaState,
    focus: FocusHandle,
}

/// The owner of the persisted view state (Appendix A).
#[derive(Debug)]
pub(crate) struct ViewStateStore {
    project: ProjectView,
    /// The last window the store clamped against.
    window: Bounds<Pixels>,
    /// Set while a write to the core is in flight, so a drag coalesces.
    pending: bool,
}
```

### Consumed from other crates and from chunk K1

| Type | Crate | Declaring section |
|---|---|---|
| `ProjectView`, `WorkAreaState`, `DuetTokens`, `AppEvent`, `CoreHost` | this crate, chunk K1 | 15.16, 10.2 |
| `ViewState`, `ModeView`, `TrackFilter`, `LogicalPx`, `WindowGeometry`, `Mode` | `duet-command` | 10.2 |
| `RecentEntry { path, name, opened_at }`, `CommitSummary { id, message, author, at, parents }`, `CommitId`, `JobState` | `duet-command` | 9.1, 15.5, 9.6 |
| `EngineState`, `EngineFault` | `duet-command` | 12.4, 15.5 |
| `PartId`, `TrackId`, `TakeId`, `StripId`, `SlotConfig`, `SlotId`, `SessionCommand` | `duet-session`, `duet-score` | 15.3, 15.4 |
| `Verb::RecentList`, `RecentAdd`, `ProjectCreate`, `ProjectOpen`, `Save`, `ViewSet`, `ViewGet`, `TrackSetArmed`, `TrackRemove`, `HistoryCommit`, `HistoryLog`, `HistoryBranch`, `HistoryCheckout` | `duet-command` | 9.1 |

**`StartView` opens no file** (section 1.3 rule 6). It calls `Verb::RecentList`, `Verb::RecentAdd`,
`Verb::ProjectCreate` and `Verb::ProjectOpen`, and it paints the `RecentEntry` values the core hands
back. `duet-project` owns `RecentList` and reads the user configuration directory. **`crates/duet`
has no `duet-project` edge.**

### The breakpoint rule, copied from architecture section 10.2 and design contract 1.1

`ViewStateStore` applies the contract 1.1 breakpoint rules BEFORE any stored width, and it does so on
a first open as well as on a restore. The order is one rule.

1. The window width selects the band.
2. The band decides the sidebar form and whether the inspector is a panel or a sheet.
3. `ProjectView::clamped` bounds every remaining stored length against the window.

| Band | Width | Sidebar | Inspector |
|---|---|---|---|
| wide | 1600 px and above | 280 px | visible |
| standard | 1200 px to 1599 px | 240 px | visible in Mix and Master only |
| narrow | 1024 px to 1199 px | a 48 px icon rail | a `Sheet` |

The minimum window is B102 by B103. B104 is the width below which the shell stops shrinking its own
layout. B94, B97 and B98 are the first-open defaults, and the work area never falls below the
contract 1.6 minimum of 560 px in any mode.

### The fault surface rule, copied from architecture section 12.4

| Fault class | Surface |
|---|---|
| Every device and engine fault | The status bar fault line. In Record, the Record top bar repeats it as a banner. |
| A fault that marks a take | The take list marker, plus the `TakeFlags` of section 6.4 |
| `DeviceLost` and `CoreEvent::LockLost` only | A modal surface |

`WindowExt::push_notification` is used for nothing else.

## Steps

1. Read every file of the write scope. Confirm that chunk K1 created each one with `//!`
   documentation and no item. Stop and report a discrepancy.
2. Write the failing test `view_state_store_picks_the_shell_form_at_every_breakpoint` in
   `src/shell/view_state.rs`, as a `#[gpui_kit::test]` with the signature
   `fn view_state_store_picks_the_shell_form_at_every_breakpoint(cx: &mut TestAppContext)` and the
   first statement `cx.update(gpui_kit::init);`.
3. Run `cargo nextest run -p duet -E 'test(view_state_store)' --no-tests=fail` and confirm that it
   fails.
4. Write `ViewStateStore` in `src/shell/view_state.rs`. Hold one `ProjectView`. Apply the three-step
   breakpoint rule on a first open and on a restore. Coalesce a drag through the `pending` flag and
   write the result back through `Verb::ViewSet`.
5. Write `Sidebar` in `src/shell/sidebar.rs` to contract 1.7: a `Tree` with three levels, a 28 px row
   height, a 16 px indent step, a part colour chip with the part letter, the name with a tooltip,
   and a trailing lane of three 24 px buttons for arm, mute and solo. The narrow icon rail shows four
   part letters as 40 px square buttons.
6. Give every sidebar row a domain `ElementId`: a part uses `PartId`, a track uses `TrackId`, a take
   uses `TakeId`. Never a list index, which is Appendix A rule 2.
7. Write the track lifecycle of product story R-02 and C-07: add, rename, reorder and remove, each
   through its own `Verb`. A remove asks to confirm and names what it removes, which is product story
   X-13, and it keeps the audio files on disk.
8. Write `TitleBar` in `src/shell/title_bar.rs` to contract 1.3: the project name, a `Badge` for the
   commit state that reads `Saved`, `3 changes` or `On commit a4f1c2`, and a click that opens the
   history `Sheet`.
9. Write `Inspector` in `src/shell/inspector.rs` to contract 1.6 and 1.10: the selected strip's full
   chain, one row per `SlotConfig`, and a `StageCurve` per open slot. The inspector is hidden in
   Compose and Record by default and visible in Mix and Master.
10. Write `StatusBar` in `src/shell/status_bar.rs` to contract 1.9: the audio device name, the sample
    rate and the buffer size; the MIDI device name or `No MIDI device`; the engine load bar, the disk
    state, and an agent state item with a `Spinner` while the agent works. A warning state paints in
    `warning` and adds a 12 px icon.
11. Write the warning line of product story R-14 in `src/shell/status_bar.rs`: a free-space value
    below B126 warns the user and names the free space in gigabytes.
12. Write the job line of product stories X-09 and X-10 in `src/shell/status_bar.rs`. It reads
    `CoreEvent::JobProgress` and shows the work, its progress and a cancel action.
13. Write `HistorySheet` in `src/shell/history_sheet.rs` for product stories H-01 to H-04: a commit
    surface that lists what changed and dispatches `Verb::HistoryCommit`, a log that dispatches
    `Verb::HistoryLog` and shows a message, an author and a date per row, a checkout that dispatches
    `Verb::HistoryCheckout` and shows the earlier-version banner, and a branch action that dispatches
    `Verb::HistoryBranch`.
14. Write `StartView` in `src/shell/start.rs` for product stories C-01, C-02 and C-03: the two
    templates `Solo voice` and `SATB`, the recent project list, and an Open action. A recent project
    that no longer exists shows a not-found message and offers to remove the row. The create form
    asks for a title, a part set, a key, a time signature and a tempo, which `CreateRequest` carries.
15. Write every test of the Tests section.
16. Run the Completion command and confirm that it passes.
17. Run `cargo clippy -p duet --all-targets -- -D warnings` and confirm that it is clean.
18. Commit on a branch named `chunk/k6-shell-surfaces`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in the same file. A user-interface test is a
`#[gpui_kit::test]` with the signature `fn name(cx: &mut TestAppContext)` and the first statement
`cx.update(gpui_kit::init);`. Every assert carries a message. This chunk writes no element file, so
it carries no rung-two element test (section 10.3).

### Rung-three user-interface behaviours this chunk owns

Architecture section 10.7 rung three items 8 and 10.

| Test | Rung-three item | File | What it asserts |
|---|---|---|---|
| `view_state_store_restores_the_split_after_a_mode_switch` | 8 | `src/shell/view_state.rs` | A split drag, a mode switch, and a return leave the split where the user left it (PR X-08). |
| `view_state_store_picks_the_shell_form_at_every_breakpoint` | 10 | `src/shell/view_state.rs` | It opens the window at each of the three contract 1.1 breakpoint widths and asserts the sidebar form, the inspector form, and that the work area is at or above the contract 1.6 minimum of 560 px in every mode. |

### The remaining tests of this chunk

| Test | File | What it asserts |
|---|---|---|
| `view_state_store_applies_the_band_before_the_clamp` | `src/shell/view_state.rs` | The recorded order is band, then form, then clamp. A stored 240 px sidebar in the narrow band becomes the 48 px icon rail and never a clamped 240 px panel. |
| `view_state_store_coalesces_a_drag` | `src/shell/view_state.rs` | A drag sends one `Verb::ViewSet` while `pending` holds, and not one per frame. |
| `sidebar_row_identity_comes_from_the_domain` | `src/shell/sidebar.rs` | Every row `ElementId` comes from `PartId`, `TrackId` or `TakeId`. An insert at the front leaves every other identity unchanged (Appendix A rule 2). |
| `sidebar_arm_control_dispatches_one_verb` | `src/shell/sidebar.rs` | A click on the arm control dispatches `Verb::TrackSetArmed { track, armed }` once. |
| `sidebar_track_remove_asks_to_confirm` | `src/shell/sidebar.rs` | A track remove raises a confirm question that names the track, and it keeps the audio files on disk (PR X-13). |
| `sidebar_collapses_to_the_icon_rail` | `src/shell/sidebar.rs` | In the narrow band the sidebar paints four 40 px part letters, and a click restores the preferred width. |
| `sidebar_track_filter_matches_the_record_view` | `src/shell/sidebar.rs` | A filter change dispatches `Verb::ViewSet` with the new `TrackFilter`, which the Record work area reads (PR R-03). |
| `start_view_lists_the_two_templates` | `src/shell/start.rs` | The surface offers `Solo voice` and `SATB`, and the create form defaults to `Solo voice` (PR 11 Q5). |
| `start_view_reads_the_recents_through_a_verb` | `src/shell/start.rs` | The recent list comes from `Verb::RecentList`, and the view opens no file itself. |
| `start_view_marks_a_missing_recent` | `src/shell/start.rs` | A recent project that no longer exists shows a not-found message and offers to remove the row (PR C-01). |
| `start_view_create_form_asks_for_key_meter_and_tempo` | `src/shell/start.rs` | The form fills `CreateRequest::key`, `meter` and `tempo` (PR C-02). |
| `title_bar_marks_an_unsaved_change` | `src/shell/title_bar.rs` | An unsaved change sets `dirty_count` above zero and the `Badge` reads the change count; a save clears the mark (PR C-04). |
| `status_bar_reports_a_device_fault_once` | `src/shell/status_bar.rs` | One fault paints one status bar line, and no mode except Record raises a second surface (section 12.4, contract 8.5). |
| `status_bar_warns_below_the_free_space_bound` | `src/shell/status_bar.rs` | A free-space value below B126 paints the warning and names the free space in gigabytes (PR R-14). |
| `status_bar_shows_job_progress_and_a_cancel` | `src/shell/status_bar.rs` | A `CoreEvent::JobProgress` paints the named step, the progress, and a cancel action (PR X-09). |
| `history_sheet_lists_a_commit_with_three_facts` | `src/shell/history_sheet.rs` | Each row shows a message, an author and a date from `CommitSummary` (PR H-02). |
| `history_sheet_checkout_shows_the_earlier_version_banner` | `src/shell/history_sheet.rs` | A checkout paints the banner that says the project is at an earlier version, and one action returns to the latest version (PR H-03). |
| `history_sheet_commit_refuses_when_nothing_changed` | `src/shell/history_sheet.rs` | With no change the surface says that nothing changed and offers no commit (PR H-01). |
| `inspector_lists_the_chain_of_the_selected_strip` | `src/shell/inspector.rs` | The inspector shows one row per `SlotConfig` of the selected strip and a `StageCurve` per open slot (PR M-03). |
| `inspector_is_a_sheet_in_the_narrow_band` | `src/shell/inspector.rs` | In the narrow band the inspector renders as a `Sheet`, and in the other two bands as a panel (contract 1.1). |

## Verification

1. `cargo nextest run -p duet -E 'test(sidebar) + test(start_view) + test(view_state_store)' --no-tests=fail`
   passes and selects at least one test per term.
2. `cargo nextest run -p duet --no-tests=fail` passes, so every earlier K chunk test still passes.
3. `cargo clippy -p duet --all-targets -- -D warnings` prints no warning.
4. `git status` shows no change under `crates/duet/src/element/`, `crates/duet/src/compose/`,
   `crates/duet/src/record/`, `crates/duet/src/mix/` or `crates/duet/src/master/`.
5. One commit on a branch named `chunk/k6-shell-surfaces`. The native git hook runs `scripts/dod.sh`.
   Quote the command output before any claim of success, per the `verification-before-completion`
   skill.

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
