"""Placement, derive, edge, dependency, snapshot, and register guard.

Prototype for `cargo xtask check-placement <document>`, which chunk M0 ports
to `tools/xtask/src/check_placement.rs`. The contract lives in architecture
section 1.5, "The placement guard", and every rule there carries an id `PG<n>`.
Section 1.9 pairs each rule with its one probe `PP<n>` (DR5). The guard runs in
the `plan-lint` job of `.github/workflows/ci.yml`, never in `scripts/dod.sh`.

PG1  usage with no argument, exit 2.
PG2  a document that does not open, exit 2; the guard is fail-closed.
PG3  a parse that yields no candidate type, and a declaration body that
     states a comment where a field belongs. A body that holds only a
     comment, and a `/* ... */` comment that stands in a field's place, each
     state no field, so PG19, PG23, PG24, and PG26 read the declaration and
     find nothing to refuse. Revision 11 substituted an empty body for
     forty-four declarations and all four rules passed over them in silence
     (critic WR-1).
PG4  a declared type the section 1.5 table omits. **The guard carries no
     hidden set.** The candidate drop list is a fenced block of section 1.5,
     and the primitive trait sets and the primitive sizes are each a fenced
     block of section 1.9. The guard exits 1 with a named line when any one
     of the three is absent, and it holds no fallback for any of them.
     **The rule's own limit, stated here.** The candidate set drops every
     name the drop list carries BEFORE PG4 runs, so a declaration whose name
     the drop list carries is invisible to this rule: a `pub struct ExitCode`
     with no section 1.5 row raises `DECLARED` and leaves `UNPLACED` at zero.
     `Duration` is the one name that section 1.5 places and the drop list
     also carries, so it is the one live overlap today (critic concern 4).
PG4b a section 1.5 name that no Rust block declares. There is no suffix
     template: every placed name carries its own declaration.
PG5  one type declared by two Rust blocks (DR2).
PG6  a framework name the 1.5 table claims for a non-application crate.
PG7  a framework type inside a non-application declaration, bare or path
     qualified; the check runs before paths are stripped and matches the last
     segment.
PG8  an enum arm name is masked inside its own enum only.
PG9  an all-`Copy` type that derives no `Copy` and carries no expectation.
PG10 a `Copy` derive over a field this document proves is not `Copy`.
PG10b a `Copy` derive over a field whose Copy-ness this document does not decide.
PG11 an edge claim the section 1.3 list does not carry.
PG12 a section 1.2 dependency row that omits a crate its declarations use.
PG13 a type the audio thread publishes that is not declared or is not `Copy`.
PG14 an undecidable field of ANY declaration, with or without a `Copy` derive,
     counted as UNKNOWN and printed with its type and its field.
PG17 an undecidable field type that the justified-unknown table of section 1.9
     does not name. One direction only: a row no field uses is not a failure.
PG18 an external type a declaration names that the section 1.9 external-verdict
     table does not name. There is no drop list.
PG15 a `Declared in` cell that omits a section which declares one of the row's
     types.
PG16 a test that section 14 selects and that no chunk writes.
PG19 a derive-closure break: a declaration that derives one of the nine
     closure traits over a field type that does not have it. The trait set of
     an external type comes from the `Traits` column of the section 1.9 table,
     and the trait set of a declared type comes from its own derive list plus
     the trait impls this document writes by hand.
PG20 a field type that sits in a crate the section 1.3 graph does not reach
     from the declaring crate.
PG21 an expectation that Appendix B.1 does not list, or a B.1 row that no
     declaration carries. Both directions, and Appendix B.1 holds one table
     per lint: `missing_copy_implementations` and `variant_size_differences`.
     Revision 11 read the first table alone, so the second one had no rule
     that held it and the code to one set (critic concern 3).
     **The rule also reads every size a reason states.** For every `#[expect]`
     attribute in the document and for every Appendix B.1 reason cell, a bare
     byte count is a failure: a digit run followed by `byte` or `bytes` that
     no `B` id reference touches and that no code span naming a section 1.9
     size-block row holds. The house forms are "the type is B<id> bytes" and
     "the type is the size the PG24 table prints for `<Name>`". Revision 11
     compared names only, so "the type is 104 bytes" changed to "9000 bytes"
     passed all three guards (critic WR-2).
PG22 a `Hash` or `Ord` derive that the VR1 derive-use table of section 3.5
     names no use for. One direction only, and the rule states the limit.
     `Eq` left the rule in revision 11, because VR1 now derives `Eq` wherever
     every field supplies it. PG23 holds that half, in the other direction.
PG23 a `PartialEq` derive with no `Eq`, no hand `impl Eq`, and a body whose
     every field type supplies `Eq`. `clippy::derive_partial_eq_without_eq` is
     a `nursery` lint, the workspace sets `nursery` to `deny`, and the
     `-D warnings` flag makes the shape fail to build. A field whose traits
     the guard cannot decide makes the type UNDECIDED, which is a count and
     not a failure. The rule skips a trait, a body this document writes
     `/* private */` or leaves empty, and an application declaration that
     names a framework type, which is the class PG19 exempts for the same
     reason: section 1.9 decides no `gpui-kit` name.
PG24 an enum whose largest arm is more than three times its next largest arm,
     which is the rustc lint `variant_size_differences`. The lint sits at
     `warn` in `[workspace.lints.rust]` and `.cargo/config.toml` adds
     `-D warnings`. The guard computes a size for every declaration it can
     decide, and it states the model here.
     * A primitive carries its own size and alignment, and the section 1.9
       block `Every primitive size the guard uses` states every one of them.
       The guard holds no copy, so this paragraph names no number.
     * Every other size comes from the section 1.9 fenced block
       `Every size the guard records`, whose lines read
       `<type expression> <size> <align>`. A row states the expression
       exactly as a field writes it, so a row is either a bare name
       (`Box 8 8`) or a whole expression (`Box<str> 16 8`,
       `Option<Span> 32 8`). Each row is a compiler fact the roster compile
       measured, which is DR3 exemption 4. The guard matches the whole
       expression first, with every space removed, and falls back to the bare
       head name after that. With no block every recorded name is undecided
       and the guard guesses none.
     * **A niche is outside the model.** Rust puts a discriminant in a spare
       bit pattern, so a layout model cannot reach the compiler's answer for
       `Option<Span>`, which is 32 bytes and not 40. The model therefore
       reads the HEAD of an expression and never a nested argument, and it
       states three language rules. A tuple is a struct of its items. A `Box`,
       an `Arc`, or an `Rc` over a slice or over `str` is a fat pointer, which
       is 16 bytes at alignment 8; over a sized argument it takes its own head
       row. `Option`, `SmallVec`, and `ArrayVec` take no head row, because a
       niche and an inline array each depend on the argument, so each one
       needs a row for the whole expression. A `BTreeMap` is its own size
       whatever its payload is (critic C-5).
     * A struct sorts its fields by decreasing alignment, which is the
       size-optimal `repr(Rust)` order rustc picks, pads each field to its own
       alignment, and rounds the total up to the largest field alignment. A
       zero-field struct is size 0 and alignment 1.
     * An enum lays out each arm as a struct by the same rule. Its alignment
       is the largest arm alignment, and its size is the largest arm size plus
       one discriminant byte, rounded up to that alignment. A fieldless enum
       is one byte at alignment 1.
     * An array is its length times its element size, at the element
       alignment. The length is a literal or a section 1.6
       `pub const NAME: usize` constant.
     * A slice, a tuple, and a reference are undecided. A generic application
       that no row states falls back to its bare head name, and it is
       undecided when the head name decides none. Undecided propagates, and
       the guard never guesses a size.
     The failure reads the arms that carry a payload, **each one padded to
     the payload alignment of the enum**, which is what rustc compares. An arm
     of size zero carries none this model can see, and rustc holds the same
     guard: its own lint fires only when the second largest arm is larger than
     zero. **rustc lints a direct tag only.** It puts the tag in a spare bit
     pattern of the largest arm when that arm has one, and it returns before
     the comparison in that case. PG24 asks the same question first: an
     integer and a float offer no spare pattern, `bool` and `char` each offer
     one, a declared enum offers its unused discriminants, a struct and an
     array offer what their parts offer, and every other name offers one.
     **The model states no niche optimization**, so a size it computes is an
     upper bound. `ROSTER_SIZES=1` prints every decided size, every arm size
     of a decided enum, and every field expression the model cannot size. The
     last list names each expression that needs a recorded row. The mode
     changes no exit code.
     **A single-site `#[expect(variant_size_differences, ...)]` takes a
     declaration out of the failure set**, exactly as an
     `#[expect(missing_copy_implementations, ...)]` takes one out of PG9. The
     run counts it on `VARIANT EXPECTED`, and PG21 holds the sites and the
     second Appendix B.1 table to one set.
PG20b a constant a declaration names, in a crate the section 1.3 graph does
     not reach from the declaring crate. An array length and a const generic
     argument are values, not types, so PG20 cannot see one. **The rule's own
     limit, stated here**: a constant inside a function body is invisible to
     it, and PG28 holds that half.
PG27 a registered block whose heading, marker, anchoring, row count, row
     membership, or token shape is wrong (DR7, WR-18). `DATA_BLOCKS` is the
     register and the document carries the same id and the same minimum in a
     marker line. The rule also refuses a marker that appears more than once
     in the document and a marker whose id the register does not hold, which
     `read_block` cannot see because it anchors inside one heading region
     (critic C-13, C-14). **Every row of every block names a referent**: the
     document states one membership kind per block in its own
     `block-members` block, `MEMBER_KINDS` is the set this guard runs, and a
     row whose referent no other block and no declaration holds is a
     failure. A `rows>=` minimum refuses a DELETED row and accepts an ADDED
     one, and a Critic probe added one name to the candidate drop list and
     turned a red document green (critic WR-18). **The rule's own limit,
     stated here**: it decides that a block is present, whole, well formed,
     and that each row names a real entity; it decides no row's meaning.
PG28 a shared limit of section 1.6 that an enforcing crate cannot reach.
     **The rule's own limit, stated here**: the enforcer list is a hand list,
     because an enforcement lives in a function body this document does not
     write.
PG29 a section 1.9 probe table that is not one set with the rule ids the
    prototypes themselves carry, a row
     whose probe id does not follow its rule id, or a recorded cell that
     states no exit code. **The rule's own limit, stated here**: the recorded
     TEXT of a cell is unchecked, because only a run produces it.
PG30 a `B` citation of the document and the section 1.6 `Used by` column
     that are not one set, in both directions (critic C-1). **The rule's own
     limit, stated here**: it reads this document alone and it skips the
     budget block, because one budget row that derives from another is a
     derivation and not a use.
PG26 a heap allocation inside audio-owned state. TH1 states that the audio
     thread never allocates and never frees. The rule reads the fenced block
     `#### Every audio-owned declaration` of section 5.7, which names one type
     per line, and it walks every declaration in that set and every declared
     type each one reaches through a field. A field type that names `Box`,
     `Vec`, `String`, `Arc`, `Rc`, `BTreeMap`, `BTreeSet`, `HashMap`,
     `HashSet`, `VecDeque`, `Cow`, or `PathBuf`, or that names an external
     type whose section 1.9 `Why` cell states a heap, is a failure.
     **An exemption row answers the FREE half of ONE named field whose head
     is ONE named type**, so a container the audio thread only reads through
     is outside this rule and the exemption is data rather than prose. The row
     is `Type.field Head reason`, and a field whose head differs from the row
     falls through to the heap test; keying the row on the path alone let a
     Critic probe hide a bare `Box<[u8]>` behind it. An exemption answers
     NEITHER PG26b nor PG26c. A token in either block that section 1.5 places
     nowhere is exit 2 (critic CR-2, CR-14).
     **One escape exists and it is a type, not a sentence** (critic CR-16).
     A field whose outermost head is an external name whose section 1.9 `Why`
     cell carries the word "defers" carries its whole subtree to the collector
     thread, so the walk stops there and counts it on `AUDIO DEFERRED`.
     `basedrop::Owned` and `basedrop::Shared` are the two names that carry it.
     A bare `rtrb`, `triple_buffer`, `crossbeam_queue`, `async_channel`, or
     `std::thread` handle is a heap name like any other and fails; revision 13
     hid three `rtrb` ends behind two exemption lines and a Critic probe put a
     `Vec<u8>` behind one of them and got a green run.
     **The walk continues through the wrapper** and suppresses the heap test
     alone, so PG26b and PG26c still read everything the wrapper carries.
     **The rule's own limit, stated here**: it reads the free half of TH1 and
     it decides neither the allocation half nor the lock half. PG26b and PG26c
     hold those.
PG26b a container that GROWS inside audio-owned state, at any depth and
     inside a deferring wrapper. A wrapper answers the free half of TH1 and it
     can never answer the allocation half, so `Owned<Vec<u8>>` calls the
     allocator on the audio thread on every `push` and revision 14 accepted it
     (critic WR-22). The set is `Vec`, `String`, `HashMap`, `HashSet`,
     `BTreeMap`, `BTreeSet`, `VecDeque`, `Cow`, `PathBuf`, and `SmallVec`,
     plus every external name whose section 1.9 `Why` cell states that the
     type grows. **The rule's own limit, stated here**: a fixed `Box<[T]>`
     allocates once and passes, so the rule decides the shape of a container
     and never the number of elements a caller puts in one.
PG26c a LOCK inside audio-owned state, at any depth, inside a deferring
     wrapper and behind an exemption line alike. TH1 bans a lock beside an
     allocation and a free, and PG26 read the heap half alone, so a `Mutex`
     field and an `RwLock` field each passed every guard (critic WR-23). The
     set is `Mutex`, `RwLock`, `ReentrantLock`, `Condvar`, `Barrier`,
     `OnceLock`, `LazyLock`, and the three guard types, plus every external
     name whose section 1.9 `Why` cell states a lock. The last path segment is
     the name, so `parking_lot::Mutex` reds here as `std::sync::Mutex` does.
     **The rule's own limit, stated here**: it reads a declared field and it
     cannot see a lock a function body takes.

**Every block and every table this guard reads is a registered block, and PG27
holds all of them.** `DATA_BLOCKS` names each one, the document states the
same id and the same minimum row count in a marker line on the line before the
block, and a failure of any part of that contract prints its own named `FAIL`
line and exits 2. A row count is the only check that separates a damaged block
from an absent one, and a marker is the only check that separates a block from
the next one: revision 12 read each block with a lazy regular expression, so
deleting the audio-owned block under a kept heading made the run parse the next
fence, print `AUDIO OWNED: 88`, and exit 0 (critic CR-14), and deleting the
name map the same way left one usable mapping and a clean `DEP MISSING: 0`
(critic WR-16).
"""

import os
import re
import sys

# The two crate rows that may name a framework type.
APP_ROWS = {"crates/duet", "duet-agent"}

# The marker that a declaration carries when the audio thread owns its value
# (critic C15-3). It is the second source of the root set, so a one-for-one
# swap in the block fails in both directions.
AUDIO_MARK = "**Audio-owned**"

# The four prototypes PG29 reads for their own rule ids. This guard holds
# every `PG` id but PG25 and PG32; `roster_compile.sh` holds PG25,
# `closure_check.py` holds PG32, and `conversion_check.py` holds every `CG`
# id. Revision 19 said "three prototypes" and named four files, because PG32
# had moved and the sentence had not (critic C19-1).
PROTOTYPES = (
    "placement_check.py",
    "conversion_check.py",
    "roster_compile.sh",
    "closure_check.py",
)

RULE_ID = re.compile(r"\b((?:PG|CG)\d{1,2}[a-z]?)\b")


def implemented_rule_ids():
    """Every rule id the prototypes implement, SCANNED and never typed (PG29).

    Revision 19 held the denominator as a literal tuple in this file, and
    called it "every rule id the three prototypes implement" (critic C19-3).
    A literal is the author's claim about the code, so a rule deleted from the
    code and left in the tuple and in the probe table stayed green, and the
    one rule that exists to prove DR5's equality proved a tuple against a
    table. The set now comes from the files themselves.

    **The rule's own limit, stated at its section 1.5 site**: a rule id is a
    TOKEN of a prototype, so an id that survives in a comment after its
    implementation is deleted still counts. The scan is strictly stronger than
    a literal, because a deleted file, a renamed id, and an id typed into the
    table alone are each red; it is not a proof that the code behind an id
    still runs, and only a probe proves that.
    """
    here = os.path.dirname(os.path.abspath(__file__))
    found = set()
    for name in PROTOTYPES:
        path = os.path.join(here, name)
        if not os.path.isfile(path):
            return None, f"the prototype `{name}` does not open, so the rule set is unknown"
        with open(path, encoding="utf-8") as handle:
            found |= set(RULE_ID.findall(handle.read()))
    if not found:
        return None, "the prototypes name no rule id, so the denominator is zero"
    return tuple(sorted(found)), None

# Every membership kind PG27 implements (WR-18). The document names one kind
# per registered block in its own `block-members` block, and a kind this tuple
# does not hold is exit 2: a rule the guard cannot run is not a rule.
MEMBER_KINDS = (
    "carrier-end",
    "crate-name",
    "pin-name",
    "map-crate",
    "not-declared",
    "ownership-row",
    "drop-name",
    "const-crate",
    "limit-row",
    "rule-id",
    "block-id",
    "verdict-name",
    "external-name",
    "expr-head",
    "sub-id",
    "declared-name",
    "first-declared",
    "primitive-name",
    "unknown-name",
    "field-path",
    "cited-elsewhere",
    "gate-site",
    "fault-arm",
    "site-crate",
    "site-path",
    "chunk-id",
    "mechanism-name",
    "impl-site",
    "review-id",
    "line-owner",
    "chunk-pair",
)

EDGE_VERBS = (
    "uses", "reads", "holds", "carries", "calls", "paints", "drains", "writes", "depends",
)

# The nine traits PG19 closes over. `Debug` and `Clone` are outside the rule:
# `Debug` is denied-by-lint everywhere and `Clone` follows `Copy`.
CLOSURE_TRAITS = (
    "Copy", "Default", "Serialize", "Deserialize",
    "PartialEq", "Eq", "Hash", "Ord", "PartialOrd",
)

# The two traits VR1 governs from revision 11 (PG22). `Eq` left the rule,
# because PG23 now demands it wherever every field supplies it.
VR1_TRAITS = ("Hash", "Ord")

# A smart pointer whose single argument is unsized. The value is then a fat
# pointer: an address and a length, which is 16 bytes at alignment 8.
FAT_POINTER_HEADS = ("Box", "Arc", "Rc")

# The heads whose size the argument decides, so a head row states nothing.
# `Option` carries a niche and the other two carry an inline array. Each one
# needs a row for the whole expression, or PG24 calls the field undecided.
INLINE_HEADS = ("Option", "SmallVec", "ArrayVec")

# Every standard container that owns a heap allocation (PG26). Section 1.9
# names the rest: a row whose `Why` cell states a heap joins this set.
HEAP_NAMES = (
    "Box", "Vec", "String", "Arc", "Rc", "BTreeMap", "BTreeSet",
    "HashMap", "HashSet", "VecDeque", "Cow", "PathBuf",
)

# Every container that GROWS, which means it may call the allocator after it
# is built (PG26b). A deferring wrapper answers the free half of TH1 and it
# can never answer the allocation half, so one of these fails inside an
# `Owned` and inside a `Shared` exactly as it fails bare (critic WR-22).
# Section 1.9 names the rest: a row whose `Why` cell states that the type
# grows joins this set. `ArrayVec` and `Box<[T]>` are fixed and are absent.
GROW_NAMES = (
    "Vec", "String", "HashMap", "HashSet", "BTreeMap", "BTreeSet",
    "VecDeque", "Cow", "PathBuf", "SmallVec",
)

# Every lock (PG26c). TH1 bans a lock on the audio thread beside an
# allocation and a free, and PG26 read the heap half alone: a `Mutex` field
# and an `RwLock` field both passed every guard (critic WR-23). The last path
# segment is the name, so `parking_lot::Mutex` reds here as `std::sync::Mutex`
# does. Section 1.9 names the rest: a row whose `Why` cell states a lock joins
# this set.
LOCK_NAMES = (
    "Mutex", "RwLock", "ReentrantLock", "Condvar", "Barrier",
    "OnceLock", "LazyLock", "MutexGuard", "RwLockReadGuard",
    "RwLockWriteGuard",
)

# The two lints a declaration of this document expects at its own site, and
# the two Appendix B.1 tables PG21 holds against them.
EXPECTED_LINTS = ("missing_copy_implementations", "variant_size_differences")

# Every primitive with no spare bit pattern. `bool` and `char` carry one, and
# so does every other name this model does not decide. PG24 reads the set to
# tell a direct tag from a niche tag.
NICHE_FREE = {
    "u8", "u16", "u32", "u64", "u128", "usize",
    "i8", "i16", "i32", "i64", "i128", "isize",
    "f32", "f64",
}


def read_document(path):
    """The whole document as text, or None when it does not open."""
    try:
        with open(path, encoding="utf-8") as handle:
            return handle.read()
    except OSError:
        return None


# Every block and every table this guard reads, by the id its marker states
# (DR7, PG27). The tuple is `(heading, kind, minimum rows)`. The document
# states the same id and the same minimum in a marker line on the line before
# the block; a disagreement is exit 2, because a count the document states and
# a count the guard expects must be one number.
DATA_BLOCKS = {
    "crate-table": ("The crate dependency table", "table", 16),
    "carrier-table": ("Every cross-thread carrier, and the two ends it needs", "table", 25),
    "name-map": ("The third-party name map", "text", 23),
    "edge-list": ("The internal edge list", "text", 18),
    "framework-types": ("Framework types", "text", 7),
    "ownership-table": ("The type ownership table", "table", 16),
    "drop-list": ("The candidate drop list", "text", 8),
    "constants": ("Every workspace constant", "rust", 122),
    "shared-limits": ("Every shared limit and its enforcers", "text", 5),
    "probe-table": ("Every rule, its probe, and the recorded result", "table", 64),
    "external-verdicts": ("Every external type, and its verdict", "table", 34),
    "external-paths": ("Where every external name comes from", "text", 70),
    "pins": ("The external crate pins the roster compile uses", "text", 13),
    "recorded-sizes": ("Every size the guard records", "text", 51),
    "substitutions": ("Every substitution the roster compile applies", "text", 8),
    "drop-impls": ("Every declaration with a hand-written Drop impl", "text", 3),
    "impl-sites": ("Every impl block the roster compiles", "text", 65),
    "heap-names": ("Every heap-owning name the audio rules refuse", "text", 12),
    "grow-names": ("Every growable name the audio rules refuse", "text", 10),
    "lock-names": ("Every lock name the audio rules refuse", "text", 10),
    "primitive-traits": ("Every primitive trait set the guard uses", "text", 3),
    "primitive-sizes": ("Every primitive size the guard uses", "text", 16),
    "justified-unknowns": ("The justified unknowns", "table", 2),
    "vr1-table": ("Every VR1 derive and its use", "table", 27),
    "audio-owned": ("Every audio-owned declaration", "text", 44),
    "audio-exempt": ("Every audio-owned field the rule exempts", "text", 1),
    "audio-reachable-leaf": ("Every reachable leaf the closure rule allows", "text", 46),
    "block-members": ("Every registered block and its membership rule", "text", 51),
    "budget-table": ("Every budget and bound", "table", 148),
    "b1-convert": ("Conversion suppressions", "table", 7),
    "b1-complexity": ("Complexity suppressions", "table", 6),
    "b1-copy": ("Expectations for missing_copy_implementations", "table", 4),
    "b1-variant": ("Expectations for variant_size_differences", "table", 3),
    "phase-table": ("The phase table", "table", 16),
    "selected-tests": ("Every test this section selects by name", "table", 8),
    "snapshot-table": ("High-rate traffic: latest value, lock free, no event", "table", 7),
    "rule-blocks": ("Which rule reads which block", "table", 33),
    "closure-r16": ("The revision-16 review, over the frozen document", "table", 46),
    "closure-r17": ("The revision-17 review, over the frozen document", "table", 26),
    "closure-r18": ("The revision-18 review, over the frozen document", "table", 20),
    "closure-r19": ("The revision-19 review, over the frozen document", "table", 45),
    "closure-r20": ("The revision-20 review, over the frozen document", "table", 23),
    "closure-r21-inner": ("The revision-21 inner review, over the frozen document", "table", 13),
    "closure-r21": ("The revision-21 external review, over the frozen document", "table", 45),
    "closure-r22-inner": ("The revision-22 inner review, over the frozen document", "table", 35),
    "closure-r23-inner": ("The revision-23 inner review, over the frozen document", "table", 31),
    "line-map": ("Every chunk line and the crate it owns", "text", 16),
    "audio-asserted": ("Every audio-owned root the closure does not reach", "text", 14),
    "phase-pair-exempt": ("Every same-phase crate edge that does not bind", "text", 24),
    "gate-defects": ("Six planted gate defects, and the rule that catches each one", "table", 6),
    "fault-messages": ("Every engine fault and the line the user reads", "table", 16),
}

