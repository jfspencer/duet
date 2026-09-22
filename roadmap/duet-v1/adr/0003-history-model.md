# ADR-0003: The history and media model

## Status

Proposed.

## Context

The product commits the composition to a git-like history and checks out an earlier commit (shared brief). The working hypothesis said "a real git repository inside the bundle, driven by gix, audio by content hash". The Engineering Critic showed that this is two designs stated as one. Every bundle, backup, and undo decision hangs from the choice (critic 2.1).

The numbers decide it. One take costs B66, and audio does not compress with zlib. A project with many takes gives gigabytes in the object store, and every checkout of an old commit would rewrite gigabytes in the worktree.

Four more constraints apply.

1. `gix` runs no clean or smudge filter, no hook, and no Large File Storage. A user's global git configuration would change what a plain `git` writes (critic 2.3).
2. `gix` exposes no repack and no garbage collection (critic 2.4).
3. macOS APFS is case insensitive by default, so a mixed-case object name can collide (critic 2.5).
4. A user can run plain `git` inside the bundle. Without a contract, a checkout, a rebase, or an amend corrupts the running application state (critic 2.2).

## Decision

Every number this record relies on is a row of specification section 1.6, cited by its id. Every
rule is cited by its id. This record carries decisions and consequences only.

1. **Git tracks text only.** Specification section 4.2 holds the tracked set.
1a. **The view state is text, and git does not track it.** A tracked viewport would put window churn in every user commit, and a checkout would move the viewport. History records the project, not the window. Specification section 10.2 holds the document, its one writer, and every field it carries.
2. **Audio lives in a content-addressed store inside the bundle and outside the git index.** The object name is a lowercase hexadecimal digest, so two distinct objects cannot merge on a case-insensitive volume such as a default APFS one. Specification section 4.3 holds the layout and the digest.
3. **`media.manifest` is tracked text, sorted by digest.** A commit therefore pins exactly which audio bytes it needs, and a region names a digest and never a path. Specification section 4.3 declares the line.
4. **A checkout restores audio by manifest lookup.** A missing digest is not a failed checkout: the region draws as absent and the core emits `MissingSource`. Specification section 4.5 holds the steps.
5. **A checkout never touches the worktree first.** A parse failure therefore changes nothing. Specification section 4.5 holds the order of the steps.
6. **The live set has several sources and two of them live in memory.** PR R-07 says an undo of a record pass removes the take and keeps the audio file, and PR R-09 and X-13 say a delete keeps it. Without the undo stack and the redo stack the collector reaped a file that a redo needed. Specification section 4.4 names every source.
6a. **`duet gc --purge` takes an explicit hash list and never a wildcard.** The user names what is destroyed, and the tool re-checks reachability against that exact list. Specification section 4.4 holds the command and its refusal.

6b. **`gc` reads the live set through the gateway, never from disk alone.** Two sources live in `DuetCore` memory, and the core holds no lock of any kind, so `Verb::Gc` is a deferred verb: the core builds an immutable snapshot on its own thread and a job thread then walks the reachable commits and scans the store. No code path computes a live set without the core. Specification section 4.4 holds every refusal.
7. **The application watches `.git/HEAD` and `.git/refs`.** On an external move it stops the transport first, and it flushes and hashes any open take before it asks the user. Specification section 4.6 holds the steps, and PR 11 Q8 holds the answer set and its default.
8. **The application refuses every history verb that rewrites while a record pass runs.** The verb returns `GatewayError::RecordInProgress`.
9. **A rewrite of history by an external tool while the project is open is unsupported.** The application detects an unknown `HEAD` and offers a reload. Wherever a pointer must survive a rewrite, the project stores a symbolic reference, never a bare commit identifier.
10. **Bundle creation writes no `extensions.objectFormat` key.** That key is an extension of repository format version 1, and at format version 0 plain `git` refuses to open the repository. A refusal would break the contract of specification section 4.6 and story A-03, which both rest on a user who runs plain `git` inside the bundle. **The repository configuration stays the authority on the object format**, and `History::object_format` is the one read path; `duet.toml` mirrors the value and decides nothing. Specification section 4.5 and specification section 4.7 hold the mechanism, and specification section 15.5 declares `CommitDigest` and `HashKind`.
11. **The object count is bounded by design, not by a repack.** B69 gives the count after one year of ordinary use, and specification section 4.8 states what bounds it.
12. **Undo and history are two mechanisms.** Specification section 4.9 holds the property table that separates them, with the undo bounds B63, B64, and B65 and the history bound B69. A checkout clears the undo stack, and the application says so before it acts.
13. **A multi-file save is crash consistent through a two-phase marker, with a directory `fsync` on both phases.** Without the `fsync` after the renames, a crash after the marker delete can leave the old files, because the rename metadata is not durable. Specification section 4.10 holds the steps and the crash matrix, and this record states the decision and not a second copy of them (DR6).
13a. **The save marker lives in `state/`, not in `derived/`.** The bundle README tells the user that `derived/` is a cache they may delete, and a deletion between the two marker steps would destroy the recovery oracle. A `git clean -xdf` deletes the marker and the temporary files together, so the bundle returns to the old state, which is the answer the recovery gives. Specification section 4.10 numbers the steps and specification section 4.1 holds the directory table.
14. **A record pass writes audio to disk as it records.** The writer streams into the incoming directory and flushes on every ring drain. A crash mid-take leaves the file, and the next start admits it as a take. Specification section 5.10 holds the writer.
15. **Every `gix` type stays inside `duet-project` behind one `History` trait.** No `gix` type appears in any other crate's public signature. Every `History` call runs on a job thread under B19, which TH3 requires. The engine disk thread serves the engine only (TH4).
16. **No stored pointer is a bare commit identifier.** A commit identifier inside a `Verb` call is a transient argument, not stored state. Specification section 4.5 holds the full table of pointers and the form of each one.
17. **Every store carries a bound, and a user-data store carries no automatic one.** `duet status` reports the size of each user-data store, and `--purge` is the only remover. A cancelled or failed job deletes its own temporary directory (ADR 0005 decision 15a). Specification section 4.1 holds the table, with B59 and B60.

