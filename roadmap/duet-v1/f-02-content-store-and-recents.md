---
id: F2
line: F
depends_on: [F1]
write_scope:
  - crates/duet-project/src/store/manifest.rs
  - crates/duet-project/src/store/content.rs
  - crates/duet-project/src/store/gc.rs
  - crates/duet-project/src/recent.rs
  - crates/duet-project/Cargo.toml
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-project -E 'test(content_store) + test(recent_list)' --no-tests=fail passes; commit SHA on a branch chunk/f2-content-store-and-recents"
---

# F2: The content store, the manifest, the reachability walk, and the recent list

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of proceeding. This chunk builds the content-addressed media store of architecture section 4.3, `media.manifest` and `ManifestEntry`, the reachability walk and the four sources of the live set of section 4.4, `gc` and `gc --purge`, the recent-project list of section 15.12, and the macOS case-insensitive store test. ADR 0003 clauses 2, 3, 6, 6a, 6b and 17 decide the store, the manifest, the live set, the explicit purge list and the store bounds. It carries MUST stories C-01 (the recent half of the start surface) and X-13 (destructive actions, the `gc --purge` half).

## Files

- `crates/duet-project/src/store/manifest.rs` — modify. Chunk F1 created the stub.
- `crates/duet-project/src/store/content.rs` — modify.
- `crates/duet-project/src/store/gc.rs` — modify.
- `crates/duet-project/src/recent.rs` — modify.
- `crates/duet-project/Cargo.toml` — modify. Add the `blake3` entry.
- `Cargo.lock` — modify. Commit it in the same commit as the manifest (SM5).

## Types and signatures

### `duet_project::store::manifest` (architecture 4.3, 15.12)

```rust
/// One line of `media.manifest`.
#[expect(
    missing_copy_implementations,
    reason = "one manifest line is the size the PG24 table prints for `ManifestEntry`, and it \
              grows when the manifest gains a column; VR5 refuses `Copy` past the size bound"
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestEntry {
    hash: SourceHash,
    channels: NonZeroU8,
    sample_rate: SampleRate,
    frames: u64,
    bytes: u64,
    container: AudioContainer,
}
```

The Appendix B.1 row for `duet-project::store::manifest::ManifestEntry` carries this exact reason text. It is the one suppression of this chunk.

`media.manifest` is tracked text, sorted by hash. A commit therefore pins exactly which audio bytes it needs. A region in `regions.jsonl` names a `SourceHash`, never a path.

### `duet_project::store::content` (architecture 4.3)

A source is named by the lowercase hexadecimal BLAKE3 hash of its file bytes, printed as 64 lowercase hexadecimal characters. Lowercase hexadecimal avoids a collision on a case-insensitive APFS volume. The store path is `media/<hh>/<hash>.wav`, where `<hh>` is the first two characters of the hash.

`SourceHash` and `AudioContainer` are `duet-session` types:

```rust
// in duet-session
/// A content address for one audio source. It is a BLAKE3 digest, printed as
/// 64 lowercase hexadecimal characters, and the PG24 table prints its size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SourceHash([u8; 32]);

/// The container a captured source is stored in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioContainer { Wav, Rf64 }
```

### `duet_project::store::gc` (architecture 4.4)

**The live set has four sources.**

1. The working `media.manifest` on disk, plus every file in `media/incoming/`.
2. Every commit reachable from any reference: a branch, a tag, the reflog, or a detached `HEAD`.
3. Every `SourceHash` that the undo stack of any open session names.
4. Every `SourceHash` that the redo stack of any open session names.

Sources 3 and 4 live in `DuetCore` memory, so a disk-only walk cannot see them. `Verb::Gc` is a deferred verb: the core builds sources 1, 3 and 4 from memory into a snapshot, and a job thread then walks the commits and scans `media/`.

```rust
// in duet-core, declared by chunk I1; this chunk consumes the result
/// Build the live set. Only the core can call this, because sources 3 and 4
/// are in-memory stacks.
///
/// # Errors
/// Returns `GatewayError::RecordInProgress` during a record pass, and
/// `GatewayError::HistoryUnresolved` while an external `HEAD` change waits
/// for the user.
pub fn live_sources(&self) -> Result<BTreeSet<SourceHash>, GatewayError>;
```

Three refusals protect the user: `gc` refuses while a record pass runs, `gc` refuses while an external `HEAD` change is unresolved, and `gc` refuses when the core is not reachable.

```text
duet gc
```

It moves every file that the live set does not name to `media/dead/`. **It never deletes.**

```text
duet gc --purge <hash> [<hash> ...]
```