MARKER = re.compile(r"(?m)^<!-- GUARD BLOCK id=([a-z0-9-]+) rows>=([0-9]+) -->$")


def split_row(line):
    """The cells of one markdown table row, as a renderer reads them.

    It splits on every `|` the author did NOT escape and then unescapes each
    cell, because `\\|` inside a backtick span is one literal pipe and not a
    cell wall (critic C21-5). Revision 21 split on every `|`, so a row the
    Architect had escaped correctly still parsed one way for a guard and
    another way for a reader, and PG38 and every membership rule would have
    disagreed about the same row.
    """
    return [
        cell.replace("\\|", "|").strip()
        for cell in re.split(r"(?<!\\)\|", line)[1:-1]
    ]


def read_block(source, block_id, register=True):
    """One registered block, as `(rows, reason)` (PG27, DR7).

    A registered block carries a `####` heading of its own, a marker line on
    the line before its opening fence or its table header, a minimum row count
    inside that marker, and a row in `DATA_BLOCKS`. Every one of those is
    checked here, and a failure returns a reason that `main` prints before it
    exits 2.

    **The anchor is the marker and never a search.** Revision 12 read each
    block with a lazy regular expression from the heading to the next fence,
    so a deleted block under a kept heading parsed the next fence in the
    document: the audio-owned set grew from 22 names to 88 source fragments
    and the run exited 0 (critic CR-14, WR-16).

    `rows` is a list of cell lists for a table, and a list of non-empty lines
    for a text or a Rust block.
    """
    heading, kind, minimum = DATA_BLOCKS[block_id]
    found = list(re.finditer(r"(?m)^#### " + re.escape(heading) + r"\s*$", source))
    if len(found) != 1:
        return None, f"the heading `#### {heading}` appears {len(found)} times"
    start = found[0].end()
    rest = source[start:]
    stop = re.search(r"(?m)^#{1,4} ", rest)
    region = rest[: stop.start()] if stop else rest
    marks = list(MARKER.finditer(region))
    if len(marks) != 1:
        return None, f"the marker line appears {len(marks)} times under the heading"
    mark = marks[0]
    if mark.group(1) != block_id:
        return None, f"the marker states id `{mark.group(1)}` and the register states `{block_id}`"
    if register and int(mark.group(2)) != minimum:
        return None, (
            f"the marker states rows>={mark.group(2)} and the register states {minimum}"
        )
    if not register:
        minimum = int(mark.group(2))
    tail = region[mark.end() :]
    if not tail.startswith("\n"):
        return None, "the marker does not end its own line"
    tail = tail[1:]
    if kind == "table":
        head = re.match(r"\|[^\n]*\|[ \t]*\n\|[-\s|:]+\|[ \t]*\n", tail)
        if not head:
            return None, "no table header follows the marker"
        rows = []
        for line in tail[head.end() :].splitlines():
            if not line.startswith("|"):
                break
            rows.append(split_row(line))
    else:
        fence = re.match(r"```([a-z]*)\n(.*?)^```", tail, re.S | re.M)
        if not fence:
            return None, "no fenced block follows the marker"
        wanted = "rust" if kind == "rust" else "text"
        if fence.group(1) != wanted:
            return None, f"the fence is `{fence.group(1)}` and `{wanted}` is required"
        rows = [line for line in fence.group(2).splitlines() if line.strip()]
    if len(rows) < minimum:
        return None, f"it holds {len(rows)} rows and the stated minimum is {minimum}"
    return rows, None


def read_blocks(source):
    """Every registered block, as `({id: rows}, [(id, reason), ...])` (PG27).

    The register and the section 1.9 rule-to-block map are held to one set
    here, so a block the register holds and no rule claims, and a claim over
    an id the register does not hold, are each a failure. A register nobody
    audits is a register that drifts.
    """
    parsed, failures = {}, []
    for block_id in DATA_BLOCKS:
        rows, reason = read_block(source, block_id)
        if reason is not None:
            failures.append((block_id, reason))
            continue
        parsed[block_id] = rows
    if "rule-blocks" not in parsed:
        return parsed, failures
    claimed = set()
    for cells in parsed["rule-blocks"]:
        if len(cells) >= 2:
            claimed |= set(re.findall(r"`([a-z0-9-]+)`", cells[1]))
    for block_id in sorted(set(DATA_BLOCKS) - claimed):
        failures.append((block_id, "the section 1.9 rule-to-block map claims no rule for it"))
    for block_id in sorted(claimed - set(DATA_BLOCKS)):
        failures.append((block_id, "the rule-to-block map names it and the register does not hold it"))
    return parsed, failures


def token_rows(rows):
    """Every whitespace token of a text block, as a list."""
    return [token for line in rows for token in line.split()]


def cell_names(cell):
    """Every code span of one markdown cell that names an identifier."""
    return re.findall(r"`([A-Za-z0-9_]+)`", cell)


def candidate_drop_list(blocks):
    """The section 1.5 candidate drop list, as a set of names (PG4)."""
    return set(token_rows(blocks["drop-list"]))


def external_verdicts(blocks):
    """The section 1.9 external-verdict table.

    Returns `({name: verdict}, {name: (traits, unconditional)})`.

    PG18 reads the verdict. There is no drop list: every external type a
    declaration names has a row with `Copy`, `not Copy`, `transparent`, or
    `undecided` (critic R2).

    PG19 reads the `Traits` cell. A trait named there is one the external type
    supplies **when every type argument supplies it**; a trait written with a
    trailing `!` is unconditional, which is how `Vec` supplies `Default` over
    a payload that has none.
    """
    verdicts, traits = {}, {}
    for cells in blocks["external-verdicts"]:
        if len(cells) < 4:
            continue
        verdict = cells[1].strip("`").lower()
        supplied, unconditional = set(), set()
        for token in re.findall(r"`([A-Za-z]+!?)`", cells[2]):
            bare = token.rstrip("!")
            if bare not in CLOSURE_TRAITS:
                continue
            supplied.add(bare)
            if token.endswith("!"):
                unconditional.add(bare)
        for name in cell_names(cells[0]):
            verdicts[name] = verdict
            traits[name] = (supplied, unconditional)
    return verdicts, traits


def primitive_traits(blocks):
    """The section 1.9 primitive block, as {name: {trait, ...}} (PG4, PG19).

    **Each line holds one colon token**, the names stand to its left, and the
    traits every one of those names supplies stand to its right.
    """
    supplied = {}
    for line in blocks["primitive-traits"]:
        halves = line.split(":")
        if len(halves) != 2:
            continue
        traits = {token for token in halves[1].split() if token in CLOSURE_TRAITS}
        for name in halves[0].split():
            supplied[name] = set(traits)
    return supplied


def primitive_sizes(blocks):
    """The section 1.9 primitive-size block, as {name: (size, align)} (PG24)."""
    sizes = {}
    for line in blocks["primitive-sizes"]:
        parts = line.split()
        if len(parts) != 3 or not parts[1].isdigit() or not parts[2].isdigit():
            continue
        sizes[parts[0]] = (int(parts[1]), int(parts[2]))
    return sizes


def audio_owned(blocks, owned):
    """The section 5.7 audio-owned set, as `(names, bad tokens)` (PG26).

    Every token must be a name the section 1.5 table places. A token that is
    not one is a parse that found prose, and PG26 fails closed on it rather
    than scanning a set of words (critic CR-14).
    """
    names, bad = set(), []
    for token in token_rows(blocks["audio-owned"]):
        if token in owned:
            names.add(token)
        else:
            bad.append(token)
    return names, bad


def audio_exempt(blocks, owned):
    """The section 5.7 exemption block, as `({(type, field): head}, bad)`.

    Each line is one `Type.field` path, then the HEAD type the row exempts,
    then a reason token. The row answers the FREE half of PG26 for that one
    field, and it answers neither PG26b nor PG26c.

    **The head is part of the row** (critic CR on the exemption). Revision 15
    keyed a row on `Type.field` alone, so the row for `GraphState.resets`
    covered whatever that field held; a Critic probe put a bare `Box<[u8]>`
    there and the run exited 0. A row now exempts one head, and a field whose
    head is anything else falls through to the heap test.
    """
    paths, bad = {}, []
    for line in blocks["audio-exempt"]:
        parts = line.split()
        if len(parts) < 3 or "." not in parts[0]:
            bad.append(line.strip())
            continue
        holder, _dot, field = parts[0].partition(".")
        head = parts[1]
        if holder not in owned or not field or not head[:1].isupper():
            bad.append(line.strip())
            continue
        paths[(holder, field)] = head
    return paths, bad


def audio_marked(source):
    """Every declaration whose doc comment carries the audio-owned marker.

    **This is the SECOND source of the root set** (critic C15-3). The block
    alone is one source, and `declared-name` can only ask whether a row names
    a type, so a one-for-one swap took `Transport` out of all three TH1 rules
    with every counter at the baseline. A marker in the declaration's own doc
    comment answers the other direction: a swap now loses a marked name from
    the block and gains an unmarked one, and each half is its own failure.
    """
    marked = set()
    for _offset, code in rust_blocks(source):
        doc = []
        for line in code.splitlines():
            stripped = line.strip()
            if stripped.startswith("///"):
                doc.append(stripped)
                continue
            match = re.match(
                r"^(?:pub(?:\([a-z]+\))?\s+)?(?:struct|enum)\s+([A-Z][A-Za-z0-9]*)", stripped
            )
            if match:
                if any(AUDIO_MARK in line for line in doc):
                    marked.add(match.group(1))
                doc = []
            elif stripped.startswith("#[") or stripped.startswith(")]") or not stripped:
                continue
            else:
                doc = []
    return marked


def audio_root_audit(roots, marked):
    """PG26d. The block and the marked set are one set, in both directions.

    **The rule's own limit, stated here** (critic C16-W1). The two sources it
    compares are two hand-written copies of ONE judgement, in one document,
    by one author, in one changeset. Holding them consistent tests
    self-consistency and not truth. **A coordinated edit of both defeats it**:
    the Critic moved `Transport` out of the block and `ChannelConfig` in, moved
    the marker line with them, and the run stayed green with a `Vec<u8>` on the
    swapped-out type. Four lines. PG26d raises the cost of a disarm from one
    line to four and adds no independent oracle of its own. PG26e is the
    independent oracle, and it covers the reachable part of the set.
    """
    failures = []
    for name in sorted(set(roots) - marked):
        failures.append((name, "the block names it and its declaration carries no marker"))
    for name in sorted(marked - set(roots)):
        failures.append((name, "its declaration carries the marker and the block omits it"))
    return failures


# The audio thread holds ONE value, and that value is the `Box<dyn
# AudioProcess>` the backend moved into its callback state. `GraphState` is a
# field of it. Revision 20 rooted this walk at `GraphState`, so six
# heap-owning handles above that field were outside every audio rule (critic
# C20-3).
AUDIO_ROOT_OF_ROOTS = "EngineProcess"


def audio_reachable(raw_decls, arms):
    """Every declared type `GraphState` reaches through a declared field.

    The set is DERIVED from the declaration bodies. It is the third source
    PG26e needs: neither the `audio-owned` block nor the `**Audio-owned**`
    marker contributes one name to it, so a coordinated edit of those two
    cannot move it (critic C16-W1).
    """
    seen, stack = set(), [AUDIO_ROOT_OF_ROOTS]
    while stack:
        name = stack.pop()
        if name in seen:
            continue
        seen.add(name)
        if name not in raw_decls:
            continue
        for _field, expression in walk_fields(name, raw_decls, arms):
            for leaf in re.findall(r"\b([A-Z][A-Za-z0-9_]*)\b", expression):
                if leaf in raw_decls and leaf not in seen:
                    stack.append(leaf)
    return seen & set(raw_decls)


def audio_asserted_audit(roots, reachable, asserted):
    """PG26f. The third source of the audio-owned root set (critic C17-W8).

    PG26d compares the `audio-owned` block with the `**Audio-owned**` markers,
    and the sixteenth Critic defeated it by moving both in one four-line edit.
    PG26e derives 35 of the 40 roots from the declaration bodies below
    `EngineProcess`, and the seventeenth Critic showed the same edit is still
    green on the roots the walk does not reach, `Transport` among them. This rule makes those eleven a THIRD
    source: the `audio-asserted` block names each one with the reason it is on
    the audio thread, and the rule holds that block and the root set to one
    set outside the reachable part.

    Both directions run. A root the closure does not reach and this block does
    not name is a failure, so a root cannot be added without a reason. A name
    this block holds that the root set omits is a failure, so the four-line
    edit at `Transport` is red: it moves two sources and leaves this one.

    **The rule's own limit, stated here.** This block is an assertion, exactly
    as the other two are, and no derivation is available for an argument type
    because the document does not write `GraphRunner::run`'s body. What the
    rule adds is a third place to edit and a stated reason per row.
    """
    failures = []
    for name in sorted(set(roots) - reachable - set(asserted)):
        failures.append(
            (name, "the root set names it, the closure does not reach it, and the"
                   " asserted block does not name it")
        )
    for name in sorted(set(asserted) - set(roots)):
        failures.append(
            (name, "the asserted block names it and the audio-owned block omits it")
        )
    return failures


def audio_closure_audit(roots, reachable, leaves):
    """PG26e. The audio-owned set is closed under reachability.

    The audio thread holds one value, `GraphState`. Everything that value
    reaches through a declared field is on the audio thread by construction,
    so every reachable declaration is an audio-owned root or a leaf the
    `audio-reachable-leaf` block names with a reason. **A root that leaves the
    block while the reachability holds is red here even when the marker
    leaves with it**, which is the coordinated edit PG26d cannot see.

    **The rule's own limit, stated here.** It is one-directional and it
    covers the reachable part of the root set only. A root the audio thread
    touches through a value that is not a declared field of `EngineProcess` is
    outside it, and five of the forty roots are in that class today:
    the two triple-buffer publications, the two MIDI record types, the
    transport machine, and the cycle types each reach the audio thread as a
    function argument or a published value. `Transport`, moved out of the
    block with its marker, is one of the eleven and stays green. PG26d's own
    site states that residual limit.
    """
    failures = []
    for name in sorted(reachable - set(roots) - set(leaves)):
        failures.append(
            (
                name,
                f"{AUDIO_ROOT_OF_ROOTS} reaches it through a declared field and it is"
                " neither an audio-owned root nor an audio-reachable leaf",
            )
        )
    for name in sorted(set(leaves) - reachable):
        failures.append(
            (name, f"the leaf block names it and {AUDIO_ROOT_OF_ROOTS} does not reach it")
        )
    return failures


def floor_audit(source, blocks):
    """PG27b. A floor BELOW its own block's row count is a failure.

    Revision 20 added `LimiterState` to the `audio-owned` block and left the
    floor at 37 against 38 rows, so one deletion passed as a clean run and the
    twentieth Critic measured it (critic C20-W10). Twelve blocks carried the
    same slack, and the two widest were the probe table and the rule-block
    table, which are the two that describe the guard set itself (critic
    C20-N5). The floor is therefore EQUAL to the row count and never below it,
    so an addition raises the floor in the same changeset and a deletion is
    always red.

    **It runs AFTER the membership rules and it reports a finding, not a
    fail-closed input failure.** A block that is one row over its floor is a
    document defect and not an unreadable input, and a membership probe that
    adds one row must still be answered by the membership rule.

    `roadmap/duet-v1/tools/sync_floors.py` writes the three copies of each
    number, so no floor is typed by hand.
    """
    failures = []
    for block_id, rows in blocks.items():
        minimum = DATA_BLOCKS[block_id][2]
        if len(rows) > minimum:
            failures.append(
                (
                    block_id,
                    f"the block holds {len(rows)} rows and the stated floor is"
                    f" {minimum}; a floor below the row count grants one free deletion"
                    " per addition",
                )
            )
    return failures


def drop_impl_names(blocks, owned):
    """The section 1.9 Drop block, as `(names, bad)` (PG9, PG21).

    A type with a hand-written `Drop` impl cannot implement `Copy`, so
    `missing_copy_implementations` never fires on it and an expectation there
    is a build error (critic WR-12).
    """
    names, bad = set(), []
    for token in token_rows(blocks["drop-impls"]):
        if token in owned:
            names.add(token)
        else:
            bad.append(token)
    return names, bad


def heap_externals(blocks):
    """Every external name whose section 1.9 `Why` cell states a heap (PG26)."""
    names = set()
    for cells in blocks["external-verdicts"]:
        if len(cells) < 4 or not re.search(r"\bheap\b", cells[3], re.I):
            continue
        names |= set(cell_names(cells[0]))
    return names


def deferring_externals(blocks):
    """Every external name whose 1.9 `Why` cell states a deferred free (PG26).

    `basedrop::Owned` and `basedrop::Shared` each move the free of everything
    below them to the collector thread (TH5), so a heap handle inside one is
    legal on the audio thread. The set is data in the document and not a set
    inside this machine (critic CR-16, critic K-4).
    """
    names = set()
    for cells in blocks["external-verdicts"]:
        if len(cells) < 4 or not re.search(r"\bdefers\b", cells[3], re.I):
            continue
        names |= set(cell_names(cells[0]))
    return names


def grow_externals(blocks):
    """Every external name whose 1.9 `Why` cell states that it grows (PG26b).

    A growable container calls the allocator after it is built, so it is
    illegal on the audio thread whatever wrapper holds it. Revision 14 let one
    pass inside a `basedrop::Owned`, because the wrapper answered the free
    half of TH1 and nothing read the allocation half (critic WR-22).
    """
    names = set()
    for cells in blocks["external-verdicts"]:
        if len(cells) < 4 or not re.search(r"\bgrows\b", cells[3], re.I):
            continue
        names |= set(cell_names(cells[0]))
    return names


def read_externals(blocks):
    """Every external name whose 1.9 `Why` cell states a read end (PG26b).

    A `triple_buffer::Output` hands the audio thread a SHARED reference to
    the published value. Growing a container needs an exclusive reference, so
    no container below a read end can grow on this thread, and PG26b stops
    there. **PG26 and PG26c do not stop there**: the read end itself owns a
    heap allocation, which is why it sits inside a `basedrop::Owned`, and a
    lock needs only a shared reference, so the lock test descends (critic
    C20-3).
    """
    names = set()
    for cells in blocks["external-verdicts"]:
        if len(cells) < 4 or not re.search(r"\breads\b", cells[3], re.I):
            continue
        names |= set(cell_names(cells[0]))
    return names


def lock_externals(blocks):
    """Every external name whose 1.9 `Why` cell states a lock (PG26c).

    TH1 bans a lock on the audio thread. PG26 read the heap half of that rule
    alone, so a `Mutex` field inside an audio-owned declaration passed every
    guard (critic WR-23).
    """
    names = set()
    for cells in blocks["external-verdicts"]:
        if len(cells) < 4 or not re.search(r"\block\b", cells[3], re.I):
            continue
        names |= set(cell_names(cells[0]))
    return names


def expr_head(text):
    """The outermost head name of a field type expression, or None (PG26)."""
    head = text.strip()
    if head.startswith("&") or head.startswith("[") or head.startswith("("):
        return None
    head = head.split("<", 1)[0].strip()
    head = head.rsplit("::", 1)[-1].strip()
    return head or None


def external_path_names(blocks):
    """Every name the section 1.9 external-path block maps (PG27, PG25)."""
    names = set()
    for line in blocks["external-paths"]:
        parts = line.split()
        if len(parts) >= 2:
            names.add(parts[0])
    return names


def framework_names(blocks):
    """The framework type list that section 1.3 declares."""
    return set(token_rows(blocks["framework-types"]))


def name_map(blocks, rows):
    """The section 1.2 third-party name map, as `({token: crate}, bad)`.

    **It fails closed on a row it cannot resolve** (critic WR-16). A line that
    is not one name and one crate, or that names a crate no section 1.2 row
    carries, is a damaged row rather than a mapping the guard may skip.
    """
    known = {crate for entry in rows.values() for crate in entry}
    mapping, bad = {}, []
    for line in blocks["name-map"]:
        parts = line.split()
        if len(parts) != 2:
            bad.append(line.strip())
            continue
        if known and parts[1] not in known:
            bad.append(line.strip())
            continue
        mapping[parts[0]] = parts[1]
    return mapping, bad


def section_of(source):
    """A list of (offset, section number) for every `### N.N` heading."""
    marks = []
    for match in re.finditer(r"(?m)^### (\d+\.\d+[a-z]?) ", source):
        marks.append((match.start(), match.group(1)))
    return marks


def section_at(marks, position):
    """The section number that contains `position`, or None."""
    current = None
    for start, number in marks:
        if start <= position:
            current = number
        else:
            break
    return current


def rust_blocks(source):
    """Every fenced Rust block, as (offset, code)."""
    return [(m.start(), m.group(1)) for m in re.finditer(r"```rust\n(.*?)```", source, re.S)]


def strip_comments(code):
    """Code with every line comment removed; a name in a comment is prose."""
    return re.sub(r"//[^\n]*", "", code)


def strip_block_comments(code):
    """Code with every block comment removed, such as `/* private */`."""
    return re.sub(r"/\*.*?\*/", " ", code, flags=re.S)


def strip_paths(code):
    """Code with every `::Name` segment removed.

    `TimeError::NotFinite` names a variant of a placed type, and
    `basedrop::Owned` names a third-party type behind its own path.
    """
    return re.sub(r"::[A-Z][A-Za-z0-9]*", "::", code)


def matching_brace(code, open_index):
    """The index of the brace that closes the one at `open_index`."""
    depth, index = 0, open_index
    while index < len(code):
        if code[index] == "{":
            depth += 1
        elif code[index] == "}":
            depth -= 1
            if depth == 0:
                return index
        index += 1
    return len(code)


def top_level_arms(body):
    """Each top-level arm of an enum body, as raw text."""
    depth, start, out = 0, 0, []
    for index, char in enumerate(body):
        if char in "({[":
            depth += 1
        elif char in ")}]":
            depth -= 1
        elif char == "," and depth == 0:
            out.append(body[start:index])
            start = index + 1
    out.append(body[start:])
    return [arm for arm in out if arm.strip()]


def mask_arms(body):
    """An enum body with the leading identifier of each top-level arm removed.

    An arm name is not a type. `StripKind::Track` must not make the guard read
    the struct `Track` as a field of `StripKind` (PG8).
    """
    return ",".join(
        re.sub(r"^(\s*)[A-Z][A-Za-z0-9]*", r"\1", arm, count=1) for arm in top_level_arms(body)
    )


def declarations(code):
    """Every declaration in one block: name -> [(kind, payload, attributes)].

    `payload` is the declaration body with enum arm names masked, so it holds
    field types and nothing else.

    A tuple body opens before the next `{` **and before the next `;`**. A unit
    struct such as `pub(crate) struct EnterCompose;` therefore yields an empty
    body. Revision 10 compared the parenthesis with the brace alone, so with
    no brace left in the block it read the `pub(crate)` of the NEXT
    declaration as a tuple body and gave four action structs one field named
    `crate` (critic C-6).
    """
    found = {}
    for match in re.finditer(
        # The multi-line alternative may not cross a `;`, a `{` or a `}`,
        # so backtracking cannot make one `#[must_use]` far above swallow the
        # code between it and the next declaration. Revision 23 declared two
        # structs after a run of attributed method signatures and the parser
        # did not see either one (critic C23I-1).
        r"((?:#\[[^\]]*\]\s*|#\[[^;{}]*?\]\s*)*)"
        r"(?:pub(?:\([a-z]+\))?\s+)?(struct|enum|trait)\s+([A-Z][A-Za-z0-9]*)",
        code,
    ):
        attrs, kind, name = match.group(1), match.group(2), match.group(3)
        tail = code[match.end() :]
        body = ""
        brace = tail.find("{")
        semi = tail.find(";")
        paren = tail.find("(")
        stops = [mark for mark in (brace, semi) if mark >= 0]
        limit = min(stops) if stops else len(tail)
        if kind == "struct" and 0 <= paren < limit:
            close = tail.find(")")
            body = tail[paren + 1 : close if close >= 0 else len(tail)]
        elif brace >= 0 and (semi < 0 or brace < semi):
            absolute = match.end() + brace
            body = code[absolute + 1 : matching_brace(code, absolute)]
        payload = mask_arms(body) if kind == "enum" else body
        found.setdefault(name, []).append((kind, payload, attrs))
    return found


