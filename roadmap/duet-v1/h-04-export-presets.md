---
id: H4
line: H
depends_on: [H3]
write_scope:
  - crates/duet-export/src/preset.rs
parallelism: independent
completion: "cargo nextest run -p duet-export -E 'test(preset)' --no-tests=fail passes; commit SHA on a branch chunk/h4-export-presets"
---

# H4: The Streaming, Broadcast, and Custom presets

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of proceeding. This chunk builds the preset set of architecture section 7.4: Streaming, Broadcast, and Custom, with the two numbers B75 gives each of the first two. It carries MUST story MA-01 (loudness target and true-peak ceiling). The export dialog of chunk K5 reads the preset list at run time, so link `H1 before K5` of section 13.4 states that H4 need not precede K5. Chunk J3 does need this chunk: link `I4 and H4 before J3` states that `export_end_to_end` exports with the Streaming preset.

**This chunk writes no member manifest**, so it names neither `crates/duet-export/Cargo.toml` nor `Cargo.lock` in its write scope, and its Completion command carries no `--locked` flag (SM3 rule 4, SM5). It is the last chunk of line H.

## Files

- `crates/duet-export/src/preset.rs` — modify. Chunk H1 created the stub.

## Types and signatures

### The two numbers each preset carries (architecture 1.6, B75)

| Preset | Integrated loudness | True-peak ceiling |
|---|---|---|
| Streaming | -14 LUFS | -1 dBTP |
| Broadcast | -23 LUFS | -1 dBTP |
| Custom | The user's value | The user's value |

B76 is the loudness tolerance the end-to-end test asserts, at 0.2 LU.

### `duet_export::preset` (architecture 7.4)

A preset is a named `Normalization` value. The type `Normalization` is a `duet-command` type, and the preset names it:

```rust
// in duet-command
/// How the render sets the level. Every level is a `Finite`, so a hand
/// edited file cannot carry a NaN into the limiter (VR4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Normalization {
    None,
    Peak { dbfs: Finite },
    Loudness { lufs: Finite, true_peak_dbtp: Finite },
}
```

The preset set this chunk declares:

```rust
/// One named loudness target the export dialog offers.
///
/// **The list is read at run time**, which is why chunk K5 needs no
/// dependency on this chunk (section 13.4). `Custom` takes both numbers from
/// the user through `Finite::new`, so the form rejects a non-finite entry
/// before the verb is built (section 7.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportPreset {
    /// -14 LUFS and -1 dBTP (B75).
    Streaming,
    /// -23 LUFS and -1 dBTP (B75).
    Broadcast,
    /// The two numbers the user entered.
    Custom { lufs: Finite, true_peak_dbtp: Finite },
}

impl ExportPreset {
    /// The `Normalization` this preset means.
    ///
    /// `Streaming` and `Broadcast` each read their two numbers from the
    /// workspace constants B75 declares, so no literal sits at this site
    /// (DR3).
    #[must_use]
    pub fn normalization(self) -> Normalization;

    /// Every preset the export dialog offers, in the order it shows them.
    ///
    /// `Custom` is not in the list, because the dialog builds it from the two
    /// values the user entered.
    pub const ORDER: [Self; 2] = [Self::Streaming, Self::Broadcast];

    /// The name the user reads.
    #[must_use]
    pub const fn label(self) -> &'static str;
}
```

**No literal sits in this module.** B75 lives in the workspace constant block that section 1.6 declares, and `normalization` reads it. A number this document chooses has one home.

`clippy::wildcard_enum_match_arm` is denied, so a new preset is a compile error at every match site, which is what the closed set gives.

### Types this chunk consumes

| Type | Crate | Path |
|---|---|---|
| `Normalization`, `ExportSpec` | `duet-command` | `duet_command` |
| `Finite` | `duet-time` | `duet_time` |
| The B75 constants | `duet-time` | `duet_time` |
| The two passes that read a `Normalization` | `duet-export` | chunk H2 |

## Steps