## Consequences

Easier:

- The repository stays small and fast. A clone, a diff, and a log are instant.
- A commit is cheap, so a commit per user action is affordable and the log reads as a list of decisions.
- An edit never rewrites audio, so a trim is reversible and a take is safe.
- A crash loses nothing: the save is two-phase, and a take is on disk as it is recorded.
- Version pinning `gix` behind one trait bounds the cost of its frequent breaking releases.

Harder:

- **A `git clone` of the bundle carries no audio.** The unit of backup is the bundle directory. The create dialog and the export surface state it.
- **`git clean -xdf` deletes every take**, because git ignores the media store. The bundle carries a README file beside the store that states it in one line.
- The content store needs its own collector, and the collector must walk every reachable commit.
- A derived cache can be evicted under the user, so a scroll may meet an absent waveform and rebuild it. The lane draws an absent column rather than a stall, which Ardour also does.
- A user who rewrites history while the project is open gets an unsupported state. The application detects it and offers a reload, and the release notes say so.
- No repack exists. If a measurement shows one is needed, it becomes a user-invoked step that names the `git` binary as an explicit dependency.

## Alternatives rejected

| Alternative | Reason |
|---|---|
| Audio bytes as git blobs | Several gigabytes in the object store for fifty takes, and a checkout that rewrites gigabytes in the worktree. Audio does not compress with zlib. |
| Git Large File Storage | `gix` has no Large File Storage support, and a filter needs the `git` binary, which the bundle contract does not carry. |
| Shell out to the `git` binary | It adds a hard runtime dependency and a shell boundary on every commit. `gix` is pure Rust, which the survey prefers. |
| `git2` over libgit2 | It links a C build. `gix` needs no C toolchain, which keeps the build reproducible on both platforms. |
| One commit per edit, so history replaces undo | A commit per note makes the log useless, and a commit is far more expensive than an inverse command. |
| A worktree checkout with plain `git checkout` semantics | It gives the file system authority over live state while the application runs, which breaks the single-writer rule and makes a record pass unsafe. |
| Base32 or base64 object names | Two names that differ only in case are one file on a default APFS volume, so two distinct objects would silently merge. |
| Define reachability from the disk alone, as revision 2 did | The undo stack and the redo stack live in memory, so a disk walk cannot see them. An undo of a record pass would leave a hash that no manifest names, and `--purge` would accept it. |
| Keep the save marker in `derived/` | The document tells the user that `derived/` is a cache they may delete. A deletion during a save would leave a mixture of old and new files with no oracle to resolve it. |
| Define reachability from the committed history alone, as the first draft did | A take recorded and not yet committed lives in the working manifest only. The collector would move it to `media/dead/` and `--purge` would then delete it. That is a documented path to data loss. |
| Let `--purge` empty `media/dead/` on one confirmation | A confirmation dialog is not a name. An explicit hash list makes the user state which audio is destroyed, and it lets the tool re-check reachability against that exact list. |
| Delete unreferenced media during garbage collection | A wrong reachability walk would destroy user audio. A move to `media/dead/` is reversible, and Ardour takes the same care. |