def hand_impls(source):
    """Every `impl Trait for Type` this document writes by hand.

    `Finite` carries five hand-written impls, because a derive over an `f64`
    field does not compile (section 2.6a). PG19 and PG22 read them, so a hand
    impl counts exactly as a derive does.
    """
    found = {}
    for _offset, block in rust_blocks(source):
        code = strip_comments(block)
        for match in re.finditer(
            r"\bimpl\b(?:\s*<[^>]*>)?\s+([A-Z][A-Za-z0-9]*)(?:\s*<[^>]*>)?\s+for\s+([A-Z][A-Za-z0-9]*)",
            code,
        ):
            trait_name, type_name = match.group(1), match.group(2)
            if trait_name in CLOSURE_TRAITS:
                found.setdefault(type_name, set()).add(trait_name)
    return found


def mask_enum_arm_names(code):
    """Code with the leading identifier of each top-level enum arm removed.

    An arm name is masked only inside the enum that declares it (PG8). The
    guard keeps no global variant set, so a struct named `Reverb`, `Track`, or
    `Peak` is still a candidate.
    """
    out = code
    for match in reversed(list(re.finditer(r"\benum\s+[A-Z][A-Za-z0-9]*\s*\{", out))):
        open_index = out.index("{", match.start())
        close_index = matching_brace(out, open_index)
        body = out[open_index + 1 : close_index]
        out = out[: open_index + 1] + mask_arms(body) + out[close_index:]
    return out


def used_types(code):
    """Every name in a type position."""
    used = set()
    for name in re.findall(
        r"(?::|->|impl|<|,)\s*&?(?:mut\s+)?(?:'[a-z]+\s+)?([A-Z][A-Za-z0-9]*)", code
    ):
        used.add(name)
    for name in re.findall(r"\b([A-Z][A-Za-z0-9]*)\s*<", code):
        used.add(name)
    return used


def body_type_names(body, known=frozenset()):
    """Every type name a struct body or a masked enum body mentions.

    An all-upper token that neither the 1.5 table nor the external table
    names is a constant, such as `MAX_STRIPS`, and not a type. `I24` is
    placed, so it stays (critic R2).
    """
    cleaned = strip_paths(strip_comments(body))
    return {
        name
        for name in re.findall(r"\b([A-Z][A-Za-z0-9]*)\b", cleaned)
        if name in known or not name.isupper() or len(name) == 1
    }


def last_segments(body):
    """The last path segment of every type-shaped token in a body.

    `gpui_kit::Px` gives `Px`, and a bare `Px` gives `Px`. PG7 runs on the
    text before `strip_paths`, so a qualified spelling cannot hide a framework
    name (critic N8).
    """
    cleaned = strip_comments(body)
    names = set()
    for match in re.finditer(
        r"(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*([A-Z][A-Za-z0-9]*)\b", cleaned
    ):
        name = match.group(1)
        if not name.isupper() or len(name) == 1:
            names.add(name)
    return names


def ownership_table(blocks):
    """The section 1.5 table as ({type: crate}, {crate: {section, ...}})."""
    owned, declared_in = {}, {}
    for cells in blocks["ownership-table"]:
        if len(cells) < 2:
            continue
        crate = cells[0].strip("`")
        for name in cell_names(cells[1]):
            owned[name] = crate
        if len(cells) >= 3:
            declared_in[crate] = set(re.findall(r"\d+\.\d+[a-z]?", cells[2]))
    return owned, declared_in


def dependency_table(blocks):
    """The section 1.2 crate table as {crate: {third-party crate, ...}}."""
    rows = {}
    for cells in blocks["crate-table"]:
        if len(cells) < 4:
            continue
        crate = cells[0].strip("`")
        if not crate.startswith("duet"):
            continue
        # PG12 reads code spans only, so prose in the cell proves nothing
        # and cannot stand in for a dependency (critic Q15).
        rows[crate] = set(re.findall(r"`([A-Za-z_][A-Za-z0-9_-]*)`", cells[3]))
    return rows


def edge_list(blocks):
    """The section 1.3 edge list as {crate: {dependency, ...}}."""
    edges = {}
    current = None
    for line in blocks["edge-list"]:
        if "->" in line:
            left, right = line.split("->", 1)
            current = left.strip()
            edges.setdefault(current, set())
        elif current is None:
            continue
        else:
            right = line
        for name in re.findall(r"[a-z][a-z-]*", right):
            if name.startswith("duet"):
                edges[current].add(name)
    return edges


def reachable(edges):
    """The transitive closure of the section 1.3 graph, as {crate: {crate}}."""
    closure = {}

    def walk(crate, seen):
        for target in edges.get(crate, set()):
            if target in seen:
                continue
            seen.add(target)
            walk(target, seen)
        return seen

    for crate in set(edges) | {t for targets in edges.values() for t in targets}:
        closure[crate] = walk(crate, set())
    return closure


def normalize_crate(name):
    """The 1.3 spelling of a 1.5 crate name."""
    return "duet" if name == "crates/duet" else name


def edge_claims(source, edges):
    """Every prose sentence that claims an edge the list does not carry."""
    failures = []
    fences = [(m.start(), m.end()) for m in re.finditer(r"```.*?```", source, re.S)]

    def inside_fence(position):
        return any(start <= position < end for start, end in fences)

    gap = r"(?:(?!\. )[^`;\n])"
    pattern = re.compile(
        r"`(duet(?:-[a-z]+)*|crates/duet)`(" + gap + r"{0,40}?)"
        r"\b(" + "|".join(EDGE_VERBS) + r")\b"
        r"(" + gap + r"{0,60}?)`(duet(?:-[a-z]+)*|crates/duet)`"
    )
    for match in pattern.finditer(source):
        if inside_fence(match.start()):
            continue
        # A negation on either side of the verb cancels the claim (critic C3).
        if re.search(r"\b(no|not|never)\b", match.group(2) + " " + match.group(4)):
            continue
        subject = normalize_crate(match.group(1))
        target = normalize_crate(match.group(5))
        if subject == target:
            continue
        # An absent subject carries the empty edge set, not a free pass.
        known = edges.get(subject, set())
        if target not in known:
            failures.append((subject, target, match.group(0).strip()))
    return failures


def copy_verdicts(decls, externals, impls):
    """Copy-ness of every name the guard can decide.

    **There is no suffix template.** Revision 8 seeded a name that ends in
    `Id`, `Name`, or `Text` from ID1, which told the chunk that `CommitId` is
    a `u64` and made the guard certify it (critic R1). Every verdict now comes
    from one of two places: a Rust block in this document, or the
    external-verdict table of section 1.9.
    """
    is_copy = {}
    for name, verdict in externals.items():
        if verdict == "copy":
            is_copy[name] = True
        elif verdict == "not copy":
            is_copy[name] = False
    for name, entries in decls.items():
        kind, _payload, attr = entries[0]
        if kind == "trait":
            continue
        is_copy[name] = bool(re.search(r"#\[derive\([^)]*\bCopy\b", attr)) or "Copy" in impls.get(
            name, set()
        )
    return is_copy


def field_names(payload, externals, known):
    """The field type names of one declaration.

    **There is no drop list.** Only a wrapper the external table marks
    `transparent` is skipped, because its Copy-ness is its payload's and the
    payload is a separate token in the same field (critic R2).
    """
    return {
        token
        for token in body_type_names(payload, known)
        if externals.get(token) != "transparent"
    }


def derives_of(attr):
    """The trait names one declaration's `#[derive(...)]` attributes carry."""
    names = set()
    for match in re.finditer(r"#\[derive\(([^)]*)\)\]", attr, re.S):
        names |= set(re.findall(r"[A-Za-z_][A-Za-z0-9_]*", match.group(1)))
    return names


def split_top(text, separator=","):
    """Split `text` on `separator` at bracket depth zero."""
    depth, start, out = 0, 0, []
    for index, char in enumerate(text):
        if char in "({[<":
            depth += 1
        elif char in ")}]>":
            depth -= 1
        elif char == separator and depth == 0:
            out.append(text[start:index])
            start = index + 1
    out.append(text[start:])
    return [piece for piece in out if piece.strip()]


def field_colon(text):
    """The index of the `:` that separates a field name from its type.

    **A path separator is not a field separator.** `split_top(piece, ":")`
    cut `probe: parking_lot::Mutex<u32>` into three pieces and rejoined two
    of them, which produced `parking_lot:Mutex<u32>`; `parse_type` then
    refused it and every rule that reads the field saw nothing, so a lock
    behind a crate path passed the whole guard. The scanner below skips a
    `::` pair and takes the first single colon at bracket depth zero.
    """
    depth = 0
    index = 0
    while index < len(text):
        char = text[index]
        if char in "({[<":
            depth += 1
        elif char in ")}]>":
            depth -= 1
        elif char == ":" and depth == 0:
            if index + 1 < len(text) and text[index + 1] == ":":
                index += 2
                continue
            return index
        index += 1
    return None


def field_exprs(kind, payload):
    """Every (field name, type expression) of one declaration body.

    The body comes from the raw block, so a path keeps its segments and PG19
    resolves it by the last one. A `/* private */` body yields nothing, and a
    declaration the document does not spell out is outside PG19 and PG20.
    """
    body = strip_block_comments(strip_comments(payload))
    out = []

    def named(text):
        for piece in split_top(text):
            piece = re.sub(r"#\[[^\]]*\]", " ", piece).strip()
            piece = re.sub(r"^pub(?:\([a-z]+\))?\s+", "", piece).strip()
            if not piece:
                continue
            cut = field_colon(piece)
            if cut is None:
                out.append(("", piece))
            else:
                out.append((piece[:cut].strip(), piece[cut + 1 :].strip()))

    if kind == "enum":
        for arm in split_top(body):
            arm = arm.strip()
            if arm.startswith("(") and arm.endswith(")"):
                named(arm[1:-1])
            elif arm.startswith("{") and arm.endswith("}"):
                named(arm[1:-1])
        return out
    named(body)
    return out


def parse_type(text):
    """One type expression as a node, or None when the guard cannot read it.

    A node is `("name", name, [arg, ...])`, `("array", inner)`,
    `("slice", inner)`, `("tuple", [item, ...])`, or `("exclusive", inner)`.

    **A shared reference and an exclusive reference are two types.** `&T` is
    `Copy` and forwards every closure trait to its payload, which is why the
    guard reds a `Copy` derive over a `&'a str` field and calls that a loud
    false red (critic K-6). `&mut T` is never `Copy`, whatever `T` is, and it
    supplies none of the nine closure traits. Revision 11 stripped `&`, `&'a`,
    and `&mut` alike, so `Cycle` read as an all-`Copy` type and PG9 asked a
    declaration that holds `&'buffers mut [f32]` to derive `Copy`, which does
    not compile.

    A `dyn` or an `impl` keyword drops and the trait path stays, so
    `Box<dyn AudioProcess>` yields the name `AudioProcess` and PG20 can place
    it. Each bound of a `+` list becomes its own name, so `dyn Trait + Send`
    never mangles into one name and a `'static` bound yields none. Revision 11
    read neither keyword, so the argument parsed as None, `leaf_names` saw no
    trait, and the Cargo cycle PG20 exists to catch passed in silence
    (critic W-4).
    """
    text = text.strip()
    reference = re.match(r"^&\s*(?:'[a-z_][a-z0-9_]*\s*)?(mut\s+)?", text)
    if reference and reference.group(0):
        inner = parse_type(text[reference.end() :])
        return ("exclusive", inner) if reference.group(1) else inner
    if not text:
        return None
    if re.match(r"^(?:dyn|impl)\b", text):
        bounds = [
            piece.strip()
            for piece in split_top(re.sub(r"^(?:dyn|impl)\b\s*", "", text), "+")
            if piece.strip() and not piece.strip().startswith("'")
        ]
        if not bounds:
            return None
        if len(bounds) == 1:
            return parse_type(bounds[0])
        return ("tuple", [parse_type(bound) for bound in bounds])
    if text.startswith("[") and text.endswith("]"):
        inner = text[1:-1]
        halves = split_top(inner, ";")
        if len(halves) == 2:
            return ("array", parse_type(halves[0]))
        return ("slice", parse_type(inner))
    if text.startswith("(") and text.endswith(")"):
        items = split_top(text[1:-1])
        if not items:
            return ("tuple", [])
        if len(items) == 1:
            return parse_type(items[0])
        return ("tuple", [parse_type(item) for item in items])
    match = re.match(
        r"^(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*([A-Za-z_][A-Za-z0-9_]*)\s*(<.*>)?$", text, re.S
    )
    if not match:
        return None
    name = match.group(1)
    args = []
    if match.group(2):
        for piece in split_top(match.group(2)[1:-1]):
            bare = piece.strip()
            if not bare or bare.startswith("'"):
                continue
            # A const-generic argument is a value, not a type. `MAX_SLOTS` in
            # `ArrayVec<SlotSpec, MAX_SLOTS>` is a section 1.6 constant.
            if bare.isupper() or bare[0].isdigit() or bare.startswith("{"):
                continue
            args.append(parse_type(piece))
    return ("name", name, args)


def leaf_names(node):
    """Every bare name a parsed type expression mentions."""
    if node is None:
        return set()
    if node[0] == "name":
        names = {node[1]}
        for arg in node[2]:
            names |= leaf_names(arg)
        return names
    if node[0] in ("array", "slice", "exclusive"):
        return leaf_names(node[1])
    names = set()
    for item in node[1]:
        names |= leaf_names(item)
    return names


def holds_exclusive(node):
    """Whether one parsed type expression reaches an exclusive reference.

    PG9 and PG10 read a field by name and never by expression, so the
    reference form reaches them here. A type that holds a `&mut` at any depth
    cannot derive `Copy`.
    """
    if node is None:
        return False
    if node[0] == "exclusive":
        return True
    if node[0] == "name":
        return any(holds_exclusive(arg) for arg in node[2])
    if node[0] in ("array", "slice"):
        return holds_exclusive(node[1])
    return any(holds_exclusive(item) for item in node[1])


def node_traits(node, resolve):
    """The closure traits one type expression supplies, or None when unknown."""
    if node is None:
        return None
    if node[0] == "exclusive":
        return set()
    if node[0] == "tuple":
        supplied = set(CLOSURE_TRAITS)
        for item in node[1]:
            inner = node_traits(item, resolve)
            if inner is None:
                return None
            supplied &= inner
        return supplied
    if node[0] == "array":
        inner = node_traits(node[1], resolve)
        return None if inner is None else set(inner)
    if node[0] == "slice":
        inner = node_traits(node[1], resolve)
        return None if inner is None else set(inner) - {"Default"}
    name, args = node[1], node[2]
    own = resolve(name)
    if own is None:
        return None
    supplied, unconditional = own
    if not args:
        return set(supplied)
    result = set()
    for trait in supplied:
        if trait in unconditional:
            result.add(trait)
            continue
        ok = True
        for arg in args:
            inner = node_traits(arg, resolve)
            if inner is None:
                return None
            if trait not in inner:
                ok = False
                break
        if ok:
            result.add(trait)
    return result


def make_resolver(decls, impls, external_traits, primitives):
    """A `name -> (traits, unconditional)` lookup for PG19.

    `primitives` comes from the section 1.9 block, never from a constant, so
    an empty map decides no primitive and every field above one goes undecided
    (critic K-4).
    """

    def resolve(name):
        if name in primitives:
            return (primitives[name], primitives[name])
        if name in decls:
            kind, _payload, attr = decls[name][0]
            if kind == "trait":
                return None
            supplied = {t for t in derives_of(attr) if t in CLOSURE_TRAITS}
            supplied |= impls.get(name, set())
            return (supplied, supplied)
        if name in external_traits:
            return external_traits[name]
        return None

    return resolve


def derive_closure_audit(raw_decls, owned, impls, external_traits, framework, primitives):
    """PG19. A derive that reaches a field type without the same trait.

    The rule reads a **derive**, because only a derive demands the trait of
    every field. A hand-written impl supplies the trait and demands nothing,
    which is the whole reason `Finite` carries five of them (section 2.6a).

    The rule covers the nine traits of `CLOSURE_TRAITS`. **Three classes sit
    outside it.** A declaration whose body this document writes
    `/* private */` has no field to read. A framework type inside an
    application declaration carries no verdict, because PG7 exempts the
    application rows and section 1.9 holds the one justified unknown. A
    `Default` derive over an array longer than thirty-two is read as the
    element's `Default`, and the standard library stops at thirty-two; this
    document holds one long array and it carries a named constructor rather
    than a derive (section 5.6). The compiler is the backstop for all three.
    """
    resolve = make_resolver(raw_decls, impls, external_traits, primitives)
    broken, undecided = [], []
    for name in sorted(raw_decls):
        kind, payload, attr = raw_decls[name][0]
        if kind == "trait":
            continue
        derived = {t for t in derives_of(attr) if t in CLOSURE_TRAITS}
        if not derived:
            continue
        crate = owned.get(name, "unplaced")
        for field, expr in field_exprs(kind, payload):
            node = parse_type(expr)
            if crate in APP_ROWS and leaf_names(node) & framework:
                continue
            supplied = node_traits(node, resolve)
            if supplied is None:
                undecided.append((name, crate, field or expr.strip(), expr.strip()))
                continue
            for trait in sorted(derived - supplied):
                broken.append((name, crate, trait, field or expr.strip(), expr.strip()))
    return broken, undecided


def reachability_audit(raw_decls, owned, closure):
    """PG20. A field type in a crate the 1.3 graph does not reach.

    **The rule reaches a field type only when `parse_type` reads it, and the
    parser still declines four forms.** A function pointer such as
    `fn(Ticks) -> Span`, a closure trait with a parenthesized argument list
    such as `Fn(Ticks)`, a qualified path such as `<Score as Document>::Id`,
    and an associated type of a generic parameter each yield None, so no name
    reaches the graph. This document writes none of the four in a field
    position, and the compiler is the backstop for all four, because a name
    from a crate with no edge does not build (critic W-4).

    The rule reads the graph, not the member manifest, so a type reached
    through a transitive edge passes. The manifest is the compiler's business,
    and section 1.3 states the rule that every named crate is a direct edge.
    """
    failures = []
    for name in sorted(raw_decls):
        kind, payload, _attr = raw_decls[name][0]
        if kind == "trait":
            continue
        crate = normalize_crate(owned.get(name, ""))
        if not crate:
            continue
        seen = set()
        for _field, expr in field_exprs(kind, payload):
            seen |= leaf_names(parse_type(expr))
        for token in sorted(seen):
            target = normalize_crate(owned.get(token, ""))
            if not target or target == crate:
                continue
            if target not in closure.get(crate, set()):
                failures.append((crate, name, token, target))
    return failures


def copy_audit(decls, owned, is_copy, externals, known, drops=frozenset()):
    """PG9, PG10, PG10b, and PG14 over the document's own declarations.

    PG14 counts an undecidable field of ANY declaration, whether or not the
    declaration derives `Copy`. Revision 7 counted the PG9 direction only, so
    the blind spot of the rule that closes the revision-6 Critical was itself
    invisible (critic Q2).

    **An exclusive reference decides the declaration on its own.** `&mut T` is
    never `Copy`, so a field that holds one puts the declaration outside PG9
    and makes a `Copy` derive over it a PG10 failure. Revision 11 read `&mut`
    as `&` and asked `Cycle` for a derive the compiler refuses.
    """
    missing, impossible, undecided, unknown = [], [], [], []
    for name, entries in decls.items():
        kind, payload, attr = entries[0]
        if kind == "trait" or not payload.strip():
            continue
        derives_copy = bool(re.search(r"#\[derive\([^)]*\bCopy\b", attr))
        crate = owned.get(name, "unplaced")
        exclusive = [
            field or expr.strip()
            for field, expr in field_exprs(kind, payload)
            if holds_exclusive(parse_type(expr))
        ]
        if exclusive and derives_copy:
            for field in exclusive:
                impossible.append((name, crate, field))
        fields = field_names(payload, externals, known)
        if not fields:
            continue
        verdicts = {field: is_copy.get(field) for field in fields}
        for field, verdict in sorted(verdicts.items()):
            if verdict is None:
                unknown.append((name, crate, field))
        if derives_copy:
            for field, verdict in sorted(verdicts.items()):
                if verdict is False:
                    impossible.append((name, crate, field))
                elif verdict is None:
                    undecided.append((name, crate, field))
            continue
        if re.search(r"missing_copy_implementations", attr):
            continue
        # A hand-written `Drop` impl makes `Copy` impossible in the language,
        # so the lint cannot fire and no derive and no expectation belongs
        # here. Section 1.9's Drop block names every one (critic WR-12).
        if name in drops:
            continue
        if exclusive:
            continue
        if all(verdict is True for verdict in verdicts.values()):
            missing.append((name, crate))
    return missing, impossible, undecided, sorted(unknown)


def placeholder_pieces(payload):
    """Every body piece that a comment stands in for (PG3).

    A piece whose text is empty once the block comments leave it states no
    field, which covers both shapes the rule refuses: a body that holds only
    a comment, and a comment that sits where one field of a list belongs.
    """
    return [
        piece.strip()
        for piece in split_top(payload)
        if "/*" in piece and not strip_block_comments(piece).strip()
    ]


def comment_placeholders(source, owned):
    """PG3. Every declaration whose body states a comment where a field goes.

    The scan reads each Rust block with its LINE comments replaced by an
    empty block comment, so a `///` line cannot declare a phantom type and a
    body that held only a line comment is still visible as a placeholder.
    Revision 11 substituted an empty body for forty-four declarations, and
    PG19, PG23, PG24, and PG26 each read nothing there (critic WR-1).
    """
    failures = set()
    for _offset, block in rust_blocks(source):
        marked = re.sub(r"//[^\n]*", "/**/", block)
        for name, entries in declarations(marked).items():
            for _kind, payload, _attrs in entries:
                if placeholder_pieces(payload):
                    failures.add((name, owned.get(name, "unplaced")))
    return sorted(failures)


def walk_fields(name, raw_decls, arms):
    """Every `(label, expression)` one declaration states, arm names kept.

    `declarations` masks an arm name, because an arm name is not a type
    (PG8). PG26 needs the name for the field chain it prints, so it reads the
    unmasked arm list the way PG24 does.
    """
    kind, payload, _attr = raw_decls[name][0]
    if kind == "trait":
        return []
    if kind != "enum":
        return [(field or expr.strip(), expr) for field, expr in field_exprs(kind, payload)]
    found = []
    for arm_name, arm_body in arms.get(name, []):
        for field, expr in field_exprs("enum", arm_body):
            found.append((arm_name + "." + field if field else arm_name, expr))
    return found