1. Read `crates/duet-export/src/preset.rs`. Confirm that chunk H1 left it a stub with a `//!` line and nothing else, and that chunk H3 has landed. Report a discrepancy and stop.
2. Confirm that the workspace constant block holds the B75 values, which chunk M0 or a trunk chunk wrote in `duet-time`. When it does not, report the discrepancy and stop; do not write a literal in this module.
3. Write the failing test `preset_streaming_targets_minus_fourteen_lufs` in `src/preset.rs`. Run `cargo nextest run -p duet-export -E 'test(preset)' --no-tests=fail` and confirm that it fails to compile.
4. Write `ExportPreset`, `ExportPreset::normalization`, `ExportPreset::ORDER` and `ExportPreset::label` in `src/preset.rs`, with the declarations above. `normalization` reads the B75 constants and writes no literal. Run the test and confirm that it passes.
5. Write the failing test `preset_broadcast_targets_minus_twenty_three_lufs`. Run it and confirm that it fails, then confirm that it passes.
6. Write the failing test `preset_custom_carries_the_user_values`. Run it and confirm that it fails, then confirm that `ExportPreset::Custom` carries the two `Finite` values it was built with.
7. Write the failing test `preset_order_holds_the_two_named_presets`. Run it and confirm that it fails, then confirm that `ORDER` holds Streaming and Broadcast in that order and that `Custom` is not in it.
8. Write the failing test `preset_drives_the_two_pass_render_inside_b76`. Run it and confirm that it fails, then confirm that a render with the Streaming preset measures -14 LUFS inside the B76 tolerance and that no sample passes -1 dBTP.
9. Confirm that `src/preset.rs` holds no numeric literal for a loudness value or a true-peak value, and that every such value reads a workspace constant.
10. Run `cargo clippy -p duet-export --all-targets -- -D warnings`. Fix every finding in the code.
11. Commit on the branch `chunk/h4-export-presets`. The native git hook runs `scripts/dod.sh`.

## Tests

All tests of this chunk are unit tests in a `#[cfg(test)] mod tests` at the bottom of `src/preset.rs`. Each test that renders creates its own scratch directory under `std::env::temp_dir()` with the process identifier and a nanosecond suffix, and removes it at the end. No test needs a device, because the dummy backend serves every export test.

| Test | File | What it asserts |
|---|---|---|
| `preset_streaming_targets_minus_fourteen_lufs` | `src/preset.rs` | `ExportPreset::Streaming.normalization()` is `Normalization::Loudness` with -14 LUFS and -1 dBTP. Message: "the Streaming preset targets -14 LUFS and -1 dBTP". |
| `preset_broadcast_targets_minus_twenty_three_lufs` | `src/preset.rs` | `ExportPreset::Broadcast.normalization()` is `Normalization::Loudness` with -23 LUFS and -1 dBTP. Message: "the Broadcast preset targets -23 LUFS and -1 dBTP". |
| `preset_custom_carries_the_user_values` | `src/preset.rs` | `ExportPreset::Custom { lufs, true_peak_dbtp }` answers a `Normalization::Loudness` with the same two values. Message: "the Custom preset carries the values the user entered". |
| `preset_order_holds_the_two_named_presets` | `src/preset.rs` | `ORDER` holds Streaming then Broadcast, and it holds no `Custom`. Message: "the preset order holds the two named presets". |
| `preset_label_names_each_preset` | `src/preset.rs` | A table-driven loop over `ORDER` asserts a non-empty label for each preset. The message names the case: `"preset {preset:?}"`. |
| `preset_drives_the_two_pass_render_inside_b76` | `src/preset.rs` | A render of a known signal with the Streaming preset measures -14 LUFS inside the B76 tolerance of 0.2 LU, and no output sample measures above -1 dBTP. Message: "the Streaming preset reaches its target inside B76". |
| `preset_holds_no_numeric_literal` | `src/preset.rs` | `normalization` returns the workspace-constant values, so a change to the B75 constants moves the preset. Message: "the preset reads the B75 constants and holds no literal". |

## Verification

1. `cargo nextest run -p duet-export -E 'test(preset)' --no-tests=fail` passes on macOS and on Linux. It fails before this chunk, because no test of that name exists.
2. `cargo nextest run -p duet-export --no-tests=fail` passes.
3. `cargo clippy -p duet-export --all-targets -- -D warnings` prints nothing.
4. `git status` inside `crates/duet-export` shows no change to `Cargo.toml` and no change to `Cargo.lock`, because this chunk adds no dependency.
5. One commit on the branch `chunk/h4-export-presets` passes the native git hook.

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