`--purge` takes an explicit hash list and never a wildcard. It recomputes the live set from all four sources and refuses the whole call when any named hash is live, with `ProjectError::HashReachable { hash }`. It then deletes only the named files.

### `duet_project::recent` (architecture 15.12)

```rust
/// The recent-project list, read from the user configuration directory.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecentList { entries: Vec<RecentEntry> }
```

**B142 bounds it and `Verb::RecentAdd` drops the OLDEST entry past that count.** The list stays a `Vec` and not an `ArrayVec`, because the file a user or an agent may have edited must PARSE before the reader can trim it; a fixed capacity would make the parse fail where the design wants a repair. `RecentEntry` is a `duet-command` type:

```rust
// in duet-command
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentEntry { path: PathBuf, name: String, opened_at: UnixSeconds }
```

The store bounds of section 4.1 that this chunk enforces:

| Store | Bound | Eviction rule |
|---|---|---|
| `derived/peaks/` (cache) | B59 | The least recently opened file is deleted when the bound is passed. A deleted pyramid rebuilds on demand. |
| `derived/analysis/` (cache) | B60 | The same rule. A pitch track rebuilds from the source. |
| `media/incoming/` | One file per armed track | A file with no live writer is offered for recovery at the next start. |
| `media/dead/` | No automatic bound; it holds user audio | `duet gc --purge` is the only remover. `duet status` reports the size. |

### Types this chunk consumes

| Type | Crate | Path |
|---|---|---|
| `SourceHash`, `AudioContainer` | `duet-session` | `duet_session` |
| `RecentEntry`, `GatewayError`, `CommitId` | `duet-command` | `duet_command` |
| `SampleRate` | `duet-time` | `duet_time` |
| `ProjectError` | `duet-project` | chunk F1 |
| `TreeSnapshot`, `HeadState` | `duet-project` | chunk F3 writes the `History` trait; this chunk takes the reachable commit set as an argument, so it names no `gix` type |

## Steps

1. Read every file of the write scope and `crates/duet-project/Cargo.toml`. Confirm that chunk F1 left each one a stub and that the manifest holds the entries F1 added. Report a discrepancy and stop.
2. Add to `crates/duet-project/Cargo.toml` the entry `blake3 = { workspace = true }`. Run `cargo build --workspace` and keep `Cargo.lock` for the same commit.
3. Write the failing test `content_store_names_every_object_in_lowercase_hexadecimal` in `src/store/content.rs`. Run `cargo nextest run -p duet-project -E 'test(content_store)' --no-tests=fail` and confirm that it fails to compile.
4. Write `src/store/content.rs`: hash a file with BLAKE3, print the digest as 64 lowercase hexadecimal characters, and place the object at `media/<hh>/<hash>.wav`. The store reads and writes nothing outside `media/`. Run the test and confirm that it passes.
5. Write `ManifestEntry` in `src/store/manifest.rs` with the `#[expect]` attribute above it, and the reader and the writer of `media.manifest`. The file is sorted by hash, one entry per line, and it is tracked text.
6. Write the failing test `content_store_is_case_safe_on_a_case_insensitive_volume`, with `#[cfg(target_os = "macos")]` and **no** `#[ignore]`, so it runs in the ordinary gate on the macOS runner. It creates a case-insensitive volume with `hdiutil create -fs "HFS+" -size 64m`. Run it on macOS and confirm that it fails.
7. Write the Linux substitute `content_store_name_matches_the_hexadecimal_pattern`, a pure unit test that asserts every generated name matches `^[0-9a-f]{64}$`. Run it and confirm that it fails.
8. Implement the lowercase rule in `src/store/content.rs` and confirm that both tests pass on their own platform.
9. Write the failing test `content_store_gc_moves_an_unreferenced_file_to_dead`. Run it and confirm that it fails.
10. Write `src/store/gc.rs`: the reachability walk over the four sources, `gc` which moves every unreferenced file to `media/dead/` and deletes nothing, and `gc --purge` which takes an explicit hash list, recomputes the live set, refuses the whole call with `ProjectError::HashReachable { hash }` when any named hash is live, and then deletes only the named files. Run the test and confirm that it passes.
11. Write the failing test `content_store_purge_refuses_the_whole_call_on_a_live_hash`. Run it and confirm that it fails, then confirm that it passes.
12. Write the failing test `recent_list_drops_the_oldest_entry_past_b142` in `src/recent.rs`. Run `cargo nextest run -p duet-project -E 'test(recent_list)' --no-tests=fail` and confirm that it fails.
13. Write `RecentList` in `src/recent.rs`: read and write `state/recent.json`, add an entry, and drop the OLDEST entry past B142 rather than refuse. A file a user edited that holds more than B142 entries parses first and is then trimmed. Run the test and confirm that it passes.
14. Implement the two cache bounds B59 and B60 in `src/store/content.rs`: the least recently opened file of `derived/peaks/` and of `derived/analysis/` is deleted when its bound is passed, and each deleted file rebuilds on demand.
15. Run `cargo clippy -p duet-project --all-targets -- -D warnings`. Fix every finding in the code.
16. Commit on the branch `chunk/f2-content-store-and-recents`. The native git hook runs `scripts/dod.sh`.