def heap_audit(
    raw_decls,
    owned,
    arms,
    roots,
    heap,
    exempt=frozenset(),
    defer=frozenset(),
    grows=frozenset(),
    locks=frozenset(),
    reads=frozenset(),
):
    """PG26, PG26b, and PG26c. What audio-owned state may not hold.

    TH1 states that the audio thread never allocates, never frees, and never
    locks. Three rules read three halves of that sentence, and each one has
    its own probe (DR5).

    **PG26, the free half.** No type section 5.7 names may own a heap
    allocation that this thread would free. A field whose outermost head is a
    deferring wrapper moves that free to the collector thread (TH5), so the
    heap test is switched off for the whole subtree below it and the field is
    counted on `AUDIO DEFERRED`. Revision 13 hid three `rtrb` ends behind two
    exemption lines, and a Critic probe put a `Vec<u8>` behind one of them and
    got a green run (critic CR-16).

    **PG26b, the allocation half.** A deferring wrapper answers the free and
    it can never answer the allocation, so a container that GROWS fails at any
    depth, a wrapper included. Revision 14 accepted `Owned<Vec<u8>>`, and a
    `push` on that field calls the allocator on the audio thread (critic
    WR-22). A fixed `Box<[T]>` allocates once, off this thread, and passes.

    **PG26c, the lock half.** A lock fails at any depth, a wrapper and an
    exemption line included. PG26 read the heap half alone, so a `Mutex` field
    and an `RwLock` field each passed every guard (critic WR-23).

    **The walk continues through a deferring wrapper** rather than stopping at
    it, because the allocation half and the lock half both have to read what
    the wrapper carries. Only the heap test is suppressed below one.

    **PG26b stops below a READ END, and PG26 and PG26c do not.** A field
    whose outermost head is an external name whose 1.9 `Why` cell carries the
    word "reads" hands this thread a shared reference, and a container needs
    an exclusive reference to grow. The handle itself still owns a heap
    allocation, so it still needs a deferring wrapper, and a lock still fails
    below it because a lock needs only a shared reference (critic C20-3).

    It returns `(heap failures, grow failures, lock failures, deferred,
    read-only fields)`.
    """
    failures, grow_bad, lock_bad, deferred, readonly = [], [], [], [], []

    def walk(root, crate, name, path, seen, inside, published=False):
        if name in seen or name not in raw_decls:
            return
        deeper = seen | {name}
        for field, expr in walk_fields(name, raw_decls, arms):
            step = path + [field]
            names = leaf_names(parse_type(expr))
            # PG26c runs first and it reads no wrapper and no exemption: a
            # lock is illegal wherever the audio thread can reach it.
            if names & locks:
                lock_bad.append((root, crate, expr.strip(), ".".join(step)))
                continue
            # PG26b runs next, and it is the ONE rule a read end stops.
            if names & grows and not published:
                grow_bad.append((root, crate, expr.strip(), ".".join(step)))
                continue
            # An exempt row answers the FREE half of ONE named field whose
            # head is ONE named type. It answers neither PG26b nor PG26c,
            # which is why both run above it.
            #
            # **The head is part of the row** (critic CR on the exemption).
            # Revision 15 keyed the row on `Type.field` alone, so the row for
            # `GraphState.resets` covered whatever that field held: a Critic
            # probe put a bare `Box<[u8]>` there and the run exited 0 with
            # `HEAP IN AUDIO: 0`. The row now names `Arc`, and a field whose
            # head is anything else falls through to the heap test.
            head = expr_head(expr)
            below = inside
            below_published = published
            # The test is on the WHOLE expression and not on the head alone:
            # `Owned<Output<TempoMap>>` reaches `TempoMap` behind the read
            # end, so everything below the field is behind a shared
            # reference whatever wrapper the head names.
            if names & reads:
                readonly.append((root, crate, expr.strip(), ".".join(step)))
                below_published = True
            # An exempt row suppresses the HEAP test for this field and the
            # subtree below it, and it stops nothing. Revision 15 wrote
            # `continue` here, so PG26b and PG26c never read below an exempt
            # field and a Critic probe put a `Mutex` one level down and got a
            # green run.
            if exempt.get((name, field)) == head:
                below = True
            if head is not None and head in defer:
                deferred.append((root, crate, expr.strip(), ".".join(step)))
                below = True
            elif not below and names & heap:
                failures.append((root, crate, expr.strip(), ".".join(step)))
                continue
            for token in sorted(names):
                walk(root, crate, token, step, deeper, below, below_published)

    for root in sorted(roots):
        walk(root, owned.get(root, "unplaced"), root, [], set(), False, False)
    return failures, grow_bad, lock_bad, deferred, readonly


def placed_without_declaration(owned, decls):
    """PG4b. A section 1.5 name that no Rust block declares."""
    return sorted(name for name in owned if name not in decls)


def external_audit(decls, owned, framework, externals, known):
    """PG18. An external token a declaration names and the 1.9 table omits."""
    missing = set()
    for name, entries in decls.items():
        for kind, payload, _attrs in entries:
            if kind == "trait":
                continue
            for token in body_type_names(payload, known):
                if token in owned or token in framework or token in externals:
                    continue
                missing.add((name, token))
    return sorted(missing)


def framework_misuse(raw_decls, owned, framework):
    """PG7. Framework names inside a non-application declaration."""
    failures = []
    for name, entries in raw_decls.items():
        crate = owned.get(name)
        if crate is None or crate in APP_ROWS:
            continue
        for _kind, payload, _attrs in entries:
            for token in sorted(last_segments(payload)):
                if token in framework:
                    failures.append((name, crate, token))
    return failures


def dependency_audit(raw_decls, owned, rows, mapping):
    """PG12. A 1.2 row that omits a crate its own declarations prove it uses.

    It also returns every `(crate, dependency)` pair a declaration PROVES, so
    the run prints that number and section 1.2 cites it rather than a hand
    count (critic WR-14).
    """
    failures, proven = set(), set()
    for name, entries in raw_decls.items():
        crate = owned.get(name)
        # The 1.5 table spells the application crate `crates/duet` and the 1.2
        # table spells it `duet`, so revision 21 skipped EVERY declaration of
        # the application crate and PG12 had never read one. I found this
        # myself while I read the rule, and `HeldDuration(ArrayVec<..>)` is the
        # live instance it hid.
        crate = normalize_crate(crate) if crate else None
        if crate is None or crate not in rows:
            continue
        for _kind, payload, attrs in entries:
            text = strip_comments(attrs) + " " + strip_comments(payload)
            for token in re.findall(r"[A-Za-z_][A-Za-z0-9_]*", text):
                needed = mapping.get(token)
                if not needed:
                    continue
                if needed not in rows[crate]:
                    failures.add((crate, needed, name))
                else:
                    proven.add((crate, needed))
    return sorted(failures), sorted(proven)


def snapshot_audit(blocks, decls, is_copy):
    """PG13. Every type the audio thread publishes is declared and is `Copy`."""
    failures, seen = [], []
    for cells in blocks["snapshot-table"]:
        if len(cells) < 3 or cells[0] != "Audio":
            continue
        found = re.search(r"triple_buffer::Output<([A-Za-z0-9]+)>", cells[2])
        if not found:
            failures.append((cells[2], "no published type name"))
            continue
        name = found.group(1)
        seen.append(name)
        if name not in decls:
            failures.append((name, "no Rust block declares it"))
        elif is_copy.get(name) is not True:
            failures.append((name, "the declaration derives no Copy"))
    return failures, seen


def register_audit(block_sections, owned, declared_in):
    """PG15. A `Declared in` cell that omits a section which declares a type."""
    failures = set()
    for name, section in block_sections.items():
        crate = owned.get(name)
        if crate is None or section is None or crate not in declared_in:
            continue
        if section not in declared_in[crate]:
            failures.add((crate, name, section))
    return sorted(failures)


def justified_unknowns(blocks):
    """The section 1.9 justified-unknown table, as a set of type names."""
    names = set()
    for cells in blocks["justified-unknowns"]:
        if len(cells) >= 4:
            names |= set(cell_names(cells[2]))
    return names


def unknown_audit(copy_unknown, justified):
    """PG17. An undecidable field type the section 1.9 table does not name."""
    return sorted(
        {(name, field) for name, _crate, field in copy_unknown if field not in justified}
    )


def expectation_tables(blocks):
    """The Appendix B.1 expectation tables, as {lint: {type name}} (PG21).

    B.1 holds one registered block per lint, so neither table's position in
    the appendix decides which rule holds it. Revision 11 read the first table
    alone, so the `variant_size_differences` rows had no rule that held them
    and the code to one set (critic concern 3).
    """
    tables = {}
    for lint, block_id in zip(EXPECTED_LINTS, ("b1-copy", "b1-variant")):
        names = tables.setdefault(lint, set())
        for cells in blocks[block_id]:
            for site in re.findall(r"`([A-Za-z0-9_:-]+)`", cells[0]):
                names.add(site.split("::")[-1])
    return tables


def expectation_audit(raw_decls, tables):
    """PG21. The expectation sites and the B.1 tables are one set per lint.

    Both directions and both lints, because B.1 is the list the Orchestrator
    adjudicates: a row nothing carries is a decision nobody needs, and a site
    no row names is a suppression nobody approved (critic W7, concern 3).
    """
    failures = []
    carried = {}
    for lint in EXPECTED_LINTS:
        sites = {
            name
            for name, entries in raw_decls.items()
            if re.search(lint, entries[0][2])
        }
        carried[lint] = sites
        listed = tables.get(lint, set())
        for name in sorted(sites - listed):
            failures.append(
                (name, f"carries a {lint} expectation and Appendix B.1 omits it")
            )
        for name in sorted(listed - sites):
            failures.append(
                (name, f"has an Appendix B.1 {lint} row and no declaration carries it")
            )
    return failures, carried


# A size a reason states: a digit run, an optional space, then the word.
BYTE_COUNT = re.compile(r"([0-9][0-9_]*) ?(bytes?)\b")


def attribute_texts(source, marks):
    """Every `#[expect]` attribute of the document, as `(site, text)` (PG21).

    The site is the item the attribute sits above, which is the name a
    reader needs, and it falls back to the section number when the guard
    cannot read one. The scan reads the whole document, because an attribute
    a Rust block writes and an attribute prose quotes state the same reason.
    """
    found = []
    for match in re.finditer(r"#\s*!?\s*\[\s*expect\b", source):
        depth, cursor = 0, source.index("[", match.start())
        while cursor < len(source):
            if source[cursor] == "[":
                depth += 1
            elif source[cursor] == "]":
                depth -= 1
                if depth == 0:
                    break
            cursor += 1
        if cursor >= len(source):
            continue
        tail = source[cursor + 1 : cursor + 400]
        item = re.search(
            r"\b(?:struct|enum|trait|fn|const|type|mod)\s+([A-Za-z_][A-Za-z0-9_]*)", tail
        )
        site = item.group(1) if item else f"section {section_at(marks, match.start())}"
        found.append((site, source[match.start() : cursor + 1]))
    return found


def b1_reason_cells(blocks):
    """Every Appendix B.1 `(site, reason)` pair (PG21)."""
    found = []
    for block_id in ("b1-convert", "b1-complexity", "b1-copy", "b1-variant"):
        for cells in blocks[block_id]:
            if len(cells) >= 2:
                found.append((cells[0].strip("`"), cells[1]))
    return found


def doc_comment_lines(source):
    """Every `///` and `//!` line of every Rust block, as `(site, text)`.

    PG21 reads a size inside one exactly as it reads a size inside an
    `#[expect]` reason. A declaration doc comment is code that a chunk copies
    into the crate, and revision 12 let one state a size that the compiler
    had already contradicted (critic WR-15).
    """
    found = []
    for offset, block in rust_blocks(source):
        line_number = source[:offset].count("\n") + 1
        for index, line in enumerate(block.splitlines()):
            text = line.strip()
            if text.startswith("///") or text.startswith("//!"):
                found.append((f"a doc comment at line {line_number + index + 1}", text))
    return found


def cites_recorded_row(span, recorded):
    """Whether one code span names a row of the section 1.9 size block."""
    return any(normalize_expr(token) in recorded for token in span.split())


def cited_size(text, match, spans, recorded):
    """Whether one stated size is a citation rather than a literal (PG21).

    A citation writes `B<id> bytes`, or it puts the count beside a `B` id, or
    it sits inside a code span that names a section 1.9 size-block row. Every
    other spelling is a number a reader cannot check.
    """
    lead = text[: match.start()]
    if re.search(r"(?:^|[^0-9A-Za-z_])B$", lead):
        return True
    if re.match(r"[\s,;(]*B[0-9]+\b", text[match.end() :]):
        return True
    return any(
        start <= match.start() and match.end() <= end and cites_recorded_row(span, recorded)
        for start, end, span in spans
    )


def reason_size_audit(sites, recorded):
    """PG21. A size inside a reason is a citation and never a literal.

    Revision 11 compared names only, so a reason that read "the type is 104
    bytes" changed to "9000 bytes" and all three guards stayed green, under a
    heading that states every size there is measured (critic WR-2). A reason
    now writes a `B` id or names the row the PG24 table prints, so every
    stated size rests on a value a guard measured.
    """
    counted, failures = 0, []
    for site, text in sites:
        spans = [
            (span.start(), span.end(), span.group(1))
            for span in re.finditer(r"`([^`]*)`", text)
        ]
        for match in BYTE_COUNT.finditer(text):
            counted += 1
            if cited_size(text, match, spans, recorded):
                continue
            failures.append((site, match.group(1)))
    return counted, failures


def vr1_table(blocks):
    """The section 3.5 VR1 derive-use table, as a set of type names (PG22)."""
    names = set()
    for cells in blocks["vr1-table"]:
        if len(cells) >= 2:
            names |= set(cell_names(cells[0]))
    return names


def vr1_audit(raw_decls, owned, impls, listed):
    """PG22. A `Hash` or `Ord` derive the VR1 table names no use for.

    **`Eq` left this rule in revision 11.** VR1 now derives `Eq` wherever
    every field supplies it, because `clippy::derive_partial_eq_without_eq`
    demands it, so an `Eq` derive needs no stated use. **PG23 holds that half
    now**, in the other direction: it fails a type that supplies `Eq` and does
    not derive it. `Hash` and `Ord` stay rare and stay in the table.

    **One direction only, and this is the site that says so.** A VR1 row that
    no declaration uses is not a failure, exactly as PG15 and PG17 are
    one-directional. The printed count lets a reader see a stale row.
    """
    failures = []
    for name in sorted(raw_decls):
        kind, _payload, attr = raw_decls[name][0]
        if kind == "trait":
            continue
        carried = {t for t in derives_of(attr) if t in VR1_TRAITS}
        carried |= {t for t in impls.get(name, set()) if t in VR1_TRAITS}
        if carried and name not in listed:
            failures.append((name, owned.get(name, "unplaced"), sorted(carried)))
    return failures


def spelled_out(payload):
    """Whether a declaration body states its fields.

    An empty body and a `/* private */` body each state none, so PG23 reads
    neither. The compiler is the backstop for a body this document elides.
    """
    return bool(strip_block_comments(payload).strip())


def private_body(payload):
    """Whether a block comment stands in for a declaration body.

    `pub struct DiskWriter { /* private */ }` states no field and
    `pub struct Unit;` states that it has none. PG24 sizes the second at zero
    and refuses the first.
    """
    return "/*" in payload


def eq_audit(raw_decls, owned, impls, external_traits, framework, primitives):
    """PG23. A `PartialEq` derive with no `Eq` over a body that supplies `Eq`.

    `clippy::derive_partial_eq_without_eq` is a `nursery` lint and the
    workspace sets `nursery` to `deny`, so the shape does not build. The rule
    reads the field expressions PG19 reads and it skips the classes PG19
    skips. A hand-written `impl Eq` counts exactly as a derive does, which is
    the rule `hand_impls` states for PG19 and PG22.
    """
    resolve = make_resolver(raw_decls, impls, external_traits, primitives)
    missing, undecided = [], []
    for name in sorted(raw_decls):
        kind, payload, attr = raw_decls[name][0]
        if kind == "trait" or not spelled_out(payload):
            continue
        derived = derives_of(attr)
        if "PartialEq" not in derived or "Eq" in derived:
            continue
        if "Eq" in impls.get(name, set()):
            continue
        crate = owned.get(name, "unplaced")
        fields = [parse_type(expr) for _field, expr in field_exprs(kind, payload)]
        if crate in APP_ROWS and any(leaf_names(node) & framework for node in fields):
            continue
        supplies = [node_traits(node, resolve) for node in fields]
        if any(item is None for item in supplies):
            undecided.append((name, crate))
        elif all("Eq" in item for item in supplies):
            missing.append((name, crate))
    return missing, undecided


def constant_owners(blocks):
    """Every `pub const` of section 1.6, as `{name: (crate, value)}`.

    The crate is the `// duet-<name>` comment above the declaration, which is
    the one place this document states where a constant lives. `value` is the
    integer literal when the constant is a `usize`, and `None` otherwise. PG24
    reads the value as an array length and PG20b reads the crate (critic
    CR-12).
    """
    found, crate = {}, ""
    body = "\n".join(blocks["constants"])
    for line in body.splitlines():
        head = re.match(r"\s*// (duet-[a-z]+)", line)
        if head:
            crate = head.group(1)
        match = re.search(r"pub const ([A-Z][A-Z0-9_]*)\s*:\s*([A-Za-z0-9_]+)\s*=\s*([^;]+);", line)
        if not match:
            continue
        value = None
        if match.group(2) == "usize":
            digits = match.group(3).replace("_", "").strip()
            if digits.isdigit():
                value = int(digits)
        found[match.group(1)] = (crate, value)
    return found


def usize_constants(blocks):
    """Every `pub const NAME: usize` of section 1.6, as {name: value} (PG24)."""
    return {
        name: value
        for name, (_crate, value) in constant_owners(blocks).items()
        if value is not None
    }


def constant_reach_audit(raw_decls, owned, closure, constants):
    """PG20b. A constant a declaration names, in a crate the graph misses.

    An array length and a const generic argument are values, so `parse_type`
    drops them and PG20 cannot see one. The rule states its own limit at its
    section 1.5 site: a constant inside a function body is invisible to it,
    and PG28 holds that half from the section 1.6 shared-limit block.
    """
    failures = []
    for name in sorted(raw_decls):
        crate = owned.get(name)
        if crate is None:
            continue
        home = normalize_crate(crate)
        for kind, payload, _attr in raw_decls[name]:
            for field, expr in field_exprs(kind, payload):
                for token in re.findall(r"(?<![:\w])([A-Z][A-Z0-9_]{2,})(?![\w])", expr):
                    if token not in constants:
                        continue
                    target = constants[token][0]
                    if not target or target == home:
                        continue
                    if target not in closure.get(home, set()):
                        failures.append((crate, name, field, token, target))
    return sorted(set(failures))


def shared_limit_audit(blocks, closure, constants):
    """PG28. A shared limit an enforcing crate cannot reach (critic CR-12).

    Each line of the section 1.6 block is the constant, the crate that
    declares it, and every crate that enforces it inside a function body. A
    body is not a field, so PG20 and PG20b both miss the use.
    """
    rows, failures = 0, []
    for line in blocks["shared-limits"]:
        parts = line.split()
        if len(parts) < 3:
            failures.append((line.strip(), "", "the line states no owner and no enforcer"))
            continue
        constant, owner, enforcers = parts[0], parts[1], parts[2:]
        rows += 1
        if constant not in constants:
            failures.append((constant, owner, "section 1.6 declares no such constant"))
            continue
        declared = constants[constant][0]
        if declared != owner:
            failures.append(
                (constant, owner, f"the constant block declares it in {declared}")
            )
            continue
        for crate in enforcers:
            if crate != owner and owner not in closure.get(crate, set()):
                failures.append((constant, crate, f"{crate} does not reach {owner}"))
    return rows, failures


def probe_table_audit(blocks, rule_ids):
    """PG29. The section 1.9 probe table and the rule set are one set (DR5).

    It states its own limit at its section 1.5 site: the recorded text of a
    cell is unchecked, because only a run can produce it (critic concern 1).
    """
    rows, failures = {}, []
    for cells in blocks["probe-table"]:
        if len(cells) < 5:
            continue
        rule = cells[0].strip("`")
        probe = cells[2].strip("`")
        rows[rule] = (probe, cells[4])
    for rule in sorted(rule_ids):
        if rule not in rows:
            failures.append((rule, "the probe table carries no row"))
            continue
        probe, recorded = rows[rule]
        wanted = rule[0] + "P" + rule[2:]
        if probe != wanted:
            failures.append((rule, f"the row names probe {probe} and {wanted} is required"))
        if not re.search(r"exit [012]|TBD-RUN", recorded):
            failures.append((rule, "the recorded cell states no exit code"))
    for rule in sorted(rows):
        if rule in ("-", "BASE"):
            continue
        if rule not in rule_ids:
            failures.append((rule, "no prototype implements this rule id"))
    return len(rows), failures


def normalize_expr(text):
    """One type expression with every space removed, which is the row key."""
    return re.sub(r"\s+", "", text)


def recorded_sizes(blocks):
    """The section 1.9 recorded-size block, as `(expressions, heads)`.

    Each row is `<type expression> <size> <align>` and each one is a compiler
    fact the roster compile measured (DR3 exemption 4). A row that ends in the
    word `generic` names a container HEAD, whose size the payload never
    changes, and only such a row answers the head fallback.
    """
    expressions, heads = {}, {}
    for line in blocks["recorded-sizes"]:
        parts = line.split()
        generic = bool(parts) and parts[-1] == "generic"
        if generic:
            parts = parts[:-1]
        if len(parts) < 3 or not parts[-1].isdigit() or not parts[-2].isdigit():
            continue
        name = normalize_expr(" ".join(parts[:-2]))
        expressions[name] = (int(parts[-2]), int(parts[-1]))
        if generic:
            heads[name] = expressions[name]
    return expressions, heads


def enum_arms_of(source):
    """Every enum arm this document declares, as {enum: [(arm, body)]} (PG24).

    `declarations` masks an arm name, because an arm name is not a type (PG8).
    PG24 needs the name for its failure line and the arm grouping for its
    layout, so it reads the unmasked body here.
    """
    found = {}
    for _offset, block in rust_blocks(source):
        code = strip_comments(block)
        for match in re.finditer(r"\benum\s+([A-Z][A-Za-z0-9]*)\s*\{", code):
            open_index = code.index("{", match.start())
            body = code[open_index + 1 : matching_brace(code, open_index)]
            arms = []
            for arm in top_level_arms(body):
                head = re.match(r"\s*([A-Z][A-Za-z0-9]*)\s*([\s\S]*)", arm)
                if head:
                    arms.append((head.group(1), head.group(2).strip()))
            found.setdefault(match.group(1), arms)
    return found


def round_up(value, align):
    """`value` raised to the next multiple of `align`."""
    if align <= 1:
        return value
    remainder = value % align
    return value if remainder == 0 else value + align - remainder


def layout_fields(parts):
    """The (size, align) of a body, from its (size, align) field list (PG24).

    The fields sort by decreasing alignment, which is the size-optimal
    `repr(Rust)` order rustc picks, each one pads to its own alignment, and
    the total rounds up to the largest field alignment.
    """
    if not parts:
        return (0, 1)
    align = max(part[1] for part in parts)
    offset = 0
    for size, field_align in sorted(parts, key=lambda part: -part[1]):
        offset = round_up(offset, field_align) + size
    return (round_up(offset, align), align)


def array_length(text, constants):
    """The length of an array expression, or None when no section states it."""
    text = text.strip()
    if re.fullmatch(r"[0-9_]+", text):
        return int(text.replace("_", ""))
    return constants.get(text.split("::")[-1])


def head_and_args(text):
    """One type expression as `(head name, [argument, ...])`, or None.

    A path keeps its last segment, as PG19 reads it. A lifetime argument and a
    const-generic argument are values and not types, so neither one returns.
    """
    match = re.fullmatch(
        r"(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*([A-Za-z_][A-Za-z0-9_]*)\s*(<[\s\S]*>)?", text
    )
    if not match:
        return None
    args = []
    if match.group(2):
        args = [piece.strip() for piece in split_top(match.group(2)[1:-1]) if piece.strip()]
    return (match.group(1), args)


def unsized_argument(text):
    """Whether one type argument is unsized, so a pointer to it is fat.

    A slice and `str` each carry a length beside the address, and a trait
    object carries a vtable address. All three make the pointer 16 bytes.
    """
    text = text.strip()
    if text == "str" or re.match(r"^(?:dyn|impl)\b", text):
        return True
    return (
        text.startswith("[") and text.endswith("]") and len(split_top(text[1:-1], ";")) == 1
    )


def size_of_expr(text, resolve, recorded, constants):
    """The (size, align) of one type expression, or None (PG24).

    A recorded row wins, because it is what the compiler measured. After that
    the model states three language rules and reads the head of the expression
    only, never a nested argument: a `BTreeMap` is its own size whatever its
    payload is (critic C-5).

    1. A tuple is a struct of its items.
    2. A `Box`, an `Arc`, or an `Rc` over a slice or over `str` is a fat
       pointer, which is an address and a length. Over any sized argument it
       takes its own head row.
    3. `Option`, `SmallVec`, and `ArrayVec` take no head row, because a niche
       and an inline array each depend on the argument. Each one needs a row
       for the whole expression.

    A bare slice and a reference stay undecided: this model states a layout
    for neither.
    """
    text = text.strip()
    if not text or text.startswith("&"):
        return None
    key = normalize_expr(text)
    if key in recorded:
        return recorded[key]
    if text.startswith("(") and text.endswith(")"):
        items = split_top(text[1:-1])
        parts = []
        for item in items:
            part = size_of_expr(item, resolve, recorded, constants)
            if part is None:
                return None
            parts.append(part)
        return layout_fields(parts)
    if text.startswith("[") and text.endswith("]"):
        halves = split_top(text[1:-1], ";")
        if len(halves) != 2:
            return None
        count = array_length(halves[1], constants)
        inner = size_of_expr(halves[0], resolve, recorded, constants)
        if count is None or inner is None:
            return None
        return (count * inner[0], inner[1])
    parsed = head_and_args(text)
    if parsed is None:
        return None
    head, args = parsed
    if head in FAT_POINTER_HEADS and len(args) == 1 and unsized_argument(args[0]):
        return (16, 8)
    if args and head in INLINE_HEADS:
        return None
    return resolve(head)


