# ADR-0002: The canonical storage format

## Status

Proposed.

## Context

The operator set MusicXML as the interchange format and as the first storage format. A canonical format of our own may replace it as storage when the application needs it. The storage format must be durable and extensible.

The product needs three things from a storage format that MusicXML does not give.

1. **Coverage.** The project holds takes, playlists, regions over immutable sources, record modes, part assignments, mixer state, automation curves, pitch tracks, and a history. MusicXML represents none of them (critic 3.1).
2. **A hand-editable surface.** A terminal agent reads and edits the composition. The file is the primary interface (PR 11 Q2).
3. **A readable git diff.** The history model commits the document, so the diff quality of a one-note change is a product property, not a detail.

Two repository rules constrain the MusicXML path. `deny.toml` sets `unknown-git = "deny"` and `CLAUDE.md` forbids a git dependency, so a fork cannot be a dependency. A vendored fork under `crates/` would inherit the full lint policy on code that no Duet engineer wrote (critic 3.2). The `musicxml` crate is idle, and the survey records the date of its last release.

## Decision

Every number this record relies on is a row of specification section 1.6, cited by its id. Every
rule is cited by its id. This record carries decisions and consequences only.

1. **The canonical format is the storage format from version one.** MusicXML is an import and export format only.
2. **The canonical format holds the four properties of specification section 3.6.** The same score must produce the same bytes, because the history model commits the document and a diff is a product property. That section names each property, names the records that carry the unknown-field bag, and states the one exclusion. Decision 3c below gives the reason the session and mix documents refuse instead.
3. **The document splits across several files, and each collection that grows with the music is JSON Lines.** Specification section 4.1 holds the layout.
3b. **Every canonical document carries its own `SchemaVersion`, and the type carries an order**,
because the migrate-or-refuse decision of specification section 3.6 cannot be written without one.
Each reader returns its own crate's error. Specification section 1.6 declares every schema constant,
and VR1 names the use of the order. The first draft gave a version to the score document alone while
it refused a version the other documents did not carry.

3c. **The `extra` bag covers the score files alone.** A foreign tool and a terminal agent hand-edit
the score files, and an unknown notation field that survives a round trip costs nothing. A silently
kept unknown routing field, send level, or record mode would produce audio the user did not ask for,
so the session and mix documents refuse a version they cannot read instead of keeping what they do
not understand. **`state/view.json` is the one exception**, because git does not track it and a lost
viewport is recoverable. Specification section 3.6 and specification section 10.2 hold both rules.

4. **Every element carries a stable identifier from a counter that the document persists.** An agent names a note by identifier, and the identifier survives a save, a commit, and a hand edit. Specification section 3.2 declares the identity rule.
4a. **The sort key of each JSON Lines file is total, and every field of that key carries an order.** Without a total key the deterministic-output property cannot be computed. Specification section 3.6 holds each key, and VR1 names the use for every field that carries the order.
4b. **A non-finite float never reaches the writer, and it never comes back from one.** `serde_json` writes `null` for a NaN or an infinity, which fails the round trip. VR4 puts a `Finite` in every float field, and ADR 0001 decision 3b keeps the constructor on the deserialization path. A hand edit of a canonical file is a product path, so the guard must hold on the way in as well as on the way out.
4c. **The reader reports every unknown key.** A key that lands in the `extra` bag is returned as an `ImportWarning`, and the command-line interface prints it. A misspelt field from a hand edit is visible at once and does not look like a correct edit.
5. **MusicXML support is a narrow in-house reader and writer in `duet-interchange`.** It covers the subset Duet models, and it keeps every unmodelled element so that an export round trip returns it. Specification section 3.7 holds the reader, the writer, and the bucket.
6. **No MusicXML fork enters the workspace.**

## Consequences

Easier:

- A one-note change gives a one-line git diff, so the history is readable and a merge is tractable.
- An agent patches one line of `score/notes.jsonl` with an ordinary text edit.
- A newer field survives an older build, so a version skew loses no user data.
- `duet-score` carries no XML dependency, and neither does any crate that depends on it.
- A save with no change produces no diff, because the writer is deterministic.

Harder:

- Two serialization surfaces exist from day one: the canonical reader and writer, and the MusicXML reader and writer. The second one is narrow by design.
- The MusicXML subset is stated, not complete. The export surface says which elements it writes, so the user is not surprised.
- The canonical format needs a migration chain from the first schema change onward. The chain is one function per step, and a test loads every historical fixture.
- `Pitch` carries an order that is sounding order, not written order. A reader of the sorted file sees C-sharp before D-flat at the same sounding pitch. That choice is arbitrary and stable, and specification section 3.3 states it.
- JSON Lines is larger on disk than a binary form. A score at the product budget B1 is small beside one take, whose size is B66.

## Alternatives rejected

| Alternative | Reason |
|---|---|
| MusicXML as storage, with our data in `<miscellaneous>` and private elements | An unmodelled element is lost on write, the identifiers are optional, and the diff is unreadable. The result is a canonical format plus an interchange format anyway, so the canonical format is not deferred, it is hidden. |
| A vendored fork of the `musicxml` crate | `deny.toml` forbids a git dependency, so the fork must live under `crates/` and inherit `missing_docs`, `indexing_slicing`, `as_conversions`, and `unwrap_used` on every item. That is a large unbudgeted body of work on code nobody here wrote. |
| A binary format such as bincode or postcard | An agent cannot patch it, and git cannot diff it. Both are product requirements. |
| SQLite as the project file | A binary file in git, plus a second concurrency model beside the audio engine. It also breaks the rule that media is findable without the application. |
| TOML for the whole document | A long note array in TOML is slow to parse and large to diff. TOML is right for `duet.toml`, which is small and read once. |
| One large JSON file | A one-note change rewrites a file whose diff context is the whole document. JSON Lines gives a one-line diff for the same edit. |
| A custom line-oriented text format of our own design | It needs a hand-written parser, its own fuzz surface, and its own tooling. JSON Lines gives the same diff quality with `serde` and needs no parser. |