## Tests

All tests of this chunk are unit tests in a `#[cfg(test)] mod tests` at the bottom of the file that holds the code under test. Each test that touches the filesystem creates its own scratch directory under `std::env::temp_dir()` with the process identifier and a nanosecond suffix, and removes it at the end.

| Test | File | Platform | What it asserts |
|---|---|---|---|
| `content_store_names_every_object_in_lowercase_hexadecimal` | `src/store/content.rs` | Both | A stored object's file name is 64 lowercase hexadecimal characters and its directory is the first two of them. Message: "an object name is 64 lowercase hexadecimal characters". |
| `content_store_name_matches_the_hexadecimal_pattern` | `src/store/content.rs` | Both | Over a table of ten sources, every generated name matches `^[0-9a-f]{64}$`. The message names the case: `"source {index}"`. |
| `content_store_is_case_safe_on_a_case_insensitive_volume` | `src/store/content.rs`, `#[cfg(target_os = "macos")]`, no `#[ignore]` | macOS | Two distinct objects whose digests differ only in the value of a byte stay two files on a volume created with `hdiutil create -fs "HFS+" -size 64m`. Message: "two distinct objects stay two files on a case-insensitive volume". |
| `content_store_manifest_is_sorted_by_hash` | `src/store/manifest.rs` | Both | A manifest written from an unsorted entry list reads back sorted by hash. Message: "media.manifest is sorted by hash". |
| `content_store_manifest_round_trips` | `src/store/manifest.rs` | Both | Every field of a `ManifestEntry` survives a write and a read. Message: "a manifest entry round trips through the file". |
| `content_store_gc_moves_an_unreferenced_file_to_dead` | `src/store/gc.rs` | Both | A file the live set does not name moves to `media/dead/` and no file is deleted. Message: "gc moves an unreferenced file and deletes nothing". |
| `content_store_gc_keeps_an_incoming_file` | `src/store/gc.rs` | Both | A file in `media/incoming/` is in the live set, so `gc` leaves it. Message: "an incoming take is live". |
| `content_store_gc_keeps_a_hash_the_undo_stack_names` | `src/store/gc.rs` | Both | A hash that source 3 supplies is live even when no manifest names it. Message: "an undo-stack hash is live". |
| `content_store_purge_refuses_the_whole_call_on_a_live_hash` | `src/store/gc.rs` | Both | A `--purge` list that holds one live hash returns `ProjectError::HashReachable { hash }` and deletes nothing at all. Message: "purge refuses the whole call when any named hash is live". |
| `content_store_purge_deletes_only_the_named_files` | `src/store/gc.rs` | Both | With two dead files and one named on the command line, only the named file is gone. Message: "purge deletes only the hashes the user named". |
| `recent_list_drops_the_oldest_entry_past_b142` | `src/recent.rs` | Both | Adding B142 plus one entries leaves B142 entries and removes the oldest. Message: "the recent list drops the oldest entry past B142". |
| `recent_list_parses_an_over_long_file_and_trims_it` | `src/recent.rs` | Both | A `state/recent.json` a user edited to hold B142 plus ten entries parses and then trims to B142. Message: "an over-long recent file parses first and is then trimmed". |
| `recent_list_round_trips` | `src/recent.rs` | Both | Every field of a `RecentEntry` survives a write and a read. Message: "a recent entry round trips through the file". |

## Verification

1. `cargo nextest run -p duet-project -E 'test(content_store) + test(recent_list)' --no-tests=fail` passes on macOS and on Linux. It fails before this chunk, because no test of either name exists. On the macOS runner the filter also selects `content_store_is_case_safe_on_a_case_insensitive_volume`, which the gate runs.
2. `cargo nextest run -p duet-project --no-tests=fail` passes.
3. `cargo clippy -p duet-project --all-targets -- -D warnings` prints nothing.
4. `cargo machete` reports no unused dependency of `duet-project`.
5. One commit on the branch `chunk/f2-content-store-and-recents` passes the native git hook. The commit carries `crates/duet-project/Cargo.toml` and `Cargo.lock` together.

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