def offers_niche(text, raw_decls, arms, constants, seen=frozenset()):
    """Whether one type offers a spare bit pattern (PG24).

    rustc lints `variant_size_differences` on a **direct** tag only. It picks
    a niche tag when the largest arm has a spare bit pattern to put the tag
    in, and it returns early from the lint in that case, so the comparison
    never runs. The model answers from the field shapes this document states.

    An integer and a float offer none. `bool` and `char` each offer one. A
    declared enum offers its unused discriminants. A struct, an array, and a
    tuple offer what their parts offer. **Every other name offers one**,
    because a pointer, a `NonZero`, and a container all carry one, and an
    unknown must not turn into a failure this guard cannot justify.
    """
    text = text.strip()
    if not text or text in seen:
        return False
    if text.startswith("&"):
        return True
    if text.startswith("(") and text.endswith(")"):
        return any(
            offers_niche(item, raw_decls, arms, constants, seen)
            for item in split_top(text[1:-1])
        )
    if text.startswith("[") and text.endswith("]"):
        halves = split_top(text[1:-1], ";")
        if len(halves) != 2 or array_length(halves[1], constants) == 0:
            return True
        return offers_niche(halves[0], raw_decls, arms, constants, seen)
    parsed = head_and_args(text)
    if parsed is None:
        return True
    head = parsed[0]
    if head in NICHE_FREE:
        return False
    if head not in raw_decls:
        return True
    kind, payload, _attr = raw_decls[head][0]
    if kind != "struct" or private_body(payload):
        return True
    deeper = seen | {text}
    return any(
        offers_niche(expr, raw_decls, arms, constants, deeper)
        for _field, expr in field_exprs("struct", payload)
    )


def make_sizer(raw_decls, arms, recorded, heads, constants, primitives):
    """A layout oracle for PG24, as the pair `(measure, explain)`.

    `primitives` comes from the section 1.9 block, never from a constant, so
    an empty map decides no primitive size and every declaration above one
    goes undecided (critic K-4).

    `measure(name)` gives `(size, align, [(arm, size), ...])` or None, and
    `explain(name)` gives every field expression of one declaration that the
    model cannot size. A cycle through a declaration is undecided, and only a
    decided answer enters the cache, so no in-flight answer is recorded.
    """
    cache = {}
    active = set()

    def resolve(name):
        answer = measure(name)
        return None if answer is None else (answer[0], answer[1])

    def arm_parts(arm_body):
        parts = []
        for _field, expr in field_exprs("enum", arm_body):
            part = size_of_expr(expr, resolve, recorded, constants)
            if part is None:
                return None
            parts.append(part)
        return parts

    def measure_enum(name):
        laid = []
        for arm_name, arm_body in arms.get(name, []):
            parts = arm_parts(arm_body)
            if parts is None:
                return None
            laid.append((arm_name, layout_fields(parts)))
        if not laid:
            return None
        if not any(part[0] for _arm_name, part in laid):
            return (1, 1, [(arm_name, 0) for arm_name, _part in laid])
        align = max(1, max(part[1] for _arm_name, part in laid))
        widest = max(part[0] for _arm_name, part in laid)
        # rustc pads each arm to the payload alignment of the enum, so the
        # spread compares the padded numbers and not the bare field sums.
        sizes = [(arm_name, round_up(part[0], align)) for arm_name, part in laid]
        return (round_up(widest + 1, align), align, sizes)

    def measure_struct(payload):
        parts = []
        for _field, expr in field_exprs("struct", payload):
            part = size_of_expr(expr, resolve, recorded, constants)
            if part is None:
                return None
            parts.append(part)
        size, align = layout_fields(parts)
        return (size, align, [])

    def measure_declaration(name):
        kind, payload, _attr = raw_decls[name][0]
        if kind == "trait" or private_body(payload):
            return None
        return measure_enum(name) if kind == "enum" else measure_struct(payload)

    def measure(name):
        if name in primitives:
            size, align = primitives[name]
            return (size, align, [])
        if name in cache:
            return cache[name]
        if name in active:
            return None
        if name in raw_decls:
            active.add(name)
            answer = measure_declaration(name)
            active.discard(name)
            if answer is not None:
                cache[name] = answer
            return answer
        if name in heads:
            return heads[name] + ([],)
        if name in recorded:
            return recorded[name] + ([],)
        return None

    def explain(name):
        kind, payload, _attr = raw_decls[name][0]
        if kind == "trait":
            return []
        if private_body(payload):
            return [("body", "/* private */")]
        if kind != "enum":
            return [
                (field or expr.strip(), expr.strip())
                for field, expr in field_exprs("struct", payload)
                if size_of_expr(expr, resolve, recorded, constants) is None
            ]
        out = []
        for arm_name, arm_body in arms.get(name, []):
            for field, expr in field_exprs("enum", arm_body):
                if size_of_expr(expr, resolve, recorded, constants) is None:
                    label = arm_name + "." + field if field else arm_name
                    out.append((label, expr.strip()))
        if not arms.get(name):
            out.append(("body", "an arm list this guard cannot read"))
        return out

    return measure, explain


def direct_tag(name, raw_decls, arms, constants, arm_sizes):
    """Whether rustc gives one enum a direct tag, which PG24 needs.

    rustc puts the tag in a spare bit pattern of the largest arm when that arm
    has one, and it returns from `variant_size_differences` before the
    comparison in that case. The lint therefore reaches a direct tag only.
    The largest arm decides, because it is the one arm a niche tag can use.
    """
    widest = max(arm_sizes, key=lambda item: item[1])[0]
    for arm_name, arm_body in arms.get(name, []):
        if arm_name != widest:
            continue
        return not any(
            offers_niche(expr, raw_decls, arms, constants)
            for _field, expr in field_exprs("enum", arm_body)
        )
    return False


def size_audit(raw_decls, owned, arms, constants, measure, explain):
    """PG24. Every declaration's size, and `variant_size_differences`.

    The spread reads the arms that carry a payload, each one padded to the
    payload alignment of the enum. An arm of size zero carries no payload this
    model can see, and rustc holds the same guard: its lint fires only when
    the second largest arm is larger than zero. A niche-tagged enum is outside
    the lint, so `direct_tag` decides before the comparison runs.

    **A single-site `#[expect(variant_size_differences, ...)]` takes a
    declaration out of the failure set** and on to the `expected` list, the
    way an `#[expect(missing_copy_implementations, ...)]` takes one out of
    PG9. Audio-owned state stays inline under TH1, so `SlotState` carries the
    spread and the expectation together, and PG21 holds the sites and the
    second Appendix B.1 table to one set.
    """
    decided, undecided, spread, expected = [], [], [], []
    for name in sorted(raw_decls):
        kind, _payload, attr = raw_decls[name][0]
        if kind == "trait":
            continue
        crate = owned.get(name, "unplaced")
        answer = measure(name)
        if answer is None:
            undecided.append((name, crate, explain(name)))
            continue
        size, align, arm_sizes = answer
        decided.append((name, crate, size, align, arm_sizes))
        carried = sorted(
            (item for item in arm_sizes if item[1] > 0), key=lambda item: -item[1]
        )
        if len(carried) < 2 or carried[0][1] <= 3 * carried[1][1]:
            continue
        if not direct_tag(name, raw_decls, arms, constants, arm_sizes):
            continue
        if re.search(r"variant_size_differences", attr):
            expected.append((name, crate, carried[0][0], carried[0][1], carried[1][1]))
            continue
        spread.append((name, crate, carried[0][0], carried[0][1], carried[1][1]))
    return decided, undecided, spread, expected


def section_fourteen(source):
    """The text of section 14, or the empty string."""
    found = re.search(
        r"## 14\. The measurable completion outcome for version one\n(.*?)\n## Appendix A",
        source,
        re.S,
    )
    return found.group(1) if found else ""


def chunk_phases(blocks):
    """Every chunk id and its phase, from the section 13.3 table."""
    phases = {}
    for cells in blocks["phase-table"]:
        if len(cells) < 4 or not cells[0].isdigit():
            continue
        phase = int(cells[0])
        for chunk in re.findall(r"\b([A-Z]{1,2}\d{1,2})\b", cells[1] + " " + cells[3]):
            phases[chunk] = phase
    return phases


def selected_tests(source, blocks):
    """PG16. Every test section 14 selects, against the selected-test table."""
    text = section_fourteen(source)
    failures = []
    if not text:
        return [("<section 14>", "the section is absent")], set()
    selected = set(re.findall(r"\btest\(([A-Za-z0-9_]+)\)", text))
    selected |= set(re.findall(r"(?<![\w-])--test\s+([A-Za-z0-9_]+)", text))
    rows = {}
    for cells in blocks["selected-tests"]:
        if len(cells) < 6:
            continue
        rows[cells[0].strip("`")] = (cells[1], cells[2])
    phases = chunk_phases(blocks)
    for name in sorted(selected):
        if name not in rows:
            failures.append((name, "no row in the selected-test table"))
            continue
        chunk, phase = rows[name]
        if chunk not in phases:
            failures.append((name, f"chunk {chunk} appears in no phase"))
        elif phase.isdigit() and phases[chunk] != int(phase):
            failures.append((name, f"chunk {chunk} is in phase {phases[chunk]}, not {phase}"))
    return failures, selected


def stray_markers(source):
    """PG27. Every `GUARD BLOCK` marker this document carries, checked.

    `read_block` anchors a block on the marker inside its own heading region,
    so a marker that sits outside that region, and a marker whose id the
    register does not hold, are both invisible to it. A Critic probe added a
    second `audio-owned` marker under section 5.12 and a `probe-block` marker
    under a new heading, and the run exited 0 on both (critic C-13, C-14).

    It returns a list of `(id, reason)`.
    """
    seen, failures = {}, []
    for match in MARKER.finditer(source):
        seen.setdefault(match.group(1), []).append(match.start())
    for block_id, offsets in sorted(seen.items()):
        if block_id not in DATA_BLOCKS:
            failures.append((block_id, "the marker id is not a registered block"))
            continue
        if len(offsets) != 1:
            failures.append(
                (block_id, f"the marker appears {len(offsets)} times in this document")
            )
    return failures


def link_rows(source):
    """Every `X before Y` row of the section 13.4 table, as (pred, succ) pairs.

    The table is prose plus chunk ids, so the parser takes the ids on each
    side of the word `before` in the row's FIRST cell and nothing else. A row
    whose first cell holds no `before` is not a link row.
    """
    pairs = []
    for line in source.splitlines():
        if not line.startswith("| ") or " before " not in line:
            continue
        head = line.split("|")[1].strip()
        # The cell is chunk ids and the word `before`, and nothing else. A
        # prose cell that happens to hold the word is not a link row, which a
        # finding id such as `C16-W13` proved the loose form cannot tell.
        if not re.fullmatch(
            r"[A-Z]\d{1,2}(?: and [A-Z]\d{1,2})* before"
            r" [A-Z]\d{1,2}(?:(?:,| and) [A-Z]\d{1,2})*",
            head,
        ):
            continue
        left, right = head.split(" before ", 1)
        before = re.findall(r"\b([A-Z]\d{1,2})\b", left)
        after = re.findall(r"\b([A-Z]\d{1,2})\b", right)
        for predecessor in before:
            for successor in after:
                pairs.append((predecessor, successor))
    return pairs


def link_audit(source, blocks):
    """PG31. Every section 13.4 link runs forward in the 13.3 phase table.

    SM8 states the rule. The phase table is the parallelism plan: a phase
    ASSERTS that its chunks have no ordering between them. Revision 16 put
    three pairs in one phase across a crate edge section 1.3 states, and
    carried no link row for any of them (critic C16-12).

    **The rule's own limit, stated here.** It reads the links the document
    WRITES. A crate edge that binds and that section 13.4 does not carry is
    outside it, exactly as PG17 is one-directional for the unknown table. The
    rule refuses a link that runs backward or sideways, and it refuses a link
    whose chunk no phase holds.
    """
    phases = chunk_phases(blocks)
    failures = []
    for predecessor, successor in link_rows(source):
        if predecessor not in phases:
            failures.append((f"{predecessor} before {successor}", f"{predecessor} is in no phase"))
            continue
        if successor not in phases:
            failures.append((f"{predecessor} before {successor}", f"{successor} is in no phase"))
            continue
        if phases[predecessor] >= phases[successor]:
            failures.append(
                (
                    f"{predecessor} before {successor}",
                    f"{predecessor} is in phase {phases[predecessor]} and"
                    f" {successor} is in phase {phases[successor]}",
                )
            )
    return failures


def chunk_lines(blocks):
    """Every chunk id mapped to the line it belongs to, from the `line-map`.

    A chunk id alone does not state its line: `T1` to `T4` are four lines of
    one chunk each and `M0` to `M8` are manifest chunks and no line at all.
    Revision 17 derived the line from the first letter and skipped `M` and `T`
    inside the machine, with no line of the document stating the exemption
    (critic C17-W9). The map is data now.
    """
    lines, crates = {}, {}
    for row in blocks["line-map"]:
        tokens = row.split()
        if len(tokens) < 2:
            continue
        lines[tokens[0]] = tokens[0]
        crates[tokens[0]] = normalize_crate(tokens[1])
    return lines, crates


def crate_of_chunk(chunk, crates):
    """The crate one chunk writes, from the `line-map`, or None."""
    if chunk in crates:
        return crates[chunk]
    return crates.get(chunk[0])


def phase_pair_audit(blocks, exempt):
    """PG31b. Two chunks in one phase whose crates carry a 1.3 edge.

    C16-12 was three chunk pairs sharing a phase across a crate edge **with no
    link row for any of them**, and PG31 reads rows, so a new pair in that
    state stays invisible (critic C17-W9). This rule reads no row: it takes
    each phase, pairs its line chunks, and asks the section 1.3 edge list
    whether one crate depends on the other.

    A pair the rule finds is either a 13.4 link, which PG31 then forbids from
    sharing a phase, or a row of the `phase-pair-exempt` block with the reason
    the edge does not bind. A pair that is neither is a failure.

    **The rule's own limit, stated here.** It decides that an edge EXISTS
    between two crates in one phase and never that the edge binds at build
    time. SM8 states when an edge binds. The exemption block is the reviewed
    record of every pair where it does not, so the set is visible rather than
    absent.
    """
    phases = chunk_phases(blocks)
    _lines, crates = chunk_lines(blocks)
    edges = edge_list(blocks)
    grouped = {}
    for chunk, phase in phases.items():
        if chunk.startswith("M"):
            continue
        grouped.setdefault(phase, []).append(chunk)
    failures, found = [], 0
    for phase in sorted(grouped):
        members = sorted(grouped[phase])
        for index, first in enumerate(members):
            for second in members[index + 1 :]:
                one, two = crate_of_chunk(first, crates), crate_of_chunk(second, crates)
                if not one or not two or one == two:
                    continue
                for consumer, producer, other in (
                    (first, second, two),
                    (second, first, one),
                ):
                    if other not in edges.get(crate_of_chunk(consumer, crates), set()):
                        continue
                    found += 1
                    if f"{consumer} {producer}" in exempt:
                        continue
                    failures.append(
                        (
                            f"{consumer} and {producer}",
                            f"both in phase {phase}, and"
                            f" {crate_of_chunk(consumer, crates)} depends on {other};"
                            " no exemption row states why the edge does not bind",
                        )
                    )
    return found, failures


WORD_NUMBERS = {
    "one": 1, "two": 2, "three": 3, "four": 4, "five": 5, "six": 6, "seven": 7,
    "eight": 8, "nine": 9, "ten": 10, "eleven": 11, "twelve": 12, "thirteen": 13,
    "fourteen": 14, "fifteen": 15, "sixteen": 16, "seventeen": 17, "eighteen": 18,
    "nineteen": 19, "twenty": 20, "twenty-one": 21, "twenty-two": 22,
}


def declared_functions(source):
    """Every `pub fn` name a Rust block of this document declares.

    It is the THIRD source PG33 needs (critic C18-3). The B.1 Site column and
    the section 2.3 sentence are both text an author types, so a name can
    enter both and exist in neither declaration; the phantom-function class
    was green under the rule written to end it. A function is not a type, so
    no placement counter moves when a declaration disappears, and this walk is
    the only thing that sees it.
    """
    found = {}
    for _offset, block in rust_blocks(source):
        for match in re.finditer(r"(?m)^\s*pub(?:\(crate\))? fn ([a-z_][A-Za-z0-9_]*)", block):
            found[match.group(1)] = found.get(match.group(1), 0) + 1
    return found


def ambiguous_functions(source):
    """Every function name more than one Rust block of this document declares.

    N19-6 recorded the limit PG33 states at its own site: the register is
    about a suppression's SITE, and a site is a function, so a B.1 row that
    shares a name with an unrelated function was green. **An ambiguous name
    is the whole of that hole**, because a unique name resolves to exactly the
    declaration the row means. The rule refuses a B.1 site whose name this
    document declares more than once, so the register can no longer point at
    two functions at once (critic N21-9 debt, closed in revision 22).
    """
    return {name for name, times in declared_functions(source).items() if times > 1}


def suppression_audit(source, blocks):
    """PG33. Three sets are one set: B.1, the 2.3 list, and the declarations.

    Appendix B.1 is the register of every accepted suppression. Revision 16
    carried a row there for `finite_to_f32_saturating`, a function no section
    declared, and it survived seventeen revisions because PG21 reads the four
    `#[expect]` expectation tables and nothing read this one (critic C17-2,
    N17-9). Revision 18 held the register against a SENTENCE, which is text
    the same author types, so the same class stayed green: deleting the
    declaration and leaving the sentence and the row exits 0 (critic C18-3).

    The rule now reads three sources and one of them is not prose. It also
    compares the sentence's own COUNT WORD with the length of the list, which
    is the half of C17-2 that was three disagreeing numbers (critic C18-W3).

    **The rule's own limit, stated here.** It reads the one section 2.3
    sentence anchored on the words `functions carry a suppression`, so a
    suppression in another crate is outside it. The declaration set is every
    `pub fn` this document writes, not only section 2.3's, so a name declared
    in another section satisfies the third set; that is deliberate, because
    the register is about a suppression's SITE and a site is a function.
    """
    sites = []
    for cells in blocks["b1-convert"]:
        if cells:
            sites.extend(cell_names(cells[0]))
    found = re.search(r"(?s)\b([A-Za-z-]+) functions carry a suppression: (.*?)\.\s", source)
    if found is None:
        return None, "section 2.3 states no list of the functions that carry a suppression"
    listed = re.findall(r"`([A-Za-z_][A-Za-z0-9_]*)`", found.group(2))
    declared = declared_functions(source)
    failures = []
    for name in sorted(set(sites) - set(listed)):
        failures.append((name, "Appendix B.1 gives it a reason and section 2.3 omits it"))
    for name in sorted(set(listed) - set(sites)):
        failures.append((name, "section 2.3 lists it and Appendix B.1 gives it no reason"))
    for name in sorted((set(sites) | set(listed)) - set(declared)):
        failures.append(
            (name, "the suppression register names it and no Rust block declares the function")
        )
    # N19-6, closed in revision 22. A site is a FUNCTION, so a register name
    # that resolves to two declarations points at neither, and the rule that
    # holds the register to the declarations cannot say which one it holds.
    for name in sorted((set(sites) | set(listed)) & ambiguous_functions(source)):
        failures.append(
            (
                name,
                f"this document declares {declared[name]} functions of that name, so the"
                " suppression register resolves to no one site",
            )
        )
    word = found.group(1).lower()
    stated = WORD_NUMBERS.get(word)
    if stated is None:
        failures.append((word, "the count word of the section 2.3 list is not a number word"))
    elif stated != len(listed):
        failures.append(
            (word, f"the section 2.3 list states {word} and holds {len(listed)} names")
        )
    # The four Appendix B.1 count sentences, each against its own block
    # (critic C19-W19). Revision 19 read the section 2.3 count word and left
    # these four unread, three lines from the tables the same rule holds: a
    # Critic changed "Seven suppressions" to "Three" and "Six complexity
    # suppressions" to "Nine" and both runs exited 0 with no finding line.
    for phrase, block_id in B1_COUNT_SENTENCES:
        found_count = re.search(r"\b([A-Za-z]+) " + re.escape(phrase), source)
        if found_count is None:
            failures.append((phrase, "Appendix B.1 states no count sentence for it"))
            continue
        said = WORD_NUMBERS.get(found_count.group(1).lower())
        if said is None:
            failures.append(
                (found_count.group(1), f"the count word of `{phrase}` is not a number word")
            )
        elif said != len(blocks[block_id]):
            failures.append(
                (
                    phrase,
                    f"Appendix B.1 states {found_count.group(1).lower()} and the"
                    f" `{block_id}` block holds {len(blocks[block_id])} rows",
                )
            )
    return (sites, listed, failures), None


# The four Appendix B.1 count sentences, each with the block it introduces
# (critic C19-W19).
B1_COUNT_SENTENCES = (
    ("suppressions in `duet-time::convert`", "b1-convert"),
    ("complexity suppressions", "b1-complexity"),
    ("`missing_copy_implementations` expectations", "b1-copy"),
    ("`variant_size_differences` expectations", "b1-variant"),
)


# PG37 reads ONE declaration line per budget id, and never a bag of the
# integers that share a block with it (critic C21-W1, C21-W2). Revision 21
# collected every integer of every block that mentioned the id and passed when
# the stated value was one of them, so five of the six constants of the
# section 1.6 block satisfied B120.
#
# A CITING LINE is a line that names the budget id, with the line above it and
# the line below it: a declaration carries its budget id in a doc comment or a
# trailing comment, and both sides occur. Two shapes carry a capacity:
# a `const NAME: usize = <n>;` and a fixed-capacity container spelled
# `[T; <n>]`, `ArrayVec<T, <n>>`, or `SmallVec<[T; <n>]>`.
CAPACITY_SITE = re.compile(
    r"\[\s*[A-Za-z_][A-Za-z0-9_:<>, ]*;\s*([0-9][0-9_]*)\s*\]"
    r"|ArrayVec<[^>]*,\s*([0-9][0-9_]*)\s*>"
    r"|:\s*usize\s*=\s*([0-9][0-9_]*)"
)


# The fewest budget rows PG37 may decide. It is a ratchet and never a target:
# the rule decides ten rows of this document, and a change that drops the
# decided set below this floor is a silent shrink of a denominator (critic
# N21-3). `sync_floors.py` does not write it, because it is a property of the
# rule's reach and not a row count of a block.
LINE_CHUNK_FLOOR = 50
VALUE_FLOOR = 9

# The most rows the `phase-pair-exempt` block may hold. N19-1 recorded that
# PG31b rejects nothing today: the found pair set and the allow list are the
# same 24 pairs, so the rule fires on no input this document holds. The floor
# of PG27b already refuses a DELETION; this ceiling refuses an ADDITION, so a
# twenty-fifth pair cannot be waved through by typing one more row and must be
# reviewed with the ceiling raised in the same changeset. **The rule's limit**:
# it bounds the SIZE of the allow list and never the truth of a row, and the
# reason rule beside it is what holds a row to its own pair.
EXEMPT_CEILING = 24


def adjacent_declaration(lines, index):
    """The one declaration line a budget citation belongs to.

    A budget id sits in a DOC COMMENT above its declaration or in a TRAILING
    comment below it, and revision 21 read both neighbours at once. The line
    before the doc comment of `LIMITER_SLOTS` is `pub const REVERB_SLOTS:
    usize = 4;`, so a planted `4 slots` for B120 was green while the rule's
    own text claimed it was red (critic C21-W2).

    The direction is decided by the citing line itself. A `///` doc comment
    reads DOWN to the first line that is not a comment. A `//` trailing
    comment reads UP to the first line that is not a comment. A citing line
    that is already a declaration is its own oracle. So each citation resolves
    to exactly one declaration and never to a neighbour of it.
    """
    text = lines[index].strip()
    if text.startswith("///") or text.startswith("//!"):
        step, position = 1, index + 1
    elif text.startswith("//"):
        step, position = -1, index - 1
    else:
        return ""
    while 0 <= position < len(lines):
        candidate = lines[position].strip()
        if candidate and not candidate.startswith("//"):
            return lines[position]
        position += step
    return ""


def budget_value_audit(source, blocks):
    """PG37. A budget VALUE that a citing declaration line refutes.

    Revision 19 changed the B113 row from "13 elements" to "12 elements",
    left the declaration at `SmallVec<[Ticks; 13]>`, and every guard was green
    while two closure rows stayed in place: PG30 reads the CITATION set and
    never the value (critic C19-W9). A budget value is the number an
    implementer types, so a stale one is a defect that reaches the code.

    **The oracle is the citing line and the line AFTER it, and never the line
    before** (critic C21I-W1, C21-W2). A declaration carries its budget id in
    a doc comment directly above it, so the capacity is on the citing line or
    on the next one and never on the previous one. Revision 21 read a
    three-line window, and the line BEFORE the doc comment of `LIMITER_SLOTS`
    is `pub const REVERB_SLOTS: usize = 4;`, so a planted `4 slots` for B120
    was green while the rule's own text and this docstring both claimed it was
    red. The window is two lines now, so `4 slots`, `8 slots`, `48 slots` and
    `2048 slots` are each red for B120.

    **The rule carries a FLOOR of its own** (critic N21-3). It decides a
    minority of the budget rows and prints `VALUE SKIPPED` for the rest, so a
    silent shrink of the decided set would look exactly like a clean run. A
    decided count below `VALUE_FLOOR` is a failure.

    **The scope is the CITING DECLARATION and not the wording of the value
    cell** (critic C21-W2). Revision 21 read only a cell that ended in
    "elements", "slots", "bins" or "records", which skipped B47, B34 and B86
    on their spelling. A budget whose value cell opens with an integer and
    whose citing line declares a capacity is in scope, whatever the cell calls
    it. The run prints `VALUE ROWS` and `VALUE SKIPPED`, so the denominator
    and the remainder are both readable.
    """
    failures, decided, skipped = [], 0, []
    bodies = [m.group(1) for m in re.finditer(r"```(?:rust|text)\n(.*?)```", source, re.S)]
    for cells in blocks["budget-table"]:
        if len(cells) < 4:
            continue
        budget = cells[0].strip().strip("`")
        found = re.match(r"^([0-9][0-9_]*)\b", cells[1].strip())
        if found is None:
            skipped.append(budget)
            continue
        stated = int(found.group(1).replace("_", ""))
        capacities = set()
        for body in bodies:
            lines = body.splitlines()
            for index, line in enumerate(lines):
                if not re.search(r"\b" + re.escape(budget) + r"\b", line):
                    continue
                window = line + "\n" + adjacent_declaration(lines, index)
                for site in CAPACITY_SITE.finditer(window):
                    for group in site.groups():
                        if group is not None:
                            capacities.add(int(group.replace("_", "")))
        if not capacities:
            skipped.append(budget)
            continue
        decided += 1
        if stated not in capacities:
            failures.append(
                (
                    budget,
                    f"the row states {stated} and the declaration line that cites it writes"
                    f" {', '.join(str(value) for value in sorted(capacities))}",
                )
            )
    return decided, failures, skipped


def tail_phase_audit(source, blocks):
    """PG34. The phase table is the source of its own three numbers.

    Revision 17 added the sixteenth phase and left three numbers in the SM6
    cost paragraph stating thirteen phases and a widest count of eight (critic
    C17-W10). A chunk moved into the acceptance phase also passed every rule,
    because the prose that says the last phase writes no file is read by
    nothing (critic N17-7).

    Revision 18 implemented the tail alone while its own docstring and the
    C17-W10 closure row both claimed it held the phase count and the widest
    phase as well, so restoring the revision-17 defect in the SM6 bullet was
    green (critic C18-W2). All three now run.

    **The rule's own limit, stated here.** It reads ONE anchored sentence of
    the SM6 bullet, the one that opens `The plan is longer and narrower`. A
    phase count stated anywhere else is outside it, and PG30's own citation
    rule is what holds a budget id to its sites.
    """
    rows = blocks["phase-table"]
    widths, chunks = [], []
    for cells in rows:
        if len(cells) < 5:
            continue
        try:
            widths.append(int(cells[4].strip()))
        except ValueError:
            widths.append(-1)
        chunks.append(re.findall(r"\b([A-Z]\d{1,2})\b", cells[3]))
    failures = []
    if not widths:
        return [("phase-table", "the block states no width")]
    if widths[-1] != 0:
        failures.append(("phase-table", f"the last phase has width {widths[-1]} and not 0"))
    if chunks and chunks[-1]:
        failures.append(
            (
                "phase-table",
                f"the last phase names {', '.join(chunks[-1])}, and the acceptance"
                " run writes no file and commits nothing",
            )
        )
    bullet = re.search(
        r"(?s)The plan is longer and narrower than revision 5's\.\*\* (.*?)\. The alternative rule",
        source,
    )
    if bullet is None:
        failures.append(
            ("SM6", "the cost bullet that states the phase count is not in this document")
        )
        return failures
    text = bullet.group(1).lower()
    # Two anchored phrases, not a word sweep: the bullet also counts chunks and
    # phases in its own prose, and a sweep would read those.
    count = re.search(r"([a-z-]+) phases replace", text)
    widest = re.search(r"falls from [a-z-]+ to ([a-z-]+)", text)
    if count is None or widest is None:
        failures.append(
            ("SM6", "the cost bullet states no `<n> phases replace` or no `falls from ... to <n>`")
        )
        return failures
    stated_count = WORD_NUMBERS.get(count.group(1))
    stated_widest = WORD_NUMBERS.get(widest.group(1))
    if stated_count != len(widths):
        failures.append(
            (
                "SM6",
                f"the cost bullet states {count.group(1)} phases and the table holds"
                f" {len(widths)}",
            )
        )
    if stated_widest != max(widths):
        failures.append(
            (
                "SM6",
                f"the cost bullet states a widest phase of {widest.group(1)} and the"
                f" table's widest is {max(widths)}",
            )
        )
    return failures


def line_phase_audit(blocks):
    """PG31. SM6: two chunks of one line never share a phase.

    A chunk id is a line letter and a number, so the line is derivable from
    the id and needs no second source.
    """
    phases = chunk_phases(blocks)
    lines, _crates = chunk_lines(blocks)
    seen, failures = {}, []
    for chunk, phase in sorted(phases.items()):
        line = lines.get(chunk) or lines.get(chunk[0])
        if line is None:
            # A manifest chunk is in no line: SM1 runs exactly one per phase,
            # so two can never share one. The `line-map` block is the source
            # and section 13.0 states the exemption (critic C17-W9).
            continue
        key = (line, phase)
        if key in seen:
            failures.append((f"{seen[key]} and {chunk}", f"one line, both in phase {phase} (SM6)"))
        else:
            seen[key] = chunk
    return failures


def chunk_ids(source):
    """Every chunk id a section 13.2 line table declares in its own row."""
    return {
        match.group(1)
        for match in re.finditer(r"(?m)^\| ([A-Z]{1,2}\d{1,2}) \| \d+ \|", source)
    }


def chunk_lock_writers(source):
    """Every chunk row of sections 13.1 and 13.2, as `{id: (phase, writes_lock)}`.

    A chunk row opens with its id and its phase, which is the shape
    `chunk_ids` already reads. The third fact this rule needs is whether the
    row names `Cargo.lock` anywhere in its own cells, because SM5 makes that
    cell the one declaration of a lock-file writer.
    """
    found = {}
    for line in source.splitlines():
        match = re.match(r"^\| ([A-Z]{1,2}\d{1,2}) \| (\d+) \|", line)
        if not match:
            continue
        found[match.group(1)] = (int(match.group(2)), "Cargo.lock" in line)
    return found


def lock_sequence_audit(source, blocks):
    """PG35. The per-phase `Cargo.lock` writer sequence is derived, never typed.

    Revision 19 stated a hand-counted sequence and it was wrong at seven of
    the sixteen phases (critic C19-6). The count is a fact of the chunk
    tables, so this rule derives it and holds the sentence to the derivation.

    It states its own limit at its section 1.5 site: it counts a chunk that
    DECLARES `Cargo.lock` in its own write scope, so a chunk that writes a
    member manifest and omits the file from its cell is an SM5 defect this
    rule cannot see.
    """
    phases = [int(cells[0]) for cells in blocks["phase-table"] if cells and cells[0].isdigit()]
    if not phases:
        return None, [("phase-table", "the block states no phase, so no sequence can be derived")]
    counts = {phase: 0 for phase in range(0, max(phases) + 1)}
    for _chunk, (phase, writes) in chunk_lock_writers(source).items():
        if writes and phase in counts:
            counts[phase] += 1
    derived = [counts[phase] for phase in range(0, max(phases) + 1)]
    stated = re.search(
        r"Per-phase `Cargo\.lock` writer counts, phases 0 to (\d+):\s*([0-9,\s]+?)\*\*",
        source,
    )
    if not stated:
        return derived, [("13.3", "the document states no per-phase `Cargo.lock` writer sentence")]
    last = int(stated.group(1))
    numbers = [int(token) for token in re.findall(r"\d+", stated.group(2))]
    failures = []
    if last != max(phases):
        failures.append(
            ("13.3", f"the sentence names phases 0 to {last} and the phase table ends at {max(phases)}")
        )
    if numbers != derived:
        failures.append(
            (
                "13.3",
                "the stated sequence is "
                + ", ".join(str(value) for value in numbers)
                + " and the chunk tables derive "
                + ", ".join(str(value) for value in derived),
            )
        )
    return derived, failures


def manifest_pin_owners(source):
    """Every pin the section 13.1 manifest table gives a chunk, as `{pin: chunk}`.

    The table is read from the raw source, exactly as `link_rows` reads the
    section 13.4 table: a manifest row opens with its chunk id and its phase,
    and its `Also writes` cell carries the one pin list of that phase.
    """
    owners = {}
    for line in source.splitlines():
        match = re.match(r"^\| (M\d+) \| \d+ \|", line)
        if not match:
            continue
        listed = re.search(r"\[workspace\.dependencies\]` only \(([^)]*)\)", line)
        if not listed:
            continue
        for pin in re.findall(r"`([a-z0-9_-]+)`", listed.group(1)):
            owners[pin] = match.group(1)
    return owners


def appendix_pin_rows(source):
    """Every Appendix B.3 and B.5 row, as `(pin, stated owner, appendix)`.

    B.3's owner is its last cell and B.5's is its fourth, so each appendix is
    read inside its own heading region and never by a shared column index.
    """
    rows = []
    for label, column, stop in (("B.3", -1, "B.4"), ("B.5", 3, "Every timeout")):
        # The heading is anchored to the start of a LINE. A recorded probe cell
        # of section 1.9 quotes this rule's own failure line, and a plain
        # substring search found that quote first and read 646 KB as one
        # appendix (critic C19-1, found while this rule was written).
        opened = re.search(r"(?m)^### " + re.escape(label) + r" ", source)
        if not opened:
            continue
        start = opened.start()
        closed = re.search(r"(?m)^#{3,4} " + re.escape(stop), source[start:])
        region = source[start : start + closed.start()] if closed else source[start:]
        for line in region.splitlines():
            if not line.startswith("| `"):
                continue
            cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
            if len(cells) <= max(column, 0):
                continue
            pin = cells[0].strip("`")
            if not re.fullmatch(r"[a-z0-9_-]+", pin):
                continue
            rows.append((pin, cells[column].strip("`").strip(), label))
    return rows


def pin_owner_audit(source):
    """PG36. The Appendix B.3 and B.5 owner columns and the 13.1 pin lists are one set.

    Revision 19 carried six B.5 rows and three B.3 rows whose Owner named a
    chunk that pins nothing of that name, and `cpal` read M3 in one appendix
    while M4 pinned it in the plan (critic C19-5). The manifest table is the
    one home of a pin owner, so both appendices cite it and this rule holds
    the three to one set.

    It states its own limit at its section 1.5 site: it decides the OWNER of
    a pin and never the feature set, because a feature list is resolved
    against a pinned version and this document holds no manifest.
    """
    owners = manifest_pin_owners(source)
    if not owners:
        return 0, [("13.1", "the manifest table lists no pin, so no owner can be derived")]
    rows = appendix_pin_rows(source)
    if not rows:
        return 0, [("B.3", "neither appendix states a pin row, so the rule has no subject")]
    failures = []
    for pin, stated, appendix in rows:
        if pin not in owners:
            failures.append((pin, f"Appendix {appendix} names owner {stated} and 13.1 pins it nowhere"))
        elif stated != owners[pin]:
            failures.append(
                (pin, f"Appendix {appendix} names owner {stated} and 13.1 gives the pin to {owners[pin]}")
            )
    return len(rows), failures


def block_member_kinds(blocks):
    """The section 1.9 membership block, as `({block: kind}, bad)` (PG27).

    Each line names one registered block and the one membership kind the
    guard runs over its rows. A block with no row, a row over an id the
    register does not hold, and a kind this guard does not implement are each
    a failure, so the register and this block are one set exactly as the
    register and the rule-to-block map are (WR-18).
    """
    kinds, bad = {}, []
    for line in blocks["block-members"]:
        parts = line.split()
        if len(parts) < 2:
            bad.append((line.strip(), "the row is not one block id and one kind"))
            continue
        block_id, kind = parts[0], parts[1]
        if block_id not in DATA_BLOCKS:
            bad.append((block_id, "the register does not hold it"))
            continue
        if kind not in MEMBER_KINDS:
            bad.append((block_id, f"the kind `{kind}` is not one this guard runs"))
            continue
        if block_id in kinds:
            bad.append((block_id, "the block carries two membership rows"))
            continue
        kinds[block_id] = kind
    for block_id in sorted(set(DATA_BLOCKS) - set(kinds)):
        bad.append((block_id, "no membership row names it"))
    return kinds, bad


def membership_audit(blocks, kinds, context):
    """PG27. Every row of every registered block names a referent (WR-18).

    DR7 closes the DELETION class: a `rows>=` minimum refuses a removed row.
    It leaves the ADDITION class open, and three blocks grant coverage by
    row, so one added word takes a type out of four rules. A Critic probe
    added `ProbeUnplaced` to the candidate drop list and a red document went
    green (critic WR-18, MINE-16). A row must therefore name an entity that
    exists elsewhere in this document or in the external table.

    **The rule's own limit, stated here**: it decides the referent of a row
    and never the row's meaning, so a row that names a real entity in the
    wrong place is outside it.

    It returns `(rows checked, [(block, row, reason), ...])`.
    """
    checked, failures = 0, []

    def fail(block_id, row, reason):
        failures.append((block_id, row, reason))

    for block_id in sorted(kinds):
        kind = kinds[block_id]
        rows = blocks[block_id]
        for row in rows:
            checked += 1
            text = row if isinstance(row, str) else " | ".join(row)
            cells = row if isinstance(row, list) else []
            tokens = text.split() if isinstance(row, str) else []
            if kind == "crate-name":
                # The two crate blocks hold each other to one set. The crate
                # table is read against the edge list and the edge list is
                # read against the crate table, so a name added to either one
                # has no referent in the other.
                universe = (
                    context["edge_crates"]
                    if block_id == "crate-table"
                    else context["crates"]
                )
                found = set(re.findall(r"`([A-Za-z0-9_/-]+)`", cells[0])) if cells else set()
                if not found:
                    found = {
                        token
                        for token in re.findall(r"[A-Za-z][A-Za-z0-9_-]*", text)
                        if token.startswith("duet")
                    }
                for name in sorted(found):
                    if normalize_crate(name) not in universe:
                        fail(block_id, text, f"`{name}` is no crate of the other block")
            elif kind == "pin-name":
                if not tokens or tokens[0].strip("`") not in context["third_party"]:
                    fail(block_id, text, "the crate is in no section 1.2 dependency cell")
            elif kind == "map-crate":
                if len(tokens) < 2 or tokens[1].strip("`") not in context["third_party"]:
                    fail(block_id, text, "the crate is in no section 1.2 dependency cell")
            elif kind == "not-declared":
                for token in tokens:
                    if token in context["decls"] or token in context["owned"]:
                        fail(block_id, text, f"`{token}` is a type this document declares")
            elif kind == "ownership-row":
                crate = normalize_crate(cells[0].strip("`")) if cells else ""
                if crate not in context["crates"]:
                    fail(block_id, text, f"`{crate}` is no crate of section 1.2")
            elif kind == "drop-name":
                for token in tokens:
                    if token not in context["decls"]:
                        continue
                    if token in context["external_paths"] and token in context["owned"]:
                        continue
                    fail(
                        block_id,
                        token,
                        "the drop list carries a name this document declares and no"
                        " external block maps",
                    )
            elif kind == "const-crate":
                head = re.match(r"\s*// (duet-[a-z]+)", text)
                if head and normalize_crate(head.group(1)) not in context["crates"]:
                    fail(block_id, text, f"`{head.group(1)}` is no crate of section 1.2")
            elif kind == "limit-row":
                if not tokens:
                    continue
                if tokens[0] not in context["constants"]:
                    fail(block_id, text, f"`{tokens[0]}` is no constant of section 1.6")
                for name in tokens[1:]:
                    if normalize_crate(name) not in context["crates"]:
                        fail(block_id, text, f"`{name}` is no crate of section 1.2")
            elif kind == "rule-id":
                head = cells[0] if cells else ""
                for rule in re.findall(r"\b((?:PG|CG)\d+[a-z]?)\b", head):
                    if rule not in context["rules"]:
                        fail(block_id, text, f"`{rule}` is no rule this plan implements")
            elif kind == "block-id":
                if tokens and tokens[0] not in DATA_BLOCKS:
                    fail(block_id, text, f"`{tokens[0]}` is no registered block")
            elif kind == "verdict-name":
                for name in cell_names(cells[0]) if cells else []:
                    if name in context["owned"]:
                        fail(block_id, name, "section 1.5 places this name, so no row decides it")
            elif kind == "external-name":
                if tokens and tokens[0] not in context["external_universe"]:
                    fail(block_id, text, f"`{tokens[0]}` is named in no other block")
            elif kind == "expr-head":
                head = expr_head(tokens[0]) if tokens else None
                if head and head not in context["size_universe"]:
                    fail(block_id, text, f"`{head}` is no type this document knows")
            elif kind == "sub-id":
                if not tokens or not re.fullmatch(r"S\d+", tokens[0]):
                    fail(block_id, text, "the row states no substitution id")
            elif kind == "review-id":
                # The row is one finding id, one summary, one state, and one
                # section. `closure_check.py` holds the id list against the
                # review file; this rule holds the SHAPE, so a damaged row is
                # red here and a missing row is red there.
                # The id carries an optional FILE TAG after the revision
                # number, because two reviews of one revision both generate
                # `N21-n` and the two blocks would collide (critic N21-1).
                # `critic-spec-r21-inner.md` generates `C21I-1` and
                # `critic-spec-r21.md` generates `C21-1`.
                head = cells[0].strip() if cells else ""
                if not re.fullmatch(r"[CN]\d+[A-Z]?-W?\d+", head):
                    fail(block_id, text, "the row states no finding id")
                elif len(cells) < 4 or not cells[3].strip():
                    fail(block_id, head, "the row names no section")
            elif kind == "chunk-pair":
                # Two chunk ids and a reason. Both ids must be chunks section
                # 13.2 declares, so an exemption cannot name a chunk that does
                # not exist.
                for chunk in tokens[:2]:
                    if chunk not in context["chunks"]:
                        fail(block_id, text, f'`{chunk}` is no chunk of section 13.2')
            elif kind == "line-owner":
                # One chunk line and the crate it owns. The crate must be a
                # crate section 1.2 carries, so the map cannot name a line
                # over a crate this plan does not build.
                if len(tokens) < 2 or normalize_crate(tokens[1]) not in context["crates"]:
                    fail(block_id, text, "the row names no crate of section 1.2")
            elif kind == "first-declared":
                # The row is one declared name plus a reason in prose, so the
                # rule reads the first token and never the reason.
                head = tokens[0] if tokens else ""
                if head not in context["decls"]:
                    fail(block_id, head, "no Rust block of this document declares it")
            elif kind == "declared-name":
                names = cell_names(cells[0]) if cells else tokens
                for name in names:
                    if name not in context["decls"]:
                        fail(block_id, name, "no Rust block of this document declares it")
            elif kind == "impl-site":
                # The first token is the declaration the impl is for. The
                # trait is the rest of the line and no rule reads it, because
                # a trait path may name a crate this document places nowhere;
                # the block's own site states that limit (concern N-3).
                head = tokens[0] if tokens else ""
                if head not in context["decls"]:
                    fail(block_id, text, f"`{head}` is no declaration of this document")
            elif kind == "primitive-name":
                # The two primitive blocks are one set in both directions. A
                # sized primitive carries a size row and is `Copy`; `str` is
                # unsized, carries no size row, and supplies no `Copy`.
                head = tokens[0] if tokens else ""
                if block_id == "primitive-sizes":
                    if head not in context["primitive_traits"]:
                        fail(block_id, text, f"`{head}` is in no primitive trait set")
                elif "Copy" in text and head not in context["primitive_sizes"]:
                    fail(block_id, text, f"`{head}` is a sized primitive with no size row")
            elif kind == "unknown-name":
                for name in cell_names(cells[2]) if len(cells) > 2 else []:
                    if name not in context["framework"] and name not in context["decls"]:
                        fail(block_id, name, "it is neither a framework name nor a declared name")
            elif kind == "field-path":
                # The row is `Type.field Head reason`. The holder must be an
                # audio-owned name, the field must be one it declares, and the
                # head must be a name this document or an external block
                # decides, because the head is what the rule compares.
                if not tokens or "." not in tokens[0]:
                    fail(block_id, text, "the row states no `Type.field` path")
                    continue
                holder, _dot, field = tokens[0].partition(".")
                head = tokens[1] if len(tokens) > 1 else ""
                if holder not in context["audio_roots"]:
                    fail(block_id, text, f"`{holder}` is in no audio-owned block")
                elif field not in context["fields"].get(holder, set()):
                    fail(block_id, text, f"`{holder}` declares no field `{field}`")
                elif not head:
                    fail(block_id, text, "the row names no head type")
                elif head not in context["externals"] and head not in context["decls"]:
                    fail(block_id, text, f"`{head}` is no type this document decides")
            elif kind == "fault-arm":
                # The one message table of section 12.4 (critic C21-W4). Each
                # row keys on an arm of `EngineFault`, so a variant added with
                # no row, and a row for an arm the enum does not hold, are
                # each a failure.
                arm = re.match(r"`([A-Za-z_][A-Za-z0-9_]*)", cells[0] if cells else "")
                arms = {name for name, _body in context["arms"].get("EngineFault", [])}
                if arm is None:
                    fail(block_id, text, "the row names no `EngineFault` arm")
                elif arm.group(1) not in arms:
                    fail(block_id, arm.group(1), "`EngineFault` declares no such arm")
            elif kind == "gate-site":
                # The SITE column of the planted gate-defect table. Each row
                # names the declaration the plant goes into, and that name is
                # a type or a function this document declares, so a row that
                # names a site the document lost is red (critic C19-W8).
                site = cells[1] if len(cells) > 1 else ""
                names = [
                    token
                    for token in re.findall(r"`([^`]+)`", site)
                ]
                leaves = set()
                for token in names:
                    for leaf in re.findall(r"\b([A-Z][A-Za-z0-9_]*)\b", token):
                        leaves.add(leaf)
                    for leaf in re.findall(r"\b([a-z_][a-z0-9_]*)\b", token.split("::")[-1]):
                        leaves.add(leaf)
                known = set(context["decls"]) | context["functions"]
                if not names:
                    fail(block_id, text, "the row names no site")
                elif not (leaves & known):
                    fail(
                        block_id,
                        site,
                        "no declaration and no function of this document carries the site",
                    )
            elif kind == "cited-elsewhere":
                site = cells[0].strip("`") if cells else text
                if context["source"].count(site) < 2:
                    fail(block_id, site, "no other line of this document names it")
            elif kind == "site-crate":
                site = cells[0].strip("`") if cells else text
                crate = normalize_crate(site.split("::", 1)[0])
                if crate not in context["crates"]:
                    fail(block_id, site, f"`{crate}` is no crate of section 1.2")
            elif kind == "site-path":
                site = cells[0].strip("`") if cells else text
                crate = normalize_crate(site.split("::", 1)[0])
                leaf = site.rsplit("::", 1)[-1]
                if crate not in context["crates"]:
                    fail(block_id, site, f"`{crate}` is no crate of section 1.2")
                elif leaf not in context["decls"]:
                    fail(block_id, site, f"`{leaf}` is no declared type")
            elif kind == "chunk-id":
                # The owner cell and the line-chunk cell hold chunk ids. Every
                # other cell holds prose, and a rule id such as SM4 has the
                # shape of a chunk id, so the rule reads the two cells alone.
                joined = " ".join(cells[i] for i in (1, 3) if i < len(cells)) if cells else text
                for chunk in re.findall(r"\b([A-Z]{1,2}\d{1,2})\b", joined):
                    if chunk not in context["chunks"]:
                        fail(block_id, text, f"`{chunk}` is no chunk of section 13.2")
            elif kind == "carrier-end":
                # TH13. The Sender cell and the Receiver cell each name one
                # `Type.field` path. The type must be a type a Rust block of
                # this document declares and the field must be one that type
                # declares, which is the `field-path` shape applied to the
                # carrier table (PG41, critic C22I-2).
                for index in (1, 3):
                    cell = cells[index] if len(cells) > index else ""
                    found = re.findall(r"`([A-Za-z_][A-Za-z0-9_]*)\.([A-Za-z_][A-Za-z0-9_]*)`", cell)
                    if not found:
                        fail(block_id, text, f"cell {index} names no `Type.field` end")
                        continue
                    for holder, field in found:
                        if holder not in context["decls"]:
                            fail(block_id, text, f"`{holder}` is no declaration of this document")
                        elif field not in context["fields"].get(holder, set()):
                            fail(block_id, text, f"`{holder}` declares no field `{field}`")
            elif kind == "mechanism-name":
                cell = cells[2] if len(cells) > 2 else text
                for name in re.findall(r"\b([A-Z][A-Za-z0-9_]+)\b", cell):
                    if (
                        name in context["decls"]
                        or name in context["externals"]
                        or name in context["framework"]
                    ):
                        continue
                    fail(block_id, text, f"`{name}` is no type this document knows")
    return checked, failures


def label_marks(source):
    """Every citation label of this document, as (offset, label) (PG30).

    A label is what a section 1.6 `Used by` cell writes: `5.5`, `14`, `B.5`,
    `C.13`, or `Appendix A`. A `####` heading carries no label of its own and
    inherits the one above it.
    """
    marks = []
    for match in re.finditer(r"(?m)^#{2,3} (.+)$", source):
        title = match.group(1).strip()
        label = None
        if title.startswith("Appendix A"):
            label = "Appendix A"
        else:
            head = re.match(r"([0-9]+\.[0-9]+[a-z]?|[0-9]+|[BC]\.[0-9]+)[.:]? ", title)
            if head:
                label = head.group(1)
        if label:
            marks.append((match.start(), label))
    return marks


# Every appendix section that records a review's closure rows. PG30 skips a
# citation inside one, because a closure row names the id of the finding and
# never uses the value (critic C18-N4).
# The heading of every `#### ` block whose marker id opens with `closure-`,
# mapped to the `### C.n` section it sits in. A closure row cites the section
# that closed a finding, and that citation is a reference and not a use site,
# so PG30 skips these (critic N18-4).
CLOSURE_HEADING = re.compile(
    r"(?m)^###\s+(C\.\d+)\b(?:(?!^###\s).)*?<!--\s*GUARD BLOCK id=closure-",
    re.S,
)


def closure_sections(source):
    """Every Appendix C section that registers a closure block (PG30).

    Revision 21 held this set as a literal tuple that tracked the document,
    and N19-2 and N21-7 both recorded it as debt: a closure section added with
    no edit here put every `B` id its rows quote back into PG30's citation
    set, and a section deleted here silenced the rule over a live section.
    The set is read from the document now, exactly as `implemented_rule_ids`
    is read from the prototypes (critic N21-7).
    """
    return {match.group(1) for match in CLOSURE_HEADING.finditer(source)}


def table_rows_of(source):
    """Every markdown table of this document, outside every fenced block.

    Each table is a list of `(line number, text)` pairs, the header first.
    """
    fenced, found, current = False, [], None
    for number, line in enumerate(source.splitlines(), 1):
        stripped = line.strip()
        if stripped.startswith("```") or stripped.startswith("~~~"):
            fenced = not fenced
            continue
        if fenced:
            continue
        if stripped.startswith("|") and stripped.endswith("|"):
            current = [(number, stripped)] if current is None else current + [(number, stripped)]
            continue
        if current:
            found.append(current)
        current = None
    if current:
        found.append(current)
    return found


def cell_count(row):
    """How many cells one markdown row holds.

    A renderer splits on every `|` the author did not escape, so this splits
    on the same rule. `\\|` inside a backtick span is one escaped pipe and not
    a cell wall, which is exactly the distinction the defect turned on.
    """
    return len(re.split(r"(?<!\\)\|", row)) - 2


def ragged_row_audit(source):
    """PG38. A table row whose cell count differs from its own header.

    A GitHub-flavoured renderer keeps the first `n` cells of a ragged row and
    DROPS the rest, so an unescaped `|` inside a backtick span silently
    truncates the row. Revision 21 carried two: chunk M0's section 13.1 row
    parsed to seven cells under a five-column header, which dropped its whole
    write scope and its Completion command, and the C20-W6 closure row parsed
    to six under four. **No rule of revision 21 could see either** (critic
    C21-5): section 13.1 is not a registered block, so DR7 gave it no marker,
    and `closure_check.py` read `row[3]`, found a token that looked like a
    section number, and passed CL4 on the truncated cell.

    **The rule covers EVERY markdown table of this document**, and not the
    section 13 tables and the closure blocks alone. The document holds 118
    tables and not one ragged row, so the wider scope costs nothing and the
    narrow scope would have left the next such defect to the next reviewer.

    **The rule's own limit, stated here**: it decides the CELL COUNT of a row
    and never the content of a cell. A row that holds the right number of
    cells and the wrong text is outside it, and CL4 and the membership rules
    are what read content.
    """
    failures, checked = [], 0
    for table in table_rows_of(source):
        if len(table) < 2:
            continue
        width = cell_count(table[0][1])
        for number, row in table[1:]:
            if not re.fullmatch(r"[|:\- ]+", row):
                checked += 1
                found = cell_count(row)
                if found != width:
                    # The row's own first cell names it, and a LINE NUMBER
                    # would make the recorded probe text stale on every edit
                    # above it.
                    head = split_row(row)
                    label = (head[0] if head else "").strip("`* ")[:24] or f"line {number}"
                    failures.append(
                        (
                            label,
                            f"the row holds {found} cells and its header holds {width};"
                            " a renderer drops the extra cells and truncates the row"
                            " (PG38)",
                        )
                    )
    return checked, failures


# The families section 1.7 indexes, mapped to the set PG39 holds each one
# against. A family this map does not hold is a family the rule declines.
INDEX_FAMILIES = ("PG", "PP", "CG", "CP")


def expand_index_ids(cell):
    """Every id one section 1.7 cell names, with its ranges expanded.

    `PG1 to PG37` is a numeric range, `PG26b to PG26f` is a letter range over
    one number, and every other token is one id.
    """
    found, text = set(), cell.replace("\u00a0", " ")
    for opening, closing in re.findall(
        r"\b([A-Z]{2}\d+[a-z]?)\s+to\s+([A-Z]{2}\d+[a-z]?)\b", text
    ):
        first = re.fullmatch(r"([A-Z]{2})(\d+)([a-z]?)", opening)
        last = re.fullmatch(r"([A-Z]{2})(\d+)([a-z]?)", closing)
        if first is None or last is None or first.group(1) != last.group(1):
            continue
        prefix = first.group(1)
        if first.group(3) and last.group(3) and first.group(2) == last.group(2):
            for letter in range(ord(first.group(3)), ord(last.group(3)) + 1):
                found.add(f"{prefix}{first.group(2)}{chr(letter)}")
        else:
            for number in range(int(first.group(2)), int(last.group(2)) + 1):
                found.add(f"{prefix}{number}")
        text = text.replace(f"{opening} to {closing}", " ")
    found |= set(re.findall(r"\b([A-Z]{2}\d+[a-z]?)\b", text))
    return found


def rule_index_audit(source, rule_ids, probe_ids):
    """PG39. Section 1.7 and the live rule set are one set.

    DR4 makes section 1.7 the ONE index of rule ids, and no rule read it, so
    revision 21's index omitted PG27b, PG37, PP27b and PP37 and named PG26b,
    PG26c and PG26d as "the three TH1 rules" where section 5.7 names PG26,
    PG26b and PG26c (critic C21-W7). An index nothing reads is prose that
    ages, which is the class DR3 and DR5 already removed elsewhere.

    The rule expands every range of the index, takes the PG and CG ids from
    the prototypes and the PP and CP ids from the probe table, and fails on an
    id in one set and not the other, in both directions.

    **The rule's own limit, stated here**: it reads the four guard families
    and declines DR, PL, VR, TH and SM, because no machine holds those ids and
    only a reader can. The run prints `INDEX IDS` so the denominator is
    readable.
    """
    table = re.search(
        r"(?m)^\| Id \| Rule, in three words \| Stated in \|\n\|[-| ]+\|\n((?:\|.*\n)+)",
        source,
    )
    if table is None:
        return 0, [("<1.7>", "the rule index table is absent; the rule is fail-closed")]
    indexed = set()
    for line in table.group(1).splitlines():
        parts = [cell.strip() for cell in line.split("|")[1:-1]]
        if len(parts) < 3:
            continue
        indexed |= expand_index_ids(parts[0])
    live = set(rule_ids) | set(probe_ids)
    failures = []
    for family in INDEX_FAMILIES:
        stated = {name for name in indexed if name.startswith(family)}
        actual = {name for name in live if re.fullmatch(family + r"\d+[a-z]?", name)}
        for name in sorted(actual - stated):
            failures.append((name, "the rule set holds it and section 1.7 omits it"))
        for name in sorted(stated - actual):
            failures.append((name, "section 1.7 names it and the rule set holds no such id"))
    return len(indexed), failures


CHUNK_ROW = re.compile(r"(?m)^\| ([A-Z]+\d*) \| (\d+) \|(.*)$")


def chunk_crate_audit(source, blocks):
    """PG40. A chunk writes only into the crate its own LINE owns.

    N19-4 recorded this as accepted debt for three revisions: SM4 and SM5 both
    rest on the Writes columns, and no rule held a chunk's write scope to the
    crate its line owns, so a chunk could take a path inside a crate another
    line is building in the same phase and every guard stayed green. The
    `line-map` block is the data the debt row named, and this is the rule that
    reads it.

    The rule takes the line of each chunk id, takes that line's crate from the
    `line-map` block, and fails on a `crates/<name>/` path in the chunk's row
    whose `<name>` is not that crate. **The M chunks are outside it**: a
    manifest chunk creates the skeleton of every crate whose first chunk runs
    in its phase, which SM1 states, so a manifest row names several crates by
    design and the line map gives it none.

    **The rule's own limit, stated here**: it reads a path that opens
    `crates/`, so a chunk that writes `tools/`, `scripts/` or a workflow file
    is outside it and SM4 is what holds those. It also reads the ROW and not
    the file system, exactly as PG35 does.
    """
    owners = {}
    for row in blocks["line-map"]:
        parts = (row[0] if isinstance(row, list) else row).split()
        if len(parts) >= 2:
            owners[parts[0]] = parts[1]
    if not owners:
        return 0, [("<line-map>", "the line map holds no row, so PG40 has no owner set")]
    checked, failures = 0, []
    for match in CHUNK_ROW.finditer(source):
        # A section 13.2 chunk row holds five cells: the id, the phase, the
        # goal, the Writes column and the Completion command. A section 1.6
        # budget row opens with the same shape and holds four, so the cell
        # count is what separates `A2` from `B28` (critic C22I-W3).
        if cell_count(match.group(0)) != 5:
            continue
        chunk, rest = match.group(1), match.group(3)
        head = re.match(r"[A-Z]+", chunk).group(0)
        owner = owners.get(chunk) or owners.get(head)
        if owner is None:
            # N21-3 forced a floor on PG37 for this class, and revision 22
            # shipped PG40 with neither a floor nor a refusal: an unmapped
            # prefix was skipped in silence, whatever the row wrote (critic
            # C22I-W3). An M chunk is the one designed exception and the map
            # gives it no row, so the rule names it here and refuses every
            # other unmapped id.
            if not re.fullmatch(r"M\d*", chunk):
                failures.append(
                    (
                        chunk,
                        "the `line-map` block carries no line for this chunk id, so no"
                        " crate owns its write scope (PG40)",
                    )
                )
            continue
        checked += 1
        for name in sorted(set(re.findall(r"crates/([a-z0-9-]+)/", rest))):
            if name != owner:
                failures.append(
                    (
                        chunk,
                        f"the row writes under `crates/{name}/` and line `{head}` owns"
                        f" `{owner}` (PG40)",
                    )
                )
    return checked, failures


def used_by_audit(source, blocks):
    """PG30. Every `B` citation against the section 1.6 `Used by` column.

    The column's own site calls itself a fact and not a claim. The forward
    half held for three revisions and the reverse half did not: revision 11
    carried fifteen cells that named a section which never cited the id,
    revision 12 carried two, and revision 13 carried fourteen. A prose fix
    failed three times, so the rule is the answer (critic C-1).

    **The rule's own limit, stated here**: it reads this document alone, so a
    citation inside an ADR is outside it; it skips the budget table itself,
    because one budget row that derives from another is a derivation and not a
    use; and it skips every closure appendix of Appendix C, because a closure
    row records the finding that created a budget and never uses its value
    (critic C18-N4).

    It returns `(rows, [(id, reason), ...])`.
    """
    table = re.search(
        r"(?m)^\| Id \| Value \| What it bounds \| Used by \|\n\|[-| ]+\|\n((?:\|.*\n)+)",
        source,
    )
    if not table:
        return 0, [("<1.6>", "the budget table is absent")]
    skipped_sections = closure_sections(source)
    if not skipped_sections:
        return 0, [("<C>", "Appendix C registers no closure block, so PG30 has no skip set")]
    cells = {}
    for line in table.group(1).splitlines():
        parts = [cell.strip() for cell in line.split("|")[1:-1]]
        if len(parts) < 4 or not re.fullmatch(r"B\d+", parts[0]):
            continue
        # An `ADR 0006` reference names a decision record and not a section
        # of this document, so it leaves the cell before the labels are read.
        cell = re.sub(r"ADR \d+", "", parts[3])
        labels = set(re.findall(r"(?<![\w.])(\d+\.\d+[a-z]?|[BC]\.\d+)(?![\w.])", cell))
        labels |= {token for token in re.findall(r"(?<![\w.])(\d+)(?![\w.])", cell)}
        if "Appendix A" in cell:
            labels.add("Appendix A")
        cells[parts[0]] = labels
    masked = source[: table.start()] + (" " * (table.end() - table.start())) + source[table.end():]
    marks = label_marks(source)
    cited = {}
    for match in re.finditer(r"(?<![\w.])B(\d+)(?![\w])", masked):
        label = section_at(marks, match.start())
        # A CLOSURE appendix mentions an id inside a closure row; that is a
        # record of the finding that created the budget and not a site that
        # uses the value (critic C18-N4). N16-7 asked for real use sites, and
        # counting a closure row gave B114 and B115 one real site each while
        # the cell claimed three. The budget table and an ADR reference are
        # skipped above for the same kind of reason.
        if label is None or label == "1.7" or label in skipped_sections:
            continue
        cited.setdefault("B" + match.group(1), set()).add(label)
    failures = []
    for budget in sorted(cells, key=lambda name: int(name[1:])):
        stated, real = cells[budget], cited.get(budget, set())
        for label in sorted(real - stated):
            failures.append((budget, f"section {label} cites it and the `Used by` cell omits it"))
        for label in sorted(stated - real):
            failures.append((budget, f"the `Used by` cell names section {label} and it cites nothing"))
    for budget in sorted(set(cited) - set(cells)):
        failures.append((budget, "no section 1.6 row declares this id"))
    return len(cells), failures


# Every type-expression head that names one end of a cross-thread carrier
# (TH13, PG41). A field whose expression names one of these holds a channel
# end, a ring end, a queue, or a publication end, and the carrier table must
# carry a row for it.
# Each head is mapped to the END it is: `write`, `read`, or `share` for a
# queue behind an `Arc`, whose two ends are one value. PG41 reads the map, so
# a row that swaps its Sender cell and its Receiver cell is red (critic
# C23I-W1).
CARRIER_HEADS = {
    "Sender": "write",
    "SyncSender": "write",
    "Producer": "write",
    "Input": "write",
    "OneshotSender": "write",
    "Receiver": "read",
    "CoreReceiver": "read",
    "Consumer": "read",
    "Output": "read",
    "OneshotReceiver": "read",
    "ArrayQueue": "share",
}


def carrier_end_kind(expr):
    """Which end one type expression holds, or `None` (PG41).

    A `triple_buffer::Input` is a write end and an `Output` is a read end, and
    the same split holds for the two `rtrb` ends, the two `async_channel`
    ends and the two `oneshot` ends. A `crossbeam_queue::ArrayQueue` behind an
    `Arc` is ONE value that both ends share, so it answers `share` and a row
    may name it on either side.
    """
    for head, kind in CARRIER_HEADS.items():
        if re.search(r"\b" + head + r"\b", expr):
            return kind
    return None

# The five primitives section 5.8 decides. A carrier cell names one of them,
# and every one but `triple_buffer` also names a B id, because a triple
# buffer's bound is three slots by construction and no budget row states it.
CARRIER_PRIMITIVES = (
    "triple_buffer",
    "rtrb",
    "ArrayQueue",
    "async_channel",
    "sync_channel",
    "oneshot",
)


def thread_names(source):
    """Every thread the section 5.7 thread table declares, as a set (PG41)."""
    table = re.search(
        r"(?m)^\| Thread \| Owner \| Owns \| Never does \|\n\|[-| ]+\|\n((?:\|.*\n)+)",
        source,
    )
    if table is None:
        return set()
    found = set()
    for line in table.group(1).splitlines():
        cells = split_row(line)
        if cells:
            found.add(cells[0].strip().strip("`*"))
    return found


def carrier_audit(source, blocks, raw_decls, arms, budgets):
    """PG41. Every cross-thread carrier has two declared ends (TH13).

    All eight Criticals of the twenty-second inner review were one class: a
    stated duty whose carrier no declaration holds. The engine reported six
    kinds of `EngineEvent` to the core over no declared channel, the audio
    thread published three snapshots through one declared write end out of
    seven, the core sent every `ConfigureCommand` with no declared sender, the
    disk thread had no declared type at all, the audio thread silenced a
    departed port's held notes with no field and no message, and a view read a
    slot measurement through no accessor. The class was invisible to all
    sixty-three rules of revision 22 (critic C22I-1, C22I-2, C22I-3, C22I-7,
    C22I-8).

    The rule runs in both directions. Forward: every row names a carrier, a B
    id where the primitive needs one, two threads the section 5.7 table
    declares, and an overflow rule. Reverse: every declared field whose type
    expression names a carrier head is an end of exactly one row.

    **The rule's own limit, stated at its section 1.5 site**: it reads the
    ENDS and never the bodies, and it declines an owned handoff that is no
    channel.

    It returns `(rows, fields, [(subject, reason), ...])`.
    """
    rows = blocks["carrier-table"]
    threads = thread_names(source)
    if not threads:
        return 0, 0, [("<5.7>", "the thread table does not read, so PG41 has no thread set")]
    # The declared carrier field set, built BEFORE the rows are read, so each
    # row's two ends can be tested against it (critic C23I-W1). Revision 23's
    # first form read the cells into a dictionary and never compared one with
    # this set, so a row could name a real field that holds no carrier and a
    # row could name its two ends in the wrong order.
    ends = {}
    for holder in sorted(raw_decls):
        for field, expr in walk_fields(holder, raw_decls, arms):
            kind = carrier_end_kind(expr)
            if kind is not None:
                ends[f"{holder}.{field}"] = (kind, expr)
    named, failures = {}, []
    for cells in rows:
        if len(cells) < 6:
            failures.append(("<row>", "the row holds fewer than six cells"))
            continue
        message = cells[0].strip().strip("`")
        for index, wanted in ((1, "write"), (3, "read")):
            for holder, field in re.findall(
                r"`([A-Za-z_][A-Za-z0-9_]*)\.([A-Za-z_][A-Za-z0-9_]*)`", cells[index]
            ):
                path = f"{holder}.{field}"
                named.setdefault(path, set()).add(message)
                if path not in ends:
                    failures.append(
                        (
                            message,
                            f"`{path}` holds no carrier end, so it cannot be the"
                            f" {wanted} end of this row",
                        )
                    )
                elif ends[path][0] not in (wanted, "share"):
                    failures.append(
                        (
                            message,
                            f"`{path}` is a {ends[path][0]} end, `{ends[path][1]}`, and this"
                            f" cell is the {wanted} end",
                        )
                    )
        carrier = cells[2]
        primitives = [name for name in CARRIER_PRIMITIVES if name in carrier]
        if not primitives:
            failures.append((message, "the carrier cell names no primitive section 5.8 decides"))
        elif primitives != ["triple_buffer"] and not re.search(r"\bB\d+\b", carrier):
            failures.append((message, "the carrier cell names no B id of section 1.6"))
        for budget in re.findall(r"\bB(\d+)\b", carrier):
            if "B" + budget not in budgets:
                failures.append((message, f"the carrier cell names B{budget} and no section 1.6 row declares it"))
        halves = cells[4].split("->")
        if len(halves) != 2:
            failures.append((message, "the thread cell states no `<sender> -> <receiver>` pair"))
        else:
            for half in halves:
                for name in half.split(" and "):
                    name = name.strip().strip("`*")
                    if name and name not in threads:
                        failures.append((message, f"`{name}` is no thread of the section 5.7 table"))
        if not cells[5].strip():
            failures.append((message, "the row states no overflow or expiry rule"))
    for path, (_kind, expr) in sorted(ends.items()):
        if path not in named:
            failures.append(
                (
                    path,
                    f"the field holds a carrier end, `{expr}`, and no row of the"
                    " carrier table names it (TH13)",
                )
            )
        elif len(named[path]) > 1:
            failures.append(
                (path, "two carrier rows name this end, and one end carries one message")
            )
    fields = len(ends)
    return len(rows), fields, failures


def main(argv):
    """Run every claim and return a process exit code."""
    if len(argv) != 2:
        sys.stdout.write("usage: placement_check.py <architecture.md>\n")
        return 2
    path = argv[1]
    source = read_document(path)
    if source is None:
        sys.stdout.write(f"FAIL: cannot open {path}; the guard is fail-closed.\n")
        return 2

    blocks, block_bad = read_blocks(source)
    block_bad = block_bad + stray_markers(source)
    for block_id, reason in block_bad:
        sys.stdout.write(
            f"FAIL: the `{block_id}` block of this document: {reason};"
            " the guard is fail-closed (DR7).\n"
        )
    if block_bad:
        sys.stdout.write(
            f"BLOCKS:          {len(DATA_BLOCKS)}     BLOCK BAD: {len(block_bad)}\n"
        )
        return 2

    floor_bad = floor_audit(source, blocks)
    framework = framework_names(blocks)
    externals, external_traits = external_verdicts(blocks)
    heap = set(HEAP_NAMES) | heap_externals(blocks) | set(token_rows(blocks["heap-names"]))
    defer = deferring_externals(blocks)
    grows = set(GROW_NAMES) | grow_externals(blocks) | set(token_rows(blocks["grow-names"]))
    locks = set(LOCK_NAMES) | lock_externals(blocks) | set(token_rows(blocks["lock-names"]))
    reads = read_externals(blocks)
    dropped = candidate_drop_list(blocks)
    primitives = primitive_traits(blocks)
    sizes = primitive_sizes(blocks)
    owned, declared_in = ownership_table(blocks)
    rows = dependency_table(blocks)
    mapping, mapping_bad = name_map(blocks, rows)
    for line in mapping_bad:
        sys.stdout.write(
            f"FAIL: the name map holds a row the guard cannot resolve: {line};"
            " the guard is fail-closed (critic WR-16).\n"
        )
    if mapping_bad:
        return 2
    audio_roots, audio_bad = audio_owned(blocks, owned)
    exempt, exempt_bad = audio_exempt(blocks, owned)
    drops, drop_bad = drop_impl_names(blocks, owned)
    for token in audio_bad:
        sys.stdout.write(
            f"FAIL: the audio-owned block holds `{token}`, which section 1.5 places nowhere;"
            " the guard is fail-closed (critic CR-14).\n"
        )
    for line in exempt_bad:
        sys.stdout.write(
            f"FAIL: the audio-exempt block holds `{line}`, which is no `Type.field` path;"
            " the guard is fail-closed.\n"
        )
    for token in drop_bad:
        sys.stdout.write(
            f"FAIL: the Drop block holds `{token}`, which section 1.5 places nowhere;"
            " the guard is fail-closed.\n"
        )
    if audio_bad or exempt_bad or drop_bad:
        return 2
    edges = edge_list(blocks)
    closure = reachable(edges)
    impls = hand_impls(source)
    marks = section_of(source)

    declared_counts = {}
    all_decls = {}
    raw_decls = {}
    block_sections = {}
    used = set()
    for offset, block in rust_blocks(source):
        section = section_at(marks, offset)
        raw = strip_comments(block)
        code = strip_paths(raw)
        for name, entries in declarations(code).items():
            declared_counts[name] = declared_counts.get(name, 0) + len(entries)
            all_decls.setdefault(name, []).extend(entries)
            block_sections.setdefault(name, section)
        for name, entries in declarations(raw).items():
            raw_decls.setdefault(name, []).extend(entries)
        used |= used_types(mask_enum_arm_names(code))

    candidates = {
        name
        for name in (set(declared_counts) | used)
        if name not in dropped and len(name) > 1 and not name.isupper()
    }

    known = set(owned) | set(externals)
    is_copy = copy_verdicts(all_decls, externals, impls)
    constants_by_crate = constant_owners(blocks)

    unplaced = sorted(name for name in candidates if name not in owned and name not in framework)
    duplicated = sorted(name for name, count in declared_counts.items() if count > 1)
    misclaimed = sorted(name for name in framework if name in owned and owned[name] not in APP_ROWS)
    misused = framework_misuse(raw_decls, owned, framework)
    copy_missing, copy_impossible, copy_undecided, copy_unknown = copy_audit(
        all_decls, owned, is_copy, externals, known, drops
    )
    undeclared = placed_without_declaration(owned, all_decls)
    external_misses = external_audit(all_decls, owned, framework, externals, known)
    edge_misses = edge_claims(source, edges)
    dependency_misses, dependency_proven = dependency_audit(raw_decls, owned, rows, mapping)
    snapshot_misses, snapshots = snapshot_audit(blocks, all_decls, is_copy)
    register_misses = register_audit(block_sections, owned, declared_in)
    test_misses, selected = selected_tests(source, blocks)
    link_bad = link_audit(source, blocks) + line_phase_audit(blocks)
    exempt_pairs, exempt_reason_bad = {}, []
    for row in blocks["phase-pair-exempt"]:
        tokens = row.split()
        if len(tokens) < 3:
            continue
        pair = " ".join(tokens[:2])
        reason = " ".join(tokens[2:])
        exempt_pairs[pair] = reason
        # The reason is the thing a reviewer reads to accept the exemption, so
        # it must name both chunks of its OWN pair. Row 6 of revision 18 named
        # `C2` in the reason of the `C1 N2` pair, which cannot be reviewed
        # (critic C18-N1).
        for chunk in tokens[:2]:
            if not re.search(r"(?<![A-Za-z0-9])" + re.escape(chunk) + r"(?![0-9])", reason):
                exempt_reason_bad.append(
                    (pair, f"the reason does not name `{chunk}`, which is one of its own pair")
                )
    pair_found, pair_bad = phase_pair_audit(blocks, exempt_pairs)
    if len(exempt_pairs) > EXEMPT_CEILING:
        exempt_reason_bad = list(exempt_reason_bad) + [
            (
                "phase-pair-exempt",
                f"the block holds {len(exempt_pairs)} rows and the ceiling is"
                f" {EXEMPT_CEILING}; a twenty-fifth pair is a review decision and not"
                " one more row (N19-1)",
            )
        ]
    tail_bad = tail_phase_audit(source, blocks) + exempt_reason_bad
    suppression, suppression_reason = suppression_audit(source, blocks)
    if suppression is None:
        sys.stdout.write(f"FAIL: {suppression_reason}; the guard is fail-closed.\n")
        return 2
    b1_sites, b1_listed, b1_bad = suppression
    justified = justified_unknowns(blocks)
    unjustified = unknown_audit(copy_unknown, justified)
    closure_broken, closure_undecided = derive_closure_audit(
        raw_decls, owned, impls, external_traits, framework, primitives
    )
    reach_misses = reachability_audit(raw_decls, owned, closure)
    listed_expectations = expectation_tables(blocks)
    expectation_misses, carried_expectations = expectation_audit(raw_decls, listed_expectations)
    placeholders = comment_placeholders(source, owned)
    vr1_listed = vr1_table(blocks)
    vr1_misses = vr1_audit(raw_decls, owned, impls, vr1_listed)
    eq_missing, eq_undecided = eq_audit(
        raw_decls, owned, impls, external_traits, framework, primitives
    )
    arms = enum_arms_of(source)
    root_bad = audio_root_audit(audio_roots, audio_marked(source))
    reachable_set = audio_reachable(raw_decls, arms)
    asserted_roots = {
        line.split()[0] for line in blocks["audio-asserted"] if line.split()
    }
    asserted_bad = audio_asserted_audit(audio_roots, reachable_set, asserted_roots)
    leaf_names_allowed = {
        line.split()[0] for line in blocks["audio-reachable-leaf"] if line.split()
    }
    closure_bad = audio_closure_audit(audio_roots, reachable_set, leaf_names_allowed)
    heap_misses, grow_misses, lock_misses, heap_deferred, heap_readonly = heap_audit(
        raw_decls, owned, arms, audio_roots, heap, exempt, defer, grows, locks, reads
    )
    constants = usize_constants(blocks)
    recorded, heads = recorded_sizes(blocks)
    measure, explain = make_sizer(
        raw_decls, arms, recorded, heads, constants, sizes
    )
    size_rows, size_undecided, spread_bad, spread_expected = size_audit(
        raw_decls, owned, arms, constants, measure, explain
    )
    reason_count, reason_bad = reason_size_audit(
        attribute_texts(source, marks) + b1_reason_cells(blocks) + doc_comment_lines(source),
        recorded,
    )
    const_reach = constant_reach_audit(raw_decls, owned, closure, constants_by_crate)
    limit_rows, limit_bad = shared_limit_audit(blocks, closure, constants_by_crate)
    rule_ids, rule_reason = implemented_rule_ids()
    if rule_ids is None:
        sys.stdout.write(f"FAIL: {rule_reason}; the guard is fail-closed (PG29).\n")
        return 2
    probe_rows, probe_bad = probe_table_audit(blocks, rule_ids)
    lock_sequence, lock_bad = lock_sequence_audit(source, blocks)
    pin_rows, pin_bad = pin_owner_audit(source)
    used_rows, used_bad = used_by_audit(source, blocks)
    member_kinds, member_kind_bad = block_member_kinds(blocks)
    member_context = {
        "source": source,
        "crates": set(rows),
        "edge_crates": {normalize_crate(name) for name in edges}
        | {normalize_crate(target) for targets in edges.values() for target in targets},
        "third_party": {crate for entry in rows.values() for crate in entry},
        "decls": set(all_decls),
        "functions": set(declared_functions(source)),
        "arms": arms,
        "owned": set(owned),
        "framework": framework,
        "externals": set(externals),
        "external_paths": external_path_names(blocks),
        "constants": set(constants_by_crate),
        "rules": set(rule_ids),
        "audio_roots": audio_roots,
        "fields": {
            name: {field for field, _expr in walk_fields(name, raw_decls, arms)}
            for name in raw_decls
        },
        "chunks": chunk_ids(source),
        "primitive_traits": set(primitives),
        "primitive_sizes": set(sizes),
    }
    member_context["external_universe"] = (
        set(externals)
        | framework
        | set(mapping)
        | dropped
        | set(owned)
    )
    member_context["size_universe"] = (
        set(all_decls) | set(externals) | framework | set(sizes) | set(owned)
    )
    member_rows, member_bad = membership_audit(blocks, member_kinds, member_context)
    for block_id, reason in member_kind_bad:
        sys.stdout.write(
            f"FAIL: the `block-members` block of this document: `{block_id}`: {reason};"
            " the guard is fail-closed (DR7, critic WR-18).\n"
        )
    if member_kind_bad:
        return 2
    expectation_misses += [
        (name, "carries a missing_copy_implementations expectation and writes a Drop impl,"
               " which makes the expectation unfulfilled and the build fail (critic WR-12)")
        for name in sorted(drops & carried_expectations[EXPECTED_LINTS[0]])
    ]

    sys.stdout.write(f"DOCUMENT:        {path}\n")
    sys.stdout.write(f"BLOCKS:          {len(DATA_BLOCKS)}     BLOCK BAD: 0\n")
    sys.stdout.write(f"FRAMEWORK NAMES: {len(framework)}    NAME MAP: {len(mapping)}\n")
    sys.stdout.write(
        f"DROP LIST:       {len(dropped)}     PRIMITIVES: {len(primitives)}"
        f"     PRIMITIVE SIZES: {len(sizes)}\n"
    )
    sys.stdout.write(
        f"CANDIDATE TYPES: {len(candidates)}   DECLARED: {len(all_decls)}"
        f"   PLACEHOLDERS: {len(placeholders)}\n"
    )
    sys.stdout.write(f"TABLE NAMES:     {len(owned)}     UNDECLARED: {len(undeclared)}\n")
    sys.stdout.write(
        f"EXTERNAL ROWS:   {len(externals)}     EXTERNAL MISSING: {len(external_misses)}\n"
    )
    sys.stdout.write(f"PLACED:          {len(candidates) - len(unplaced)}\n")
    sys.stdout.write(f"UNPLACED:        {len(unplaced)}     DUPLICATED:   {len(duplicated)}\n")
    sys.stdout.write(f"MISCLAIMED:      {len(misclaimed)}     FRAMEWORK MISUSE: {len(misused)}\n")
    sys.stdout.write(
        f"COPY MISSING:    {len(copy_missing)}     COPY IMPOSSIBLE:  {len(copy_impossible)}\n"
    )
    sys.stdout.write(
        f"COPY UNDECIDED:  {len(copy_undecided)}     UNKNOWN: {len(copy_unknown)}"
        f"     JUSTIFIED: {len(justified)}     UNJUSTIFIED: {len(unjustified)}\n"
    )
    sys.stdout.write(
        f"DERIVE CLOSURE:  {len(closure_broken)}     DERIVE UNDECIDED: {len(closure_undecided)}\n"
    )
    sys.stdout.write(
        f"REACH BAD:       {len(reach_misses)}     CONST REACH BAD: {len(const_reach)}\n"
    )
    sys.stdout.write(f"LIMIT ROWS:      {limit_rows}     LIMIT BAD: {len(limit_bad)}\n")
    sys.stdout.write(f"PROBE ROWS:      {probe_rows}     PROBE BAD: {len(probe_bad)}\n")
    sys.stdout.write(
        f"RULE IDS:        {len(rule_ids)}     LOCK PHASES: {len(lock_sequence or ())}"
        f"     LOCK BAD: {len(lock_bad)}     PIN ROWS: {pin_rows}     PIN BAD: {len(pin_bad)}\n"
    )
    sys.stdout.write(
        f"MEMBER BLOCKS:   {len(member_kinds)}     MEMBER ROWS: {member_rows}"
        f"     MEMBER BAD: {len(member_bad)}\n"
    )
    sys.stdout.write(f"BUDGET ROWS:     {used_rows}     USED BY BAD: {len(used_bad)}\n")
    sys.stdout.write(
        f"EXPECTATIONS:    {len(carried_expectations[EXPECTED_LINTS[0]])}"
        f"     B.1 ROWS: {len(listed_expectations.get(EXPECTED_LINTS[0], ()))}"
        f"     B.1 BAD: {len(expectation_misses)}\n"
    )
    sys.stdout.write(
        f"VARIANT SITES:   {len(carried_expectations[EXPECTED_LINTS[1]])}"
        f"     B.1 VARIANT ROWS: {len(listed_expectations.get(EXPECTED_LINTS[1], ()))}\n"
    )
    sys.stdout.write(
        f"REASON SIZES:    {reason_count}     REASON BAD: {len(reason_bad)}\n"
    )
    sys.stdout.write(f"VR1 ROWS:        {len(vr1_listed)}     VR1 BAD: {len(vr1_misses)}\n")
    sys.stdout.write(
        f"EQ MISSING:     {len(eq_missing)}     EQ UNDECIDED: {len(eq_undecided)}\n"
    )
    # A size here states no niche, so it is an upper bound. Every shape a
    # niche decides is undecided instead, and `ROSTER_SIZES=1` names it.
    sys.stdout.write(
        f"SIZES DECIDED:  {len(size_rows)}     SIZE UNDECIDED: {len(size_undecided)}"
        f"     VARIANT SPREAD BAD: {len(spread_bad)}"
        f"     VARIANT EXPECTED: {len(spread_expected)}\n"
    )
    sys.stdout.write(
        f"AUDIO OWNED:     {len(audio_roots)}     AUDIO EXEMPT: {len(exempt)}"
        f"     AUDIO DEFERRED: {len(heap_deferred)}"
        f"     HEAP IN AUDIO: {len(heap_misses)}     DROP IMPLS: {len(drops)}\n"
    )
    sys.stdout.write(
        f"GROW IN AUDIO:   {len(grow_misses)}     LOCK IN AUDIO: {len(lock_misses)}"
        f"     AUDIO READ ONLY: {len(heap_readonly)}     ROOT BAD: {len(root_bad)}\n"
        f"AUDIO REACHABLE: {len(reachable_set)}     REACHABLE LEAVES: {len(leaf_names_allowed)}"
        f"     CLOSURE BAD: {len(closure_bad)}\n"
    )
    sys.stdout.write(
        f"EDGES PARSED:    {sum(len(v) for v in edges.values())}"
        f"    EDGE CLAIMS BAD: {len(edge_misses)}\n"
    )
    sys.stdout.write(
        f"DEP ROWS:        {len(rows)}    DEP PROVEN: {len(dependency_proven)}"
        f"    DEP MISSING: {len(dependency_misses)}\n"
    )
    sys.stdout.write(f"SNAPSHOTS:       {len(snapshots)}     SNAPSHOT BAD: {len(snapshot_misses)}\n")
    value_rows, value_bad, value_skipped = budget_value_audit(source, blocks)
    ragged_rows, ragged_bad = ragged_row_audit(source)
    chunk_rows, chunk_crate_bad = chunk_crate_audit(source, blocks)
    if chunk_rows < LINE_CHUNK_FLOOR:
        chunk_crate_bad = list(chunk_crate_bad) + [
            (
                "PG40",
                f"the rule decided {chunk_rows} chunk rows and its floor is"
                f" {LINE_CHUNK_FLOOR}; a decided set below the floor is a silent shrink"
                " of a denominator (critic C22I-W3)",
            )
        ]
    carrier_rows, carrier_fields, carrier_bad = carrier_audit(
        source, blocks, raw_decls, arms, {row[0].strip() for row in blocks["budget-table"] if row}
    )
    index_ids, index_bad = rule_index_audit(
        source,
        rule_ids,
        [cells[2].strip().strip("`") for cells in blocks["probe-table"] if len(cells) >= 5],
    )
    if value_rows < VALUE_FLOOR:
        value_bad = list(value_bad) + [
            (
                "PG37",
                f"the rule decided {value_rows} budget rows and its floor is"
                f" {VALUE_FLOOR}; a decided set below the floor is a silent shrink"
                " of a denominator (critic N21-3)",
            )
        ]
    sys.stdout.write(
        f"REGISTER BAD:    {len(register_misses)}     FLOOR SLACK: {len(floor_bad)}"
        f"     VALUE ROWS: {value_rows}     VALUE FLOOR: {VALUE_FLOOR}"
        f"     VALUE SKIPPED: {len(value_skipped)}"
        f"     VALUE BAD: {len(value_bad)}\n"
    )
    sys.stdout.write(
        f"TABLE ROWS:      {ragged_rows}     RAGGED ROWS: {len(ragged_bad)}"
        f"     INDEX IDS: {index_ids}     INDEX BAD: {len(index_bad)}\n"
    )
    sys.stdout.write(
        f"LINE CHUNKS:     {chunk_rows}     CHUNK CRATE BAD: {len(chunk_crate_bad)}"
        f"     LINE CHUNK FLOOR: {LINE_CHUNK_FLOOR}"
        f"     PAIR CEILING: {EXEMPT_CEILING}\n"
    )
    sys.stdout.write(
        f"CARRIERS:        {carrier_rows}     CARRIER FIELDS: {carrier_fields}"
        f"     CARRIER BAD: {len(carrier_bad)}\n"
    )
    for subject, reason in carrier_bad:
        sys.stdout.write(f"  CARRIER:    {subject}: {reason}\n")
    sys.stdout.write(f"TESTS SELECTED:  {len(selected)}     TEST ROWS BAD: {len(test_misses)}\n")
    sys.stdout.write(f"PLAN LINKS:      {len(link_rows(source))}     LINK BAD: {len(link_bad)}\n")
    sys.stdout.write(
        f"PHASE PAIRS:     {pair_found}     PAIR EXEMPT: {len(exempt_pairs)}"
        f"     PAIR BAD: {len(pair_bad)}     TAIL BAD: {len(tail_bad)}\n"
    )
    sys.stdout.write(
        f"B.1 SITES:       {len(b1_sites)}     2.3 LISTED: {len(b1_listed)}"
        f"     SUPPRESSION BAD: {len(b1_bad)}     ASSERTED ROOTS: {len(asserted_roots)}"
        f"     ASSERTED BAD: {len(asserted_bad)}\n"
    )
    if os.environ.get("ROSTER_SIZES") == "1":
        for name, crate, size, align, arm_sizes in size_rows:
            sys.stdout.write(f"  SIZE:       {crate}::{name} = {size} (align {align})\n")
            for arm_name, arm_size in arm_sizes:
                sys.stdout.write(f"  ARM:        {crate}::{name}::{arm_name} = {arm_size}\n")
        for name, crate, reasons in size_undecided:
            for field, expr in reasons:
                sys.stdout.write(f"  NO SIZE:    {crate}::{name}: {field} = {expr}\n")
    for name, crate in placeholders:
        sys.stdout.write(
            f"  PLACEHOLDER: {crate}::{name} has a comment where a field belongs\n"
        )
    for block_id, row, reason in member_bad:
        sys.stdout.write(f"  MEMBER:     {block_id}: {row}: {reason}\n")
    for budget, reason in used_bad:
        sys.stdout.write(f"  USED BY:    {budget}: {reason}\n")
    for label, reason in ragged_bad:
        sys.stdout.write(f"  RAGGED:     {label}: {reason}\n")
    for name, reason in index_bad:
        sys.stdout.write(f"  INDEX:      {name}: {reason}\n")
    for chunk, reason in chunk_crate_bad:
        sys.stdout.write(f"  CHUNK CRATE:{chunk}: {reason}\n")
    for name in undeclared:
        sys.stdout.write(f"  UNDECLARED: {name} is placed by 1.5 and no Rust block declares it\n")
    for name, token in external_misses:
        sys.stdout.write(f"  EXTERNAL:   {name} names {token}, which no 1.9 row decides\n")
    for name in unplaced:
        sys.stdout.write(f"  UNPLACED:   {name}\n")
    for name in duplicated:
        sys.stdout.write(f"  DUPLICATED: {name}\n")
    for name in misclaimed:
        sys.stdout.write(f"  MISCLAIMED: {name} claimed by {owned[name]}\n")
    for name, crate, token in misused:
        sys.stdout.write(f"  FRAMEWORK:  {crate}::{name} holds {token}\n")
    for name, crate in copy_missing:
        sys.stdout.write(f"  COPY:       {crate}::{name} needs Copy or an #[expect]\n")
    for name, crate, field in copy_impossible:
        sys.stdout.write(f"  NOT COPY:   {crate}::{name} derives Copy over {field}\n")
    for name, crate, field in copy_undecided:
        sys.stdout.write(f"  NOT DECIDED:{crate}::{name} derives Copy over {field}\n")
    for name, field in unjustified:
        sys.stdout.write(f"  UNJUSTIFIED:{name}.{field} is in no section 1.9 row\n")
    for name, crate, trait, field, expr in closure_broken:
        sys.stdout.write(
            f"  CLOSURE:    {crate}::{name} derives {trait} over {field}: {expr} has no {trait}\n"
        )
    for name, crate, field, expr in closure_undecided:
        sys.stdout.write(f"  UNREADABLE: {crate}::{name}.{field}: {expr}\n")
    for crate, name, token, target in reach_misses:
        sys.stdout.write(f"  REACH:      {crate}::{name} holds {token}, which lives in {target}\n")
    for name, reason in expectation_misses:
        sys.stdout.write(f"  B.1:        {name} {reason}\n")
    for site, count in reason_bad:
        sys.stdout.write(f"  REASON:     {site} states {count} bytes as a literal\n")
    for root, crate, expr, path in heap_misses:
        sys.stdout.write(f"  HEAP:       {crate}::{root} holds {expr} through {path}\n")
    for name, reason in root_bad:
        sys.stdout.write(f"  ROOT:       {name}: {reason}\n")
    for name, reason in closure_bad:
        sys.stdout.write(f"  CLOSURE:    {name}: {reason}\n")
    for name, reason in asserted_bad:
        sys.stdout.write(f"  ASSERTED:   {name}: {reason}\n")
    for name, reason in pair_bad:
        sys.stdout.write(f"  PAIR:       {name}: {reason}\n")
    for name, reason in tail_bad:
        sys.stdout.write(f"  TAIL:       {name}: {reason}\n")
    for name, reason in b1_bad:
        sys.stdout.write(f"  SUPPRESS:   {name}: {reason}\n")
    for link, reason in link_bad:
        sys.stdout.write(f"  LINK:       {link}: {reason}\n")
    for root, crate, expr, path in grow_misses:
        sys.stdout.write(f"  GROW:       {crate}::{root} holds {expr} through {path}\n")
    for root, crate, expr, path in lock_misses:
        sys.stdout.write(f"  LOCK:       {crate}::{root} holds {expr} through {path}\n")
    for name, crate, carried in vr1_misses:
        sys.stdout.write(
            f"  VR1:        {crate}::{name} derives {', '.join(carried)} with no VR1 row\n"
        )
    for name, crate in eq_missing:
        sys.stdout.write(
            f"  EQ:         {crate}::{name} derives PartialEq and every field supplies Eq\n"
        )
    for name, crate, arm_name, largest, second in spread_bad:
        sys.stdout.write(
            f"  VARIANT:    {crate}::{name}: arm {arm_name} is {largest} bytes"
            f" and the next largest is {second}\n"
        )
    for subject, target, sentence in edge_misses:
        sys.stdout.write(f"  EDGE MISS:  {subject} -> {target}: {sentence[:110]}\n")
    for crate, name, field, token, target in const_reach:
        sys.stdout.write(
            f"  CONST:      {crate}::{name}.{field} names {token}, which lives in {target}\n"
        )
    for constant, crate, reason in limit_bad:
        sys.stdout.write(f"  LIMIT:      {constant}: {crate}: {reason}\n")
    for rule, reason in probe_bad:
        sys.stdout.write(f"  PROBE:      {rule}: {reason}\n")
    for block_id, reason in floor_bad:
        sys.stdout.write(f"  FLOOR:      {block_id}: {reason} (PG27b)\n")
    for budget, reason in value_bad:
        sys.stdout.write(f"  VALUE:      {budget}: {reason}\n")
    for site, reason in lock_bad:
        sys.stdout.write(f"  LOCK SEQ:   {site}: {reason}\n")
    for pin, reason in pin_bad:
        sys.stdout.write(f"  PIN OWNER:  {pin}: {reason}\n")
    for crate, needed, name in dependency_misses:
        sys.stdout.write(f"  DEP MISS:   {crate} uses {needed} through {name}\n")
    for name, reason in snapshot_misses:
        sys.stdout.write(f"  SNAPSHOT:   {name}: {reason}\n")
    for crate, name, section in register_misses:
        sys.stdout.write(f"  REGISTER:   {crate} declares {name} in {section}, which its cell omits\n")
    for name, reason in test_misses:
        sys.stdout.write(f"  TEST MISS:  {name}: {reason}\n")

    if not candidates:
        sys.stdout.write("FAIL: the candidate set is empty, so the parse is broken.\n")
        return 1
    if (
        placeholders
        or heap_misses
        or grow_misses
        or lock_misses
        or root_bad
        or closure_bad
        or asserted_bad
        or pair_bad
        or tail_bad
        or b1_bad
        or link_bad
        or reason_bad
        or undeclared
        or external_misses
        or unplaced
        or duplicated
        or misclaimed
        or misused
        or copy_missing
        or copy_impossible
        or copy_undecided
        or unjustified
        or closure_broken
        or closure_undecided
        or reach_misses
        or expectation_misses
        or vr1_misses
        or eq_missing
        or spread_bad
        or edge_misses
        or dependency_misses
        or snapshot_misses
        or register_misses
        or test_misses
        or const_reach
        or limit_bad
        or probe_bad
        or member_bad
        or used_bad
        or lock_bad
        or pin_bad
        or floor_bad
        or value_bad
        or ragged_bad
        or index_bad
        or chunk_crate_bad
        or carrier_bad
    ):
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
