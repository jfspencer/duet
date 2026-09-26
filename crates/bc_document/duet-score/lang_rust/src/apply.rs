//! The transactional command path of the score aggregate.
//!
//! `Score::apply` takes one `ScoreCommand`, validates it in full, and mutates
//! only after every check passes, so a refused command leaves no partial
//! change. `Score::copy` reads a selection and changes nothing, and a cut is
//! a copy and then `ScoreCommand::Remove` in one undo transaction. Section
//! 3.4 of `roadmap/duet-v1/architecture.md` states the design.

use core::num::{NonZeroU8, NonZeroU16};
use std::collections::{BTreeMap, BTreeSet};

use duet_time::{Bbt, Meter, MeterPoint, TempoMap, TempoMapEdit, Ticks};
use smallvec::SmallVec;

use crate::command::{Clipboard, PitchEdit, ScoreCommand, Selection};
use crate::error::ScoreError;
use crate::event::ScoreEvent;
use crate::ids::{
    ElementRef, LyricText, MarkId, MeasureId, NoteId, PartId, PartName, SpannerId, StaffId,
    VerseNumber, VoiceId,
};
use crate::model::{
    Accidental, Articulation, Clef, Duration, Dynamic, InverseCost, KeySignature, MarkKind,
    Measure, Note, Part, Pitch, Rest, Score, ScoreMark, Spanner, SpannerKind, Staff, Step,
    TieState, Voice, VoiceType, note_order_key, rest_order_key, spanner_order_key,
};

/// What one command produced.
///
/// Its three fields are public. Section 3.5 names `Applied` as one of the three
/// stated `VR7` exemptions, and no other type of this crate takes that shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Applied {
    /// The facts a view needs.
    pub events: Vec<ScoreEvent>,
    /// The commands that undo this one, in application order.
    ///
    /// **`Score::apply` is transactional for ONE command and not for a list.**
    /// A caller applies the whole list, in order, and a refusal part way leaves
    /// the commands before it applied: the score is then partly undone, and
    /// this crate holds no roll-back for that state. The list-level
    /// transaction belongs to the undo stack of section 4.9 of
    /// `roadmap/duet-v1/architecture.md`, which is `duet-session`.
    ///
    /// The list is sound against the score that the forward command LEFT, and
    /// against no other score: the inverse of a `Remove` is a `Paste` whose
    /// tick ranges must still be free and whose spanner endpoints must still
    /// stand, and the inverse of a `SetVoice` names a voice that must still
    /// stand. A caller that replays the list against a score that other
    /// commands have touched can meet a refusal.
    pub inverse: Vec<ScoreCommand>,
    /// How many commands and how many bytes the inverse holds.
    pub inverse_cost: InverseCost,
}

/// The events and the inverse of one accepted command.
struct Outcome {
    /// The facts the command produced.
    events: Vec<ScoreEvent>,
    /// The commands that undo it, in application order.
    inverse: Vec<ScoreCommand>,
    /// The measured cost of the inverse.
    cost: InverseCost,
}

impl Outcome {
    /// The outcome of `events` and `inverse`, with the cost of the inverse.
    ///
    /// Every arm calls this after its last validation and before its first
    /// change to the content, so a refused command leaves the content, the
    /// revision, and the identifier counter as they were.
    ///
    /// The cost counts the commands of the list and the bytes of its JSON
    /// form, which is the measure that the undo stack of section 4.9 holds its
    /// memory bound against. Both counts hold at `u32::MAX`.
    ///
    /// **The measurement cannot refuse.** A list with no JSON form measures
    /// `u32::MAX` bytes, which is the value the undo stack already reads as too
    /// large to keep. A refusal here would be the one fallible step that stands
    /// AFTER a mint in eight arms, and the transaction rule of section 3.4 says
    /// that no identifier is minted before every validation passes.
    fn new(events: Vec<ScoreEvent>, inverse: Vec<ScoreCommand>) -> Self {
        let bytes = serde_json::to_vec(&inverse).map_or(u32::MAX, |form| {
            u32::try_from(form.len()).unwrap_or(u32::MAX)
        });
        let cost = InverseCost::new(u32::try_from(inverse.len()).unwrap_or(u32::MAX), bytes);
        Self {
            events,
            inverse,
            cost,
        }
    }
}

/// The elements that one selection names, after every reference is checked.
#[derive(Debug, Default)]
struct Resolved {
    /// The notes of the selection, in canonical order.
    notes: Vec<NoteId>,
    /// The rests of the selection, in canonical order.
    rests: Vec<NoteId>,
    /// The spanners of the selection, and every spanner that names a selected
    /// note or rest.
    spanners: Vec<SpannerId>,
    /// The marks of the selection, in canonical order.
    marks: Vec<MarkId>,
}

impl Resolved {
    /// Every note and every rest of this selection, the notes first.
    fn elements(&self) -> Vec<NoteId> {
        let mut all = self.notes.clone();
        all.extend(self.rests.iter().copied());
        all
    }

    /// Every note and every rest of this selection, as a set.
    fn element_set(&self) -> BTreeSet<NoteId> {
        self.elements().into_iter().collect()
    }
}

/// Where one note or rest moves, and where it came from.
struct Placement {
    /// The note or the rest that moves.
    id: NoteId,
    /// The staff that takes it.
    staff: StaffId,
    /// The voice that takes it.
    voice: VoiceId,
    /// The onset that it takes.
    onset: Ticks,
    /// The staff that it came from.
    earlier_staff: StaffId,
    /// The voice that it came from.
    earlier_voice: VoiceId,
}

/// One tick range that a command hands to one staff and one voice.
struct Taken {
    /// The staff that takes the range.
    staff: StaffId,
    /// The voice that takes the range.
    voice: VoiceId,
    /// The first tick of the range.
    onset: Ticks,
    /// The first tick after the range.
    end: Ticks,
}

impl Taken {
    /// The range that `duration` covers from `onset` in `staff` and `voice`.
    fn new(staff: StaffId, voice: VoiceId, onset: Ticks, duration: Duration) -> Self {
        Self {
            staff,
            voice,
            onset,
            end: onset.saturating_add(duration.ticks()),
        }
    }

    /// Whether this range and `other` cover one tick of one staff and voice.
    const fn meets(&self, other: &Self) -> bool {
        if self.staff.get() != other.staff.get() || self.voice.get() != other.voice.get() {
            return false;
        }
        let other_opens_first = other.onset.get() < self.end.get();
        let this_opens_first = self.onset.get() < other.end.get();
        other_opens_first && this_opens_first
    }
}

/// Refuse a range that an earlier range of the same command already took.
///
/// `check_free` reads the score, and the score still holds every moved element
/// at its earlier place, so the skip set of a group command hides the group
/// from itself: a moved element must not collide with where it came from. This
/// is the other half of the D15 rule. Every range that the command hands out
/// is checked against the ranges it has already handed out, so two elements of
/// one command cannot cover one tick range of one staff and one voice.
///
/// # Errors
/// Returns `ScoreError::OverlappingNote` with the staff, the voice, and the
/// onset of the range under test.
fn check_taken(taken: &[Taken], next: &Taken) -> Result<(), ScoreError> {
    for held in taken {
        if held.meets(next) {
            return Err(ScoreError::OverlappingNote {
                staff: next.staff,
                voice: next.voice,
                onset: next.onset,
            });
        }
    }
    Ok(())
}

/// The elements that one paste mints, with its events and its inverse.
struct Pasted {
    /// The notes that the paste inserts.
    notes: Vec<Note>,
    /// The rests that the paste inserts.
    rests: Vec<Rest>,
    /// The spanners that the paste inserts.
    spanners: Vec<Spanner>,
    /// The marks that the paste inserts.
    marks: Vec<ScoreMark>,
    /// The facts that the paste reports.
    events: Vec<ScoreEvent>,
    /// The commands that undo the paste.
    inverse: Vec<ScoreCommand>,
}

/// The tempo map with every meter entry after `after` moved by `by`.
///
/// An entry AT `after` stays where it stands, because the measure that opens
/// there keeps its start tick and only the measures behind it move. An entry
/// that lands on a tick another entry holds replaces it, which is the rule that
/// `meters_without` reads.
///
/// # Errors
/// Returns `ScoreError::Time` when the moved list breaks a rule of
/// `TempoMapEdit::finish`.
fn meters_moved_after(map: &TempoMap, after: Ticks, by: Ticks) -> Result<TempoMap, ScoreError> {
    let mut kept: BTreeMap<i64, MeterPoint> = BTreeMap::new();
    for point in map.meters() {
        let tick = if point.ticks() > after {
            point.ticks().saturating_add(by)
        } else {
            point.ticks()
        };
        kept.insert(
            tick.get(),
            MeterPoint::new(tick, point.bbt(), point.meter()),
        );
    }
    let mut edit = TempoMapEdit::new();
    for tempo in map.tempos() {
        edit = edit.push_tempo(*tempo);
    }
    for point in kept.into_values() {
        edit = edit.push_meter(point);
    }
    edit.recompute_cached_views()
        .finish()
        .map_err(ScoreError::from)
}

/// The tempo map that states the meter of the measure after a changed one.
///
/// `following` is the meter of that measure and `at` is the start tick it takes.
/// It carries a meter entry of its own when its meter differs from `meter`, the
/// meter that the changed measure now holds, and it carries none when the two
/// agree, because the entry of the changed measure governs that tick too.
/// `Score::meters_at` reads the same rule at the changed measure, and the pair
/// of the two keeps one statement of the meter at every tick. The rule is its
/// own inverse, so a do-and-undo pair of `SetTimeSignature` is an identity on
/// the map.
///
/// A change to the last measure of the score has no following measure and
/// answers the map it took.
///
/// # Errors
/// Returns `ScoreError::Time` when the new list breaks a rule of
/// `TempoMapEdit::finish`.
fn meters_at_boundary(
    map: &TempoMap,
    at: Ticks,
    following: Option<Meter>,
    meter: Meter,
) -> Result<TempoMap, ScoreError> {
    let Some(next) = following else {
        return Ok(map.clone());
    };
    let edit = map.edit();
    let placed = if next == meter {
        edit.remove_meter_at(at)
    } else {
        edit.insert_meter(MeterPoint::new(at, Bbt::ORIGIN, next))
    };
    placed
        .recompute_cached_views()
        .finish()
        .map_err(ScoreError::from)
}

/// The tick span of one bar of `meter`.
fn bar_ticks(meter: Meter) -> Ticks {
    let beat = meter.beat_unit().ticks().get();
    Ticks::new(beat.saturating_mul(i64::from(meter.beats_per_bar().get())))
}

/// `count` as a non-zero 16-bit number, held inside the two bounds.
///
/// A count of zero and a count above the 16-bit bound both answer the bound
/// they cross: one for zero, `u16::MAX` for a count above the range.
fn non_zero_u16(count: usize) -> NonZeroU16 {
    let raw = u16::try_from(count).unwrap_or(u16::MAX);
    NonZeroU16::new(raw).unwrap_or(NonZeroU16::MIN)
}

/// The ordinal above `greatest`, or the first ordinal where there is none.
///
/// It is one above the greatest ordinal that a voice type already carries, and
/// NOT a count of the parts of that voice type. A count repeats an ordinal
/// after a removal from the middle of the run: with "Soprano 1" gone and
/// "Soprano 2" standing, a count answers two and two parts then print one
/// label. Decision D21 states the count, and this is the amendment that the
/// second critic round asked for.
fn next_ordinal(greatest: Option<NonZeroU16>) -> NonZeroU16 {
    greatest.map_or(NonZeroU16::MIN, |held| {
        NonZeroU16::new(held.get().saturating_add(1)).unwrap_or(NonZeroU16::MAX)
    })
}

/// The notes and the rests that one `(staff, voice)` pair of a clipboard holds.
#[derive(Default)]
struct VoiceContent {
    /// The notes of the pair.
    notes: Vec<Note>,
    /// The rests of the pair.
    rests: Vec<Rest>,
}

/// The staff and the voice that a paste of no note and no rest names.
///
/// **Neither identifier is ever read.** `Score::paste` answers the clipboard
/// with no note and no rest through `paste_clipboard` with `None`, before it
/// checks the staff, and `paste_clipboard` reads the pair only to place a note
/// or a rest. A spanner names two notes and a mark names a tick, so a clipboard
/// of those alone needs no staff and no voice. Zero is a live number in this
/// keyspace, because `Score::new` mints it for the first measure, so a reader of
/// the inverse list must not take the pair for a reference.
const NO_PLACE: (StaffId, VoiceId) = (StaffId::new(0), VoiceId::new(0));

/// The pastes that put `clipboard` back at `at`.
///
/// One `Paste` names one staff and one voice, and it sends every note and every
/// rest of its clipboard there. The answer therefore holds one `Paste` for each
/// distinct `(staff, voice)` pair of the clipboard, and each one carries the
/// notes and the rests of that pair alone. A removal that spanned two staves
/// comes back into two staves, each element into the staff it came from.
///
/// A spanner names two notes and a mark names a tick, so neither holds a staff
/// of its own. Both ride in one further `Paste`, which names `NO_PLACE`: a mark
/// and a spanner ignore the staff and the voice of the `Paste` that carries
/// them. That `Paste` stands last, after every note it can name is back.
///
/// Every `Paste` reads the origin of `clipboard`, so `at` places the whole
/// answer as one: an `at` of that origin restores every onset unchanged.
fn restore_pastes(clipboard: &Clipboard, at: Ticks) -> Vec<ScoreCommand> {
    let mut grouped: BTreeMap<(StaffId, VoiceId), VoiceContent> = BTreeMap::new();
    for note in clipboard.notes() {
        grouped
            .entry((note.staff(), note.voice()))
            .or_default()
            .notes
            .push(note.clone());
    }
    for rest in clipboard.rests() {
        grouped
            .entry((rest.staff(), rest.voice()))
            .or_default()
            .rests
            .push(rest.clone());
    }
    let mut pastes: Vec<ScoreCommand> = grouped
        .into_iter()
        .map(|((staff, voice), content)| ScoreCommand::Paste {
            clipboard: Box::new(Clipboard::new(
                clipboard.origin(),
                content.notes,
                content.rests,
                Vec::new(),
                Vec::new(),
            )),
            at,
            staff,
            voice,
        })
        .collect();
    if !clipboard.spanners().is_empty() || !clipboard.marks().is_empty() {
        let (staff, voice) = NO_PLACE;
        pastes.push(ScoreCommand::Paste {
            clipboard: Box::new(Clipboard::new(
                clipboard.origin(),
                Vec::new(),
                Vec::new(),
                clipboard.spanners().to_vec(),
                clipboard.marks().to_vec(),
            )),
            at,
            staff,
            voice,
        });
    }
    pastes
}

/// The semitones of one step above the C of its own octave.
const fn step_semitones(step: Step) -> i32 {
    match step {
        Step::C => 0,
        Step::D => 2,
        Step::E => 4,
        Step::F => 5,
        Step::G => 7,
        Step::A => 9,
        Step::B => 11,
    }
}

/// The step that sits `semitones` above C, or `None` where no step does.
const fn step_at(semitones: i32) -> Option<Step> {
    match semitones {
        0 => Some(Step::C),
        2 => Some(Step::D),
        4 => Some(Step::E),
        5 => Some(Step::F),
        7 => Some(Step::G),
        9 => Some(Step::A),
        11 => Some(Step::B),
        _ => None,
    }
}

/// The alteration that `accidental` writes.
const fn accidental_alter(accidental: Accidental) -> i8 {
    match accidental {
        Accidental::Natural => 0,
        Accidental::Sharp => 1,
        Accidental::Flat => -1,
        Accidental::DoubleSharp => 2,
        Accidental::DoubleFlat => -2,
    }
}

/// `pitch` written with `accidental`, at the same sounding pitch.
///
/// A sounding pitch that has no spelling with that accidental is left
/// unchanged, so a respell never refuses.
fn respelled(pitch: Pitch, accidental: Accidental) -> Pitch {
    let alter = accidental_alter(accidental);
    let sounding = i32::from(pitch.octave())
        .saturating_mul(12)
        .saturating_add(step_semitones(pitch.step()))
        .saturating_add(i32::from(pitch.alter()));
    let natural = sounding.saturating_sub(i32::from(alter));
    let Some(step) = step_at(natural.rem_euclid(12)) else {
        return pitch;
    };
    let Ok(octave) = i8::try_from(natural.div_euclid(12)) else {
        return pitch;
    };
    Pitch::new(octave, step, alter)
}

/// The pitch that `edit` makes of `pitch`.
///
/// `Absolute` takes the written pitch whole. `BySemitones` moves the
/// alteration and `ByOctaves` moves the octave: both keep the step, so both
/// are exact and reversible and neither respells the note. A rule that chooses
/// a new step for a moved pitch is not yet stated, and `escalation:T2-11`
/// records the gap. `Respell` keeps the sounding pitch and writes it with the
/// named accidental.
///
/// `BySemitones` has a second effect that the same gap covers: the alteration
/// grows without a bound, so a move of twelve semitones writes an alteration of
/// twelve, which names no accidental an engraver can draw. `Pitch::alter`
/// states the range of a double flat to a double sharp and `Pitch::new` takes
/// any `i8`, so the bound belongs on the type, and what a pitch outside that
/// range means is a decision this chunk cannot make.
fn edited_pitch(pitch: Pitch, edit: PitchEdit) -> Pitch {
    match edit {
        PitchEdit::Absolute(taken) => taken,
        PitchEdit::BySemitones(step) => Pitch::new(
            pitch.octave(),
            pitch.step(),
            pitch.alter().saturating_add(step),
        ),
        PitchEdit::ByOctaves(step) => Pitch::new(
            pitch.octave().saturating_add(step),
            pitch.step(),
            pitch.alter(),
        ),
        PitchEdit::Respell(accidental) => respelled(pitch, accidental),
    }
}

/// The commands that move a selection back to where it came from.
fn move_inverse(
    selection: &Selection,
    by: Ticks,
    to_staff: Option<StaffId>,
    places: &[Placement],
) -> Vec<ScoreCommand> {
    let home = places.first().map(|place| place.earlier_staff);
    let mut inverse = vec![ScoreCommand::Move {
        selection: selection.clone(),
        by: Ticks::new(by.get().saturating_neg()),
        to_staff: to_staff.and(home),
    }];
    if to_staff.is_none() {
        return inverse;
    }
    let mut by_voice: BTreeMap<u64, Vec<NoteId>> = BTreeMap::new();
    for place in places {
        by_voice
            .entry(place.earlier_voice.get())
            .or_default()
            .push(place.id);
    }
    for (voice, notes) in by_voice {
        inverse.push(ScoreCommand::SetVoice {
            notes,
            voice: VoiceId::new(voice),
        });
    }
    inverse
}

/// The commands that put a removed run of measures back.
///
/// `InsertMeasures` names the measure that the new ones follow, so the inverse
/// can only name a measure that stays. A run that starts at the first measure
/// therefore comes back AFTER the measure that follows it, and a run that
/// covers every measure comes back not at all, because no command inserts a
/// measure into a score that holds none. The rebuilt measures also take new
/// identifiers. `escalation:T2-9` records the gap.
fn remove_measures_inverse(
    ordered: &[MeasureId],
    start: usize,
    len: usize,
    count: NonZeroU16,
) -> Vec<ScoreCommand> {
    let before = start.checked_sub(1).and_then(|index| ordered.get(index));
    let after = ordered.get(start.saturating_add(len));
    before.or(after).map_or_else(Vec::new, |anchor| {
        vec![ScoreCommand::InsertMeasures {
            after: *anchor,
            count,
        }]
    })
}

/// The commands that take one paste back out of the score.
///
/// A `Remove` of the pasted notes and rests also takes out every spanner that
/// names one of them, so the spanner list of the inverse names only the
/// spanners that name none.
fn paste_inverse(
    notes: &[Note],
    rests: &[Rest],
    spanners: &[Spanner],
    marks: &[ScoreMark],
) -> Vec<ScoreCommand> {
    let elements: Vec<NoteId> = notes
        .iter()
        .map(Note::id)
        .chain(rests.iter().map(Rest::id))
        .collect();
    let held: BTreeSet<NoteId> = elements.iter().copied().collect();
    let orphans: Vec<SpannerId> = spanners
        .iter()
        .filter(|spanner| !held.contains(&spanner.from()) && !held.contains(&spanner.to()))
        .map(Spanner::id)
        .collect();
    let mut inverse = Vec::new();
    if !orphans.is_empty() {
        inverse.push(ScoreCommand::Remove {
            selection: Selection::Spanners(orphans),
        });
    }
    if !elements.is_empty() {
        inverse.push(ScoreCommand::Remove {
            selection: Selection::Notes(elements),
        });
    }
    if !marks.is_empty() {
        inverse.push(ScoreCommand::Remove {
            selection: Selection::Marks(marks.iter().map(ScoreMark::id).collect()),
        });
    }
    inverse
}

impl Score {
    /// Apply one command. The score is unchanged when the result is an error.
    ///
    /// # Errors
    /// Returns `ScoreError` when the command names a missing element or breaks
    /// a notation invariant.
    pub fn apply(&mut self, command: ScoreCommand) -> Result<Applied, ScoreError> {
        let outcome = self.dispatch(command)?;
        self.bump_revision();
        Ok(Applied {
            events: outcome.events,
            inverse: outcome.inverse,
            inverse_cost: outcome.cost,
        })
    }

    /// Copy a selection without a change. A cut is this and then `Remove`.
    ///
    /// The clipboard carries every element of the selection under the
    /// identifier that it holds now, so a paste of that clipboard restores
    /// what a remove took. An empty selection answers an empty clipboard.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingElement` when the selection names an
    /// element the score does not hold, and `ScoreError::MissingStaff` when a
    /// range names a staff the score does not hold.
    pub fn copy(&self, selection: &Selection) -> Result<Clipboard, ScoreError> {
        let resolved = self.resolve(selection)?;
        let origin = self.origin_of(selection, &resolved);
        Ok(self.clipboard_of(&resolved, origin))
    }

    /// Run the arm that `command` names.
    ///
    /// Every arm validates in full before it mints an identifier and before it
    /// changes the content, so a refusal leaves the score equal to its own
    /// clone and leaves the identifier counter where it was.
    ///
    /// # Errors
    /// Returns the refusal of the arm.
    fn dispatch(&mut self, command: ScoreCommand) -> Result<Outcome, ScoreError> {
        match command {
            ScoreCommand::AddPart { voice_type, name } => Ok(self.add_part(voice_type, name)),
            ScoreCommand::RemovePart { part } => self.remove_part(part),
            ScoreCommand::AddStaff { part, clef } => self.add_staff(part, clef),
            ScoreCommand::RemoveStaff { staff } => self.remove_staff(staff),
            ScoreCommand::SetClef { staff, at, clef } => self.set_clef(staff, at, clef),
            ScoreCommand::InsertMeasures { after, count } => self.insert_measures(after, count),
            ScoreCommand::RemoveMeasures { from, count } => self.remove_measures(from, count),
            ScoreCommand::SetKeySignature { measure, key } => self.set_key_signature(measure, key),
            ScoreCommand::SetTimeSignature { measure, meter } => {
                self.set_time_signature(measure, meter)
            },
            ScoreCommand::AddMark { at, kind } => Ok(self.add_mark(at, kind)),
            ScoreCommand::RemoveMark { mark } => self.remove_mark(mark),
            ScoreCommand::InsertNote {
                staff,
                voice,
                onset,
                pitch,
                duration,
            } => self.insert_note(staff, voice, onset, pitch, duration),
            ScoreCommand::InsertRest {
                staff,
                voice,
                onset,
                duration,
            } => self.insert_rest(staff, voice, onset, duration),
            ScoreCommand::SetPitch { notes, pitch } => self.set_pitch(&notes, pitch),
            ScoreCommand::SetDuration { notes, duration } => self.set_duration(&notes, duration),
            ScoreCommand::SetTie { note, tie } => self.set_tie(note, tie),
            ScoreCommand::SetArticulations {
                notes,
                articulations,
            } => self.set_articulations(&notes, &articulations),
            ScoreCommand::SetLyric { note, verse, text } => self.set_lyric(note, verse, text),
            ScoreCommand::SetDynamic {
                staff,
                onset,
                dynamic,
            } => self.set_dynamic(staff, onset, dynamic),
            ScoreCommand::AddSpanner { kind, from, to } => self.add_spanner(kind, from, to),
            ScoreCommand::RemoveSpanner { spanner } => self.remove_spanner(spanner),
            ScoreCommand::SetVoice { notes, voice } => self.set_voice(&notes, voice),
            ScoreCommand::SetPart { notes, part, staff } => self.set_part(&notes, part, staff),
            ScoreCommand::Move {
                selection,
                by,
                to_staff,
            } => self.move_selection(&selection, by, to_staff),
            ScoreCommand::Duplicate { selection, at } => self.duplicate(&selection, at),
            ScoreCommand::Remove { selection } => self.remove(&selection),
            ScoreCommand::Paste {
                clipboard,
                at,
                staff,
                voice,
            } => self.paste(&clipboard, at, staff, voice),
        }
    }
}

impl Score {
    /// The staff that `staff` names.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingStaff` when the score holds no such staff.
    fn staff_of(&self, staff: StaffId) -> Result<&Staff, ScoreError> {
        self.staves()
            .get(&staff)
            .ok_or(ScoreError::MissingStaff(staff))
    }

    /// Check that `staff` holds `voice`.
    ///
    /// `ScoreError` declares no missing-voice variant, so a voice that the
    /// named staff does not hold answers `MissingStaff` of that staff. The
    /// refusal roster cannot say more, and `escalation:T2-7` records the gap.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingStaff` for an absent staff and for a voice
    /// that the staff does not hold.
    fn voice_in(&self, staff: StaffId, voice: VoiceId) -> Result<(), ScoreError> {
        if self.staff_of(staff)?.voices().contains(&voice) {
            return Ok(());
        }
        Err(ScoreError::MissingStaff(staff))
    }

    /// The first voice of `staff`.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingStaff` for an absent staff and for a staff
    /// that holds no voice.
    fn first_voice(&self, staff: StaffId) -> Result<VoiceId, ScoreError> {
        self.staff_of(staff)?
            .voices()
            .first()
            .copied()
            .ok_or(ScoreError::MissingStaff(staff))
    }

    /// The note that `note` names.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingElement` when the score holds no such note.
    fn note_of(&self, note: NoteId) -> Result<&Note, ScoreError> {
        self.notes()
            .get(&note)
            .ok_or(ScoreError::MissingElement(ElementRef::Note(note)))
    }

    /// The measure that `measure` names.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingMeasure` when the score holds no such
    /// measure.
    fn measure_of(&self, measure: MeasureId) -> Result<&Measure, ScoreError> {
        self.measures()
            .get(&measure)
            .ok_or(ScoreError::MissingMeasure(measure))
    }

    /// Check that the score holds every note of `notes`.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingElement` for the first absent note.
    fn check_notes(&self, notes: &[NoteId]) -> Result<(), ScoreError> {
        for note in notes {
            self.note_of(*note)?;
        }
        Ok(())
    }

    /// The part that holds `staff`.
    fn part_of_staff(&self, staff: StaffId) -> Option<PartId> {
        self.parts()
            .values()
            .find(|part| part.staves().contains(&staff))
            .map(Part::id)
    }

    /// The staff, the voice, the onset, and the duration of one note or rest.
    fn place_of(&self, id: NoteId) -> Option<(StaffId, VoiceId, Ticks, Duration)> {
        if let Some(note) = self.notes().get(&id) {
            return Some((note.staff(), note.voice(), note.onset(), note.duration()));
        }
        let rest = self.rests().get(&id)?;
        Some((rest.staff(), rest.voice(), rest.onset(), rest.duration()))
    }

    /// Every note span and rest span of `staff` and `voice`.
    fn held_spans(&self, staff: StaffId, voice: VoiceId) -> Vec<(NoteId, Ticks, Ticks)> {
        let notes = self
            .notes()
            .values()
            .filter(|note| note.staff() == staff && note.voice() == voice)
            .map(|note| {
                (
                    note.id(),
                    note.onset(),
                    note.onset().saturating_add(note.duration().ticks()),
                )
            });
        let rests = self
            .rests()
            .values()
            .filter(|rest| rest.staff() == staff && rest.voice() == voice)
            .map(|rest| {
                (
                    rest.id(),
                    rest.onset(),
                    rest.onset().saturating_add(rest.duration().ticks()),
                )
            });
        notes.chain(rests).collect()
    }

    /// Refuse a span of `staff` and `voice` that a note or a rest already fills.
    ///
    /// Two elements of one staff and one voice overlap when their half open
    /// tick intervals intersect. `skip` names the elements that the command
    /// itself moves or resizes, which never collide with their own earlier
    /// place.
    ///
    /// # Errors
    /// Returns `ScoreError::OverlappingNote` with the staff, the voice, and
    /// the onset that the command asked for.
    fn check_free(
        &self,
        staff: StaffId,
        voice: VoiceId,
        onset: Ticks,
        duration: Duration,
        skip: &BTreeSet<NoteId>,
    ) -> Result<(), ScoreError> {
        let end = onset.saturating_add(duration.ticks());
        for (id, held_onset, held_end) in self.held_spans(staff, voice) {
            if !skip.contains(&id) && held_onset < end && onset < held_end {
                return Err(ScoreError::OverlappingNote {
                    staff,
                    voice,
                    onset,
                });
            }
        }
        Ok(())
    }

    /// The next note of the staff and the voice of `note`, by onset and then
    /// by identifier.
    fn partner_of(&self, note: &Note) -> Option<NoteId> {
        let anchor = (note.onset(), note.id());
        self.notes()
            .values()
            .filter(|other| other.staff() == note.staff() && other.voice() == note.voice())
            .filter(|other| (other.onset(), other.id()) > anchor)
            .min_by_key(|other| (other.onset(), other.id()))
            .map(Note::id)
    }

    /// Every spanner that names an element of `elements`, in canonical order.
    fn spanners_touching(&self, elements: &BTreeSet<NoteId>) -> Vec<Spanner> {
        let mut held: Vec<Spanner> = self
            .spanners()
            .values()
            .filter(|spanner| {
                elements.contains(&spanner.from()) || elements.contains(&spanner.to())
            })
            .cloned()
            .collect();
        held.sort_by_key(spanner_order_key);
        held
    }
}

impl Score {
    /// The elements that `selection` names.
    ///
    /// It adds every spanner that names a selected note or rest, so a cut
    /// carries the spanners that the removal takes with it. A range is HALF
    /// OPEN: it holds every element whose onset is at or after `from` and
    /// below `to`, so two ranges that touch select each element once.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingElement` for an element the score does not
    /// hold, and `ScoreError::MissingStaff` for a range over an absent staff.
    fn resolve(&self, selection: &Selection) -> Result<Resolved, ScoreError> {
        let mut resolved = Resolved::default();
        match *selection {
            Selection::Notes(ref ids) => self.resolve_notes(ids, &mut resolved)?,
            Selection::Spanners(ref ids) => self.resolve_spanners(ids, &mut resolved)?,
            Selection::Marks(ref ids) => self.resolve_marks(ids, &mut resolved)?,
            Selection::Range { staff, from, to } => {
                self.resolve_range(staff, from, to, &mut resolved)?;
            },
        }
        self.add_held_spanners(&mut resolved);
        self.sort_resolved(&mut resolved);
        Ok(resolved)
    }

    /// Put `ids` in the spanners of `resolved`.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingElement` for the first absent spanner.
    fn resolve_spanners(
        &self,
        ids: &[SpannerId],
        resolved: &mut Resolved,
    ) -> Result<(), ScoreError> {
        for id in ids {
            if !self.spanners().contains_key(id) {
                return Err(ScoreError::MissingElement(ElementRef::Spanner(*id)));
            }
            resolved.spanners.push(*id);
        }
        Ok(())
    }

    /// Put `ids` in the marks of `resolved`.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingElement` for the first absent mark.
    fn resolve_marks(&self, ids: &[MarkId], resolved: &mut Resolved) -> Result<(), ScoreError> {
        for id in ids {
            if !self.marks().contains_key(id) {
                return Err(ScoreError::MissingElement(ElementRef::Mark(*id)));
            }
            resolved.marks.push(*id);
        }
        Ok(())
    }

    /// Put every note and rest of `staff` between `from` and `to` in `resolved`.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingStaff` for an absent staff.
    fn resolve_range(
        &self,
        staff: StaffId,
        from: Ticks,
        to: Ticks,
        resolved: &mut Resolved,
    ) -> Result<(), ScoreError> {
        self.staff_of(staff)?;
        resolved.notes = self
            .notes()
            .values()
            .filter(|note| note.staff() == staff && from <= note.onset() && note.onset() < to)
            .map(Note::id)
            .collect();
        resolved.rests = self
            .rests()
            .values()
            .filter(|rest| rest.staff() == staff && from <= rest.onset() && rest.onset() < to)
            .map(Rest::id)
            .collect();
        Ok(())
    }

    /// Sort `ids` into the notes and the rests of `resolved`.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingElement` for the first identifier that
    /// names neither a note nor a rest.
    fn resolve_notes(&self, ids: &[NoteId], resolved: &mut Resolved) -> Result<(), ScoreError> {
        for id in ids {
            if self.notes().contains_key(id) {
                resolved.notes.push(*id);
            } else if self.rests().contains_key(id) {
                resolved.rests.push(*id);
            } else {
                return Err(ScoreError::MissingElement(ElementRef::Note(*id)));
            }
        }
        Ok(())
    }

    /// Add every spanner that names a selected note or rest.
    fn add_held_spanners(&self, resolved: &mut Resolved) {
        let elements = resolved.element_set();
        if elements.is_empty() {
            return;
        }
        let mut held: BTreeSet<SpannerId> = resolved.spanners.iter().copied().collect();
        for spanner in self.spanners_touching(&elements) {
            held.insert(spanner.id());
        }
        resolved.spanners = held.into_iter().collect();
    }

    /// Order the four lists of `resolved` by the canonical keys of section 3.6.
    fn sort_resolved(&self, resolved: &mut Resolved) {
        resolved
            .notes
            .sort_by_key(|id| self.notes().get(id).map(note_order_key));
        resolved
            .rests
            .sort_by_key(|id| self.rests().get(id).map(rest_order_key));
        resolved
            .spanners
            .sort_by_key(|id| self.spanners().get(id).map(spanner_order_key));
        resolved
            .marks
            .sort_by_key(|id| self.marks().get(id).map(|mark| (mark.at(), mark.id())));
    }

    /// The tick that every onset of a copy is measured from.
    ///
    /// A range takes its own `from`, which keeps a leading gap. Every other
    /// selection takes the least onset that it holds, and an empty selection
    /// takes tick zero.
    fn origin_of(&self, selection: &Selection, resolved: &Resolved) -> Ticks {
        if let Selection::Range { from, .. } = *selection {
            return from;
        }
        let notes = resolved
            .notes
            .iter()
            .filter_map(|id| self.notes().get(id))
            .map(Note::onset);
        let rests = resolved
            .rests
            .iter()
            .filter_map(|id| self.rests().get(id))
            .map(Rest::onset);
        notes.chain(rests).min().unwrap_or(Ticks::ZERO)
    }

    /// The detached copy of everything that `resolved` names.
    fn clipboard_of(&self, resolved: &Resolved, origin: Ticks) -> Clipboard {
        let notes = resolved
            .notes
            .iter()
            .filter_map(|id| self.notes().get(id).cloned())
            .collect();
        let rests = resolved
            .rests
            .iter()
            .filter_map(|id| self.rests().get(id).cloned())
            .collect();
        let spanners = resolved
            .spanners
            .iter()
            .filter_map(|id| self.spanners().get(id).cloned())
            .collect();
        let marks = resolved
            .marks
            .iter()
            .filter_map(|id| self.marks().get(id).cloned())
            .collect();
        Clipboard::new(origin, notes, rests, spanners, marks)
    }

    /// The commands that put `notes` back as they stand now.
    ///
    /// It is the inverse of last resort: a `Remove` of the notes and then the
    /// pastes of the copy that this call takes first. A paste restores every
    /// identifier, so the list undoes any change to a note that no `Set`
    /// command can express. Use it only where none can.
    ///
    /// The notes can stand in more than one staff, so `restore_pastes` builds
    /// the paste of each `(staff, voice)` pair of the copy.
    fn restore_of(&self, notes: &[NoteId]) -> Vec<ScoreCommand> {
        let kept: Vec<Note> = notes
            .iter()
            .filter_map(|id| self.notes().get(id).cloned())
            .collect();
        let elements: BTreeSet<NoteId> = notes.iter().copied().collect();
        let spanners = self.spanners_touching(&elements);
        let origin = kept.iter().map(Note::onset).min().unwrap_or(Ticks::ZERO);
        let clipboard = Clipboard::new(origin, kept, Vec::new(), spanners, Vec::new());
        let mut inverse = vec![ScoreCommand::Remove {
            selection: Selection::Notes(notes.to_vec()),
        }];
        inverse.extend(restore_pastes(&clipboard, origin));
        inverse
    }
}

impl Score {
    /// Add one part, with the ordinal that its voice type gives it.
    ///
    /// The ordinal is one above the greatest ordinal that the same voice type
    /// already carries: "Soprano 1" and then "Soprano 2". A count of the parts
    /// would repeat an ordinal after a removal, and the ordinal is the number
    /// that a user reads in the part label.
    ///
    /// It refuses nothing: a part names no other entity, and the ordinal and
    /// the identifier both come from the score itself.
    fn add_part(&mut self, voice_type: VoiceType, name: PartName) -> Outcome {
        let ordinal = next_ordinal(
            self.parts()
                .values()
                .filter(|part| part.voice_type() == voice_type)
                .map(Part::ordinal)
                .max(),
        );
        let part = PartId::new(self.mint_id());
        let outcome = Outcome::new(
            vec![ScoreEvent::PartAdded(part)],
            vec![ScoreCommand::RemovePart { part }],
        );
        self.parts_mut()
            .insert(part, Part::new(part, voice_type, ordinal, name));
        outcome
    }

    /// Remove one part, its staves, its voices, and their content.
    ///
    /// The inverse rebuilds the part alone, under a NEW identifier, and
    /// restores no staff and no note. No command of the declared set names an
    /// identifier for a new part, a new staff, or a new voice, so a structure
    /// removal has no true undo. `escalation:T2-9` records the gap. The ORDINAL
    /// of the rebuilt part is one above the greatest that its voice type then
    /// carries, which is the ordinal it held only where it held the greatest.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingPart` for an absent part.
    fn remove_part(&mut self, part: PartId) -> Result<Outcome, ScoreError> {
        let held = self
            .parts()
            .get(&part)
            .ok_or(ScoreError::MissingPart(part))?;
        let voice_type = held.voice_type();
        let name = held.name().clone();
        let staves: Vec<StaffId> = held.staves().to_vec();
        let outcome = Outcome::new(
            vec![ScoreEvent::PartRemoved(part)],
            vec![ScoreCommand::AddPart { voice_type, name }],
        );
        for staff in staves {
            self.strip_staff(staff);
        }
        self.parts_mut().remove(&part);
        Ok(outcome)
    }

    /// Take `staff`, its voices, and their content out of the score.
    fn strip_staff(&mut self, staff: StaffId) {
        let voices: Vec<VoiceId> = self
            .staves()
            .get(&staff)
            .map(|held| held.voices().to_vec())
            .unwrap_or_default();
        let gone: BTreeSet<NoteId> = self
            .notes()
            .values()
            .filter(|note| note.staff() == staff)
            .map(Note::id)
            .chain(
                self.rests()
                    .values()
                    .filter(|rest| rest.staff() == staff)
                    .map(Rest::id),
            )
            .collect();
        self.spanners_mut()
            .retain(|_, spanner| !gone.contains(&spanner.from()) && !gone.contains(&spanner.to()));
        for id in &gone {
            self.notes_mut().remove(id);
            self.rests_mut().remove(id);
        }
        for voice in voices {
            self.voices_mut().remove(&voice);
        }
        self.staves_mut().remove(&staff);
    }

    /// Add one staff to one part, with its first voice.
    ///
    /// No command of the declared set mints a voice, and `InsertNote` names
    /// one, so the staff opens with one voice of ordinal one and stems up.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingPart` for an absent part.
    fn add_staff(&mut self, part: PartId, clef: Clef) -> Result<Outcome, ScoreError> {
        if !self.parts().contains_key(&part) {
            return Err(ScoreError::MissingPart(part));
        }
        let staff = StaffId::new(self.mint_id());
        let voice = VoiceId::new(self.mint_id());
        let outcome = Outcome::new(
            vec![ScoreEvent::StaffAdded(staff)],
            vec![ScoreCommand::RemoveStaff { staff }],
        );
        let mut record = Staff::new(staff, clef, 0);
        record.push_voice(voice);
        self.staves_mut().insert(staff, record);
        self.voices_mut()
            .insert(voice, Voice::new(voice, NonZeroU8::MIN, true));
        if let Some(owner) = self.parts_mut().get_mut(&part) {
            owner.push_staff(staff);
        }
        Ok(outcome)
    }

    /// Remove one staff, its voices, and their content.
    ///
    /// The inverse rebuilds the staff under a NEW identifier and restores no
    /// note, for the reason `remove_part` states.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingStaff` for an absent staff.
    fn remove_staff(&mut self, staff: StaffId) -> Result<Outcome, ScoreError> {
        let clef = self.staff_of(staff)?.clef();
        let owner = self.part_of_staff(staff);
        let inverse =
            owner.map_or_else(Vec::new, |part| vec![ScoreCommand::AddStaff { part, clef }]);
        let outcome = Outcome::new(vec![ScoreEvent::StaffRemoved(staff)], inverse);
        self.strip_staff(staff);
        if let Some(record) = owner.and_then(|part| self.parts_mut().get_mut(&part)) {
            record.drop_staff(staff);
        }
        Ok(outcome)
    }

    /// Replace the clef that one staff opens with.
    ///
    /// `Staff` holds one clef and no list of clef changes, so the `at` tick has
    /// nowhere to live and this command cannot write a clef change in the
    /// middle of a staff. `escalation:T2-10` records the gap.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingStaff` for an absent staff.
    fn set_clef(&mut self, staff: StaffId, at: Ticks, clef: Clef) -> Result<Outcome, ScoreError> {
        let earlier = self.staff_of(staff)?.clef();
        let outcome = Outcome::new(
            vec![ScoreEvent::ClefChanged(staff)],
            vec![ScoreCommand::SetClef {
                staff,
                at,
                clef: earlier,
            }],
        );
        if let Some(record) = self.staves_mut().get_mut(&staff) {
            record.set_clef(clef);
        }
        Ok(outcome)
    }
}

impl Score {
    /// The measures of the score, ordered by start tick.
    fn measures_in_order(&self) -> Vec<MeasureId> {
        let mut ordered: Vec<(Ticks, MeasureId)> = self
            .measures()
            .values()
            .map(|measure| (measure.start(), measure.id()))
            .collect();
        ordered.sort_unstable();
        ordered.into_iter().map(|(_, id)| id).collect()
    }

    /// The first tick of `run` and the tick span that it covers.
    fn run_span(&self, run: &[MeasureId]) -> (Ticks, Ticks) {
        let start = run
            .first()
            .and_then(|id| self.measures().get(id))
            .map_or(Ticks::ZERO, Measure::start);
        let span = run
            .iter()
            .filter_map(|id| self.measures().get(id))
            .fold(Ticks::ZERO, |total, measure| {
                total.saturating_add(bar_ticks(measure.meter()))
            });
        (start, span)
    }

    /// Refuse a run of measures that holds a note or a rest.
    ///
    /// **A measure holds content when the half-open span of a note or a rest
    /// intersects the half-open span of the measure.** That is the D15 rule
    /// that `check_free` and `Taken::meets` read. An onset test answers another
    /// question, the measure that a note BEGINS in, and it reads a measure that
    /// a longer note sounds over as empty: `remove_measures` would then accept
    /// the run, move the later content earlier, and leave two elements of one
    /// staff and one voice over one tick range.
    ///
    /// A duration of zero ticks covers an empty span, which intersects nothing,
    /// so such an element fills no measure. `Duration::ticks` states the one
    /// input that answers zero ticks, and `escalation:T2-16` holds the bound.
    ///
    /// # Errors
    /// Returns `ScoreError::MeasureNotEmpty` for the first filled measure.
    fn check_measures_empty(&self, run: &[MeasureId]) -> Result<(), ScoreError> {
        let notes = self.notes().values().map(|note| {
            (
                note.onset(),
                note.onset().saturating_add(note.duration().ticks()),
            )
        });
        let rests = self.rests().values().map(|rest| {
            (
                rest.onset(),
                rest.onset().saturating_add(rest.duration().ticks()),
            )
        });
        let spans: Vec<(Ticks, Ticks)> = notes.chain(rests).collect();
        for id in run {
            let Some(measure) = self.measures().get(id) else {
                continue;
            };
            let start = measure.start();
            let end = start.saturating_add(bar_ticks(measure.meter()));
            let filled = spans
                .iter()
                .any(|(onset, sounds_to)| *onset < end && start < *sounds_to);
            if filled {
                return Err(ScoreError::MeasureNotEmpty(*id));
            }
        }
        Ok(())
    }

    /// The tempo map with every meter entry at or after `from` moved by `by`.
    ///
    /// # Errors
    /// Returns `ScoreError::Time` when the moved list breaks a rule of
    /// `TempoMapEdit::finish`.
    fn meters_shifted(&self, from: Ticks, by: Ticks) -> Result<TempoMap, ScoreError> {
        let mut edit = TempoMapEdit::new();
        for tempo in self.tempo_map().tempos() {
            edit = edit.push_tempo(*tempo);
        }
        for point in self.tempo_map().meters() {
            let tick = if point.ticks() >= from {
                point.ticks().saturating_add(by)
            } else {
                point.ticks()
            };
            edit = edit.push_meter(MeterPoint::new(tick, point.bbt(), point.meter()));
        }
        edit.recompute_cached_views()
            .finish()
            .map_err(ScoreError::from)
    }

    /// The tempo map with the meter entries between `from` and `to` dropped
    /// and every entry at or after `to` moved earlier by the span of the two.
    ///
    /// An entry that sits exactly at `from` stays, because the meter that
    /// governs the removed run also governs what follows it. An entry that
    /// lands on a tick another entry holds replaces it, because the later
    /// entry is the one that governs from that tick on.
    ///
    /// # Errors
    /// Returns `ScoreError::Time` when the new list breaks a rule of
    /// `TempoMapEdit::finish`.
    fn meters_without(&self, from: Ticks, to: Ticks) -> Result<TempoMap, ScoreError> {
        let by = to.saturating_sub(from);
        let mut kept: BTreeMap<i64, MeterPoint> = BTreeMap::new();
        for point in self.tempo_map().meters() {
            if point.ticks() > from && point.ticks() < to {
                continue;
            }
            let tick = if point.ticks() >= to {
                point.ticks().saturating_sub(by)
            } else {
                point.ticks()
            };
            kept.insert(
                tick.get(),
                MeterPoint::new(tick, point.bbt(), point.meter()),
            );
        }
        let mut edit = TempoMapEdit::new();
        for tempo in self.tempo_map().tempos() {
            edit = edit.push_tempo(*tempo);
        }
        for point in kept.into_values() {
            edit = edit.push_meter(point);
        }
        edit.recompute_cached_views()
            .finish()
            .map_err(ScoreError::from)
    }

    /// The meter of the measure that stands before `measure` on the timeline.
    ///
    /// The answer is `None` for the first measure of the score, and for a
    /// measure the score does not hold.
    fn meter_before(&self, measure: MeasureId) -> Option<Meter> {
        let ordered = self.measures_in_order();
        let place = ordered.iter().position(|id| *id == measure)?;
        let earlier = place.checked_sub(1)?;
        ordered
            .get(earlier)
            .and_then(|id| self.measures().get(id))
            .map(Measure::meter)
    }

    /// The meter of the measure that stands after `measure` on the timeline.
    ///
    /// The answer is `None` for the last measure of the score, and for a measure
    /// the score does not hold.
    fn meter_after(&self, measure: MeasureId) -> Option<Meter> {
        let ordered = self.measures_in_order();
        let place = ordered.iter().position(|id| *id == measure)?;
        ordered
            .get(place.saturating_add(1))
            .and_then(|id| self.measures().get(id))
            .map(Measure::meter)
    }

    /// The tempo map that states `meter` from `start` on.
    ///
    /// A measure carries a meter entry of its own when its meter differs from
    /// the meter of the measure before it, and it carries none when the two
    /// agree, because the entry that governs the earlier measure governs that
    /// tick too. `governing` is the meter of the measure before, and `None`
    /// names the first measure of the score, which always carries an entry:
    /// the meter list opens at tick zero.
    ///
    /// The rule is what makes the do-and-undo pair of a `SetTimeSignature` an
    /// identity on the map. An insert alone cannot be one, because no command
    /// of the declared set removes a meter entry, so the undo of a change to a
    /// measure that carried no entry would leave one behind.
    ///
    /// A document that already carries an entry the rule does not ask for
    /// loses it here. The meter at every tick is unchanged by that, because the
    /// entry it drops states the meter that the entry before it already states.
    ///
    /// # Errors
    /// Returns `ScoreError::Time` when the new list breaks a rule of
    /// `TempoMapEdit::finish`.
    fn meters_at(
        &self,
        start: Ticks,
        meter: Meter,
        governing: Option<Meter>,
    ) -> Result<TempoMap, ScoreError> {
        let edit = self.tempo_map().edit();
        let placed = if governing == Some(meter) {
            edit.remove_meter_at(start)
        } else {
            edit.insert_meter(MeterPoint::new(start, Bbt::ORIGIN, meter))
        };
        placed
            .recompute_cached_views()
            .finish()
            .map_err(ScoreError::from)
    }

    /// Move every measure, mark, note, and rest at or after `from` by `by`,
    /// and take `meters` as the new tempo map.
    fn shift_from(&mut self, from: Ticks, by: Ticks, meters: TempoMap) {
        for measure in self.measures_mut().values_mut() {
            let start = measure.start();
            if start >= from {
                measure.set_start(start.saturating_add(by));
            }
        }
        for mark in self.marks_mut().values_mut() {
            let at = mark.at();
            if at >= from {
                mark.set_at(at.saturating_add(by));
            }
        }
        for note in self.notes_mut().values_mut() {
            let onset = note.onset();
            if onset >= from {
                note.set_onset(onset.saturating_add(by));
            }
        }
        for rest in self.rests_mut().values_mut() {
            let onset = rest.onset();
            if onset >= from {
                rest.set_onset(onset.saturating_add(by));
            }
        }
        self.set_tempo_map(meters);
    }

    /// Insert empty measures after one measure, and move later content later.
    ///
    /// The new measures copy the meter and the key of the measure they follow.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingMeasure` for an absent measure, and
    /// `ScoreError::Time` when the moved meter list breaks a map rule.
    fn insert_measures(
        &mut self,
        after: MeasureId,
        count: NonZeroU16,
    ) -> Result<Outcome, ScoreError> {
        let anchor = self.measure_of(after)?;
        let meter = anchor.meter();
        let key = anchor.key();
        let bar = bar_ticks(meter);
        let start = anchor.start().saturating_add(bar);
        let span = Ticks::new(bar.get().saturating_mul(i64::from(count.get())));
        let meters = self.meters_shifted(start, span)?;
        let minted: Vec<MeasureId> = (0..count.get())
            .map(|_| MeasureId::new(self.mint_id()))
            .collect();
        let first = minted.first().copied().unwrap_or(after);
        let outcome = Outcome::new(
            vec![ScoreEvent::MeasuresChanged {
                from: first,
                count: count.get(),
            }],
            vec![ScoreCommand::RemoveMeasures { from: first, count }],
        );
        self.shift_from(start, span, meters);
        for (index, id) in minted.into_iter().enumerate() {
            let step = i64::try_from(index).unwrap_or(i64::MAX);
            let offset = Ticks::new(bar.get().saturating_mul(step));
            self.measures_mut().insert(
                id,
                Measure::new(id, start.saturating_add(offset), meter, key),
            );
        }
        Ok(outcome)
    }

    /// Remove a run of empty measures and move later content earlier.
    ///
    /// The run stops at the last measure of the score, and the event and the
    /// inverse both carry the number of measures REMOVED, which is below the
    /// number asked for where the run reaches the end.
    ///
    /// A score holds at least one measure, so a run that would cover every
    /// measure is refused. `ScoreError` declares no variant for that state, and
    /// the arm answers `MeasureNotEmpty` of the named measure: the message names
    /// a note and a rest, which this path has neither of, so the refusal reads
    /// wrong to a person. The roster needs a variant of its own, and
    /// `escalation:T2-9` records the gap. A refusal with a poor message is still
    /// the honest answer, because the alternative is an accepted command that
    /// destroys the measure grid and answers an inverse that undoes nothing.
    ///
    /// **The inverse rebuilds equivalent structure, and not the structure that
    /// stood.** `InsertMeasures` names the measure that the new ones follow, so
    /// the inverse can name only a measure that stays: a run that starts at the
    /// first measure comes back AFTER the measure that followed it, and the
    /// rebuilt measures take NEW identifiers and the meter and the key of their
    /// anchor. `escalation:T2-9` records that gap too.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingMeasure` for an absent measure,
    /// `ScoreError::MeasureNotEmpty` for a measure of the run that holds a note
    /// or a rest and for a run that covers every measure of the score, and
    /// `ScoreError::Time` when the new meter list breaks a map rule.
    fn remove_measures(
        &mut self,
        from: MeasureId,
        count: NonZeroU16,
    ) -> Result<Outcome, ScoreError> {
        self.measure_of(from)?;
        let ordered = self.measures_in_order();
        let start_index = ordered.iter().position(|id| *id == from).unwrap_or(0);
        let run: Vec<MeasureId> = ordered
            .iter()
            .skip(start_index)
            .take(usize::from(count.get()))
            .copied()
            .collect();
        if run.len() >= ordered.len() {
            return Err(ScoreError::MeasureNotEmpty(from));
        }
        self.check_measures_empty(&run)?;
        let removed = non_zero_u16(run.len());
        let (start, span) = self.run_span(&run);
        let end = start.saturating_add(span);
        let meters = self.meters_without(start, end)?;
        let next = ordered
            .get(start_index.saturating_add(run.len()))
            .copied()
            .unwrap_or(from);
        let inverse = remove_measures_inverse(&ordered, start_index, run.len(), removed);
        let outcome = Outcome::new(
            vec![ScoreEvent::MeasuresChanged {
                from: next,
                count: removed.get(),
            }],
            inverse,
        );
        for id in &run {
            self.measures_mut().remove(id);
        }
        self.shift_from(end, Ticks::new(span.get().saturating_neg()), meters);
        Ok(outcome)
    }

    /// Change the key signature of one measure.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingMeasure` for an absent measure.
    fn set_key_signature(
        &mut self,
        measure: MeasureId,
        key: KeySignature,
    ) -> Result<Outcome, ScoreError> {
        let earlier = self.measure_of(measure)?.key();
        let outcome = Outcome::new(
            vec![ScoreEvent::SignatureChanged(measure)],
            vec![ScoreCommand::SetKeySignature {
                measure,
                key: earlier,
            }],
        );
        if let Some(record) = self.measures_mut().get_mut(&measure) {
            record.set_key(key);
        }
        Ok(outcome)
    }

    /// Change the meter of one measure, in the measure and in the tempo map.
    ///
    /// **It moves no content.** A note onset is absolute, in ticks from score
    /// zero, and a measure is a shared entry on the timeline and not a
    /// container, which section 3.3 of `roadmap/duet-v1/architecture.md` states.
    /// The bar lines therefore move under the notes: the command moves no note,
    /// no rest, no mark, and no spanner, and nothing can collide because nothing
    /// moves. The consequence a reader must know is that a note can now sound
    /// across a bar line that it did not cross before, and a note that filled
    /// its own measure can reach into the measure that follows. This crate holds
    /// no rule that refuses that state, and no rule of section 3 asks for one.
    ///
    /// **The start tick of every LATER measure is re-laid.** A bar of three
    /// quarter notes and a bar of four cover different spans, so every measure
    /// behind the changed one moves by the difference of the two spans. The
    /// measure grid stays one contiguous run, and no tick range belongs to no
    /// measure.
    ///
    /// The two statements of the meter move together, so a query of the map and
    /// a read of the measure can never disagree. `meters_at` states the rule that
    /// decides whether the changed measure carries a meter entry of its own,
    /// `meters_moved_after` moves the entries of the later measures with those
    /// measures, and `meters_at_boundary` states the meter of the measure that
    /// follows. Each of the three rules is its own inverse, which is what makes
    /// the inverse restore the map the score held.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingMeasure` for an absent measure, and
    /// `ScoreError::Time` when a new meter list breaks a map rule.
    fn set_time_signature(
        &mut self,
        measure: MeasureId,
        meter: Meter,
    ) -> Result<Outcome, ScoreError> {
        let record = self.measure_of(measure)?;
        let earlier = record.meter();
        let start = record.start();
        let bar = bar_ticks(meter);
        let by = Ticks::new(bar.get().saturating_sub(bar_ticks(earlier).get()));
        let governing = self.meter_before(measure);
        let following = self.meter_after(measure);
        let changed = self.meters_at(start, meter, governing)?;
        let moved = meters_moved_after(&changed, start, by)?;
        let map = meters_at_boundary(&moved, start.saturating_add(bar), following, meter)?;
        let outcome = Outcome::new(
            vec![ScoreEvent::SignatureChanged(measure)],
            vec![ScoreCommand::SetTimeSignature {
                measure,
                meter: earlier,
            }],
        );
        if let Some(held) = self.measures_mut().get_mut(&measure) {
            held.set_meter(meter);
        }
        self.move_measures_after(start, by);
        self.set_tempo_map(map);
        Ok(outcome)
    }

    /// Move the start tick of every measure after `start` by `by`.
    ///
    /// It moves no note, no rest, no mark, and no spanner, which is what makes a
    /// meter change move the bar lines under the content. `shift_from` is the
    /// other rule, for a command that moves the content with the grid.
    fn move_measures_after(&mut self, start: Ticks, by: Ticks) {
        for record in self.measures_mut().values_mut() {
            let held = record.start();
            if held > start {
                record.set_start(held.saturating_add(by));
            }
        }
    }

    /// Add one score mark at one tick.
    ///
    /// It refuses nothing: a mark names a tick and no other entity.
    fn add_mark(&mut self, at: Ticks, kind: MarkKind) -> Outcome {
        let mark = MarkId::new(self.mint_id());
        let outcome = Outcome::new(
            vec![ScoreEvent::MarkAdded(mark)],
            vec![ScoreCommand::RemoveMark { mark }],
        );
        self.marks_mut()
            .insert(mark, ScoreMark::new(mark, at, kind));
        outcome
    }

    /// Remove one score mark.
    ///
    /// The inverse is a paste of the mark, which brings it back under its own
    /// identifier.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingElement` for an absent mark.
    fn remove_mark(&mut self, mark: MarkId) -> Result<Outcome, ScoreError> {
        let held = self
            .marks()
            .get(&mark)
            .ok_or(ScoreError::MissingElement(ElementRef::Mark(mark)))?
            .clone();
        let at = held.at();
        let clipboard = Clipboard::new(at, Vec::new(), Vec::new(), Vec::new(), vec![held]);
        let outcome = Outcome::new(
            vec![ScoreEvent::MarkRemoved(mark)],
            restore_pastes(&clipboard, at),
        );
        self.marks_mut().remove(&mark);
        Ok(outcome)
    }
}

impl Score {
    /// Insert one note.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingStaff` for an absent staff and for a voice
    /// the staff does not hold, and `ScoreError::OverlappingNote` for an onset
    /// that a note or a rest of that staff and voice already fills.
    fn insert_note(
        &mut self,
        staff: StaffId,
        voice: VoiceId,
        onset: Ticks,
        pitch: Pitch,
        duration: Duration,
    ) -> Result<Outcome, ScoreError> {
        self.voice_in(staff, voice)?;
        self.check_free(staff, voice, onset, duration, &BTreeSet::new())?;
        let note = NoteId::new(self.mint_id());
        let outcome = Outcome::new(
            vec![ScoreEvent::NoteInserted(note)],
            vec![ScoreCommand::Remove {
                selection: Selection::Notes(vec![note]),
            }],
        );
        self.notes_mut()
            .insert(note, Note::new(note, staff, voice, onset, duration, pitch));
        Ok(outcome)
    }

    /// Insert one rest.
    ///
    /// # Errors
    /// Returns what `insert_note` returns.
    fn insert_rest(
        &mut self,
        staff: StaffId,
        voice: VoiceId,
        onset: Ticks,
        duration: Duration,
    ) -> Result<Outcome, ScoreError> {
        self.voice_in(staff, voice)?;
        self.check_free(staff, voice, onset, duration, &BTreeSet::new())?;
        let rest = NoteId::new(self.mint_id());
        let outcome = Outcome::new(
            vec![ScoreEvent::NoteInserted(rest)],
            vec![ScoreCommand::Remove {
                selection: Selection::Notes(vec![rest]),
            }],
        );
        self.rests_mut()
            .insert(rest, Rest::new(rest, staff, voice, onset, duration));
        Ok(outcome)
    }

    /// Change the pitch of every named note.
    ///
    /// An empty note list changes nothing and answers an empty inverse.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingElement` for an absent note.
    fn set_pitch(&mut self, notes: &[NoteId], pitch: PitchEdit) -> Result<Outcome, ScoreError> {
        self.check_notes(notes)?;
        let mut events = Vec::with_capacity(notes.len());
        let mut inverse = Vec::with_capacity(notes.len());
        let mut changes = Vec::with_capacity(notes.len());
        for id in notes {
            let earlier = self.note_of(*id)?.pitch();
            events.push(ScoreEvent::NoteChanged(*id));
            inverse.push(ScoreCommand::SetPitch {
                notes: vec![*id],
                pitch: PitchEdit::Absolute(earlier),
            });
            changes.push((*id, edited_pitch(earlier, pitch)));
        }
        let outcome = Outcome::new(events, inverse);
        for (id, next) in changes {
            if let Some(note) = self.notes_mut().get_mut(&id) {
                note.set_pitch(next);
            }
        }
        Ok(outcome)
    }

    /// Change the duration of every named note.
    ///
    /// A duration that makes a note reach the next note or rest of the same
    /// staff and voice is refused. An empty note list changes nothing and
    /// answers an empty inverse.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingElement` for an absent note, and
    /// `ScoreError::OverlappingNote` for a duration that reaches another
    /// element.
    fn set_duration(
        &mut self,
        notes: &[NoteId],
        duration: Duration,
    ) -> Result<Outcome, ScoreError> {
        self.check_notes(notes)?;
        let mut events = Vec::with_capacity(notes.len());
        let mut inverse = Vec::with_capacity(notes.len());
        for id in notes {
            let note = self.note_of(*id)?;
            let staff = note.staff();
            let voice = note.voice();
            let onset = note.onset();
            let earlier = note.duration();
            let mut skip = BTreeSet::new();
            skip.insert(*id);
            self.check_free(staff, voice, onset, duration, &skip)?;
            events.push(ScoreEvent::NoteChanged(*id));
            inverse.push(ScoreCommand::SetDuration {
                notes: vec![*id],
                duration: earlier,
            });
        }
        let outcome = Outcome::new(events, inverse);
        for id in notes {
            if let Some(note) = self.notes_mut().get_mut(id) {
                note.set_duration(duration);
            }
        }
        Ok(outcome)
    }

    /// Change the tie of one note.
    ///
    /// A tie holds one pitch, so a tie that starts at a note whose partner
    /// carries another pitch is refused. The partner is the next note of the
    /// same staff and the same voice, by onset and then by identifier. A tie
    /// that starts at the last note of its staff and voice is ACCEPTED, because
    /// a tie crosses a system and the refusal roster names no dangling start.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingElement` for an absent note, and
    /// `ScoreError::TieAcrossPitch` for a partner of another pitch.
    fn set_tie(&mut self, note: NoteId, tie: TieState) -> Result<Outcome, ScoreError> {
        let held = self.note_of(note)?;
        let earlier = held.tie();
        if tie.starts() {
            self.check_tie_partner(held)?;
        }
        let outcome = Outcome::new(
            vec![ScoreEvent::NoteChanged(note)],
            vec![ScoreCommand::SetTie { note, tie: earlier }],
        );
        if let Some(record) = self.notes_mut().get_mut(&note) {
            record.set_tie(tie);
        }
        Ok(outcome)
    }

    /// Refuse a tie that starts at `note` and reaches a note of another pitch.
    ///
    /// A note with no partner accepts the tie, because a tie crosses a system.
    ///
    /// # Errors
    /// Returns `ScoreError::TieAcrossPitch` for a partner of another pitch.
    fn check_tie_partner(&self, note: &Note) -> Result<(), ScoreError> {
        let Some(partner) = self.partner_of(note) else {
            return Ok(());
        };
        if self.note_of(partner)?.pitch() == note.pitch() {
            return Ok(());
        }
        Err(ScoreError::TieAcrossPitch {
            from: note.id(),
            to: partner,
        })
    }

    /// Replace the articulations of every named note.
    ///
    /// An empty note list changes nothing and answers an empty inverse.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingElement` for an absent note.
    fn set_articulations(
        &mut self,
        notes: &[NoteId],
        articulations: &SmallVec<[Articulation; 2]>,
    ) -> Result<Outcome, ScoreError> {
        self.check_notes(notes)?;
        let mut events = Vec::with_capacity(notes.len());
        let mut inverse = Vec::with_capacity(notes.len());
        for id in notes {
            let earlier: SmallVec<[Articulation; 2]> =
                self.note_of(*id)?.articulations().iter().copied().collect();
            events.push(ScoreEvent::NoteChanged(*id));
            inverse.push(ScoreCommand::SetArticulations {
                notes: vec![*id],
                articulations: earlier,
            });
        }
        let outcome = Outcome::new(events, inverse);
        for id in notes {
            if let Some(note) = self.notes_mut().get_mut(id) {
                note.set_articulations(articulations.clone());
            }
        }
        Ok(outcome)
    }

    /// Put one lyric syllable on one note, in one verse.
    ///
    /// A verse that already holds a syllable takes the new one, and the inverse
    /// writes the earlier text back. A verse that held none inverts to a
    /// `Remove` and a `Paste` of the earlier note, because no command of the
    /// declared set takes a syllable off a note and `LyricText` refuses an
    /// empty string. `escalation:T2-10` records the gap.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingElement` for an absent note.
    fn set_lyric(
        &mut self,
        note: NoteId,
        verse: VerseNumber,
        text: LyricText,
    ) -> Result<Outcome, ScoreError> {
        let earlier = self
            .note_of(note)?
            .lyrics()
            .iter()
            .find(|lyric| lyric.verse() == verse)
            .map(|lyric| lyric.text().clone());
        let inverse = earlier.map_or_else(
            || self.restore_of(&[note]),
            |kept| {
                vec![ScoreCommand::SetLyric {
                    note,
                    verse,
                    text: kept,
                }]
            },
        );
        let outcome = Outcome::new(vec![ScoreEvent::NoteChanged(note)], inverse);
        if let Some(record) = self.notes_mut().get_mut(&note) {
            record.set_lyric(verse, text);
        }
        Ok(outcome)
    }

    /// Put a dynamic mark on one staff at one tick.
    ///
    /// The declared model holds no dynamic: `Score` carries no dynamics map and
    /// `Note` carries no dynamic field. This command therefore checks the
    /// staff, changes nothing, reports no event, and inverts to itself. A
    /// dynamic mark is not representable in version one, and `escalation:T2-10`
    /// records the gap.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingStaff` for an absent staff.
    fn set_dynamic(
        &self,
        staff: StaffId,
        onset: Ticks,
        dynamic: Dynamic,
    ) -> Result<Outcome, ScoreError> {
        self.staff_of(staff)?;
        Ok(Outcome::new(
            Vec::new(),
            vec![ScoreCommand::SetDynamic {
                staff,
                onset,
                dynamic,
            }],
        ))
    }

    /// Add one spanner between two notes.
    ///
    /// A spanner between two staves is legal, because section 3.1 states that a
    /// slur crosses a staff.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingElement` for an absent endpoint, and names
    /// the start before the end.
    fn add_spanner(
        &mut self,
        kind: SpannerKind,
        from: NoteId,
        to: NoteId,
    ) -> Result<Outcome, ScoreError> {
        self.note_of(from)?;
        self.note_of(to)?;
        let spanner = SpannerId::new(self.mint_id());
        let outcome = Outcome::new(
            vec![ScoreEvent::SpannerAdded(spanner)],
            vec![ScoreCommand::RemoveSpanner { spanner }],
        );
        self.spanners_mut()
            .insert(spanner, Spanner::new(spanner, kind, from, to));
        Ok(outcome)
    }

    /// Remove one spanner.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingElement` for an absent spanner.
    fn remove_spanner(&mut self, spanner: SpannerId) -> Result<Outcome, ScoreError> {
        let held = self
            .spanners()
            .get(&spanner)
            .ok_or(ScoreError::MissingElement(ElementRef::Spanner(spanner)))?
            .clone();
        let clipboard = Clipboard::new(Ticks::ZERO, Vec::new(), Vec::new(), vec![held], Vec::new());
        let outcome = Outcome::new(
            vec![ScoreEvent::SpannerRemoved(spanner)],
            restore_pastes(&clipboard, Ticks::ZERO),
        );
        self.spanners_mut().remove(&spanner);
        Ok(outcome)
    }
}

impl Score {
    /// Move every named note into one voice.
    ///
    /// A voice belongs to one staff, so the target voice is checked against the
    /// staff of EVERY named note: a note keeps its own staff, and a note whose
    /// staff does not hold the target voice would carry a dangling reference.
    ///
    /// An empty note list changes nothing and answers an empty inverse.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingElement` for an absent note,
    /// `ScoreError::MissingStaff` for a voice that the staff of a named note
    /// does not hold, and `ScoreError::OverlappingNote` for a note that the
    /// target voice already sounds over and for two named notes that would
    /// cover one tick range of it.
    fn set_voice(&mut self, notes: &[NoteId], voice: VoiceId) -> Result<Outcome, ScoreError> {
        self.check_notes(notes)?;
        let skip: BTreeSet<NoteId> = notes.iter().copied().collect();
        let mut taken: Vec<Taken> = Vec::with_capacity(notes.len());
        let mut events = Vec::with_capacity(notes.len());
        let mut inverse = Vec::with_capacity(notes.len());
        for id in notes {
            let note = self.note_of(*id)?;
            let staff = note.staff();
            let earlier = note.voice();
            let onset = note.onset();
            let duration = note.duration();
            self.voice_in(staff, voice)?;
            self.check_free(staff, voice, onset, duration, &skip)?;
            let next = Taken::new(staff, voice, onset, duration);
            check_taken(&taken, &next)?;
            taken.push(next);
            events.push(ScoreEvent::NoteChanged(*id));
            inverse.push(ScoreCommand::SetVoice {
                notes: vec![*id],
                voice: earlier,
            });
        }
        let outcome = Outcome::new(events, inverse);
        for id in notes {
            if let Some(note) = self.notes_mut().get_mut(id) {
                let held = note.staff();
                note.set_place(held, voice);
            }
        }
        Ok(outcome)
    }

    /// Move every named note into one staff of one part.
    ///
    /// The notes take the first voice of the target staff, so the inverse names
    /// the earlier staff AND the earlier voice of every note.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingPart` for an absent part,
    /// `ScoreError::MissingStaff` for a staff the part does not hold,
    /// `ScoreError::MissingElement` for an absent note, and
    /// `ScoreError::OverlappingNote` for a note the target already sounds over
    /// and for two named notes that would cover one tick range of it.
    fn set_part(
        &mut self,
        notes: &[NoteId],
        part: PartId,
        staff: StaffId,
    ) -> Result<Outcome, ScoreError> {
        let Some(owner) = self.parts().get(&part) else {
            return Err(ScoreError::MissingPart(part));
        };
        if !owner.staves().contains(&staff) {
            return Err(ScoreError::MissingStaff(staff));
        }
        let voice = self.first_voice(staff)?;
        self.check_notes(notes)?;
        let skip: BTreeSet<NoteId> = notes.iter().copied().collect();
        let mut taken: Vec<Taken> = Vec::with_capacity(notes.len());
        let mut events = Vec::with_capacity(notes.len());
        let mut inverse = Vec::with_capacity(notes.len());
        for id in notes {
            let note = self.note_of(*id)?;
            let earlier_staff = note.staff();
            let earlier_voice = note.voice();
            let onset = note.onset();
            let duration = note.duration();
            let Some(earlier_part) = self.part_of_staff(earlier_staff) else {
                return Err(ScoreError::MissingStaff(earlier_staff));
            };
            self.check_free(staff, voice, onset, duration, &skip)?;
            let next = Taken::new(staff, voice, onset, duration);
            check_taken(&taken, &next)?;
            taken.push(next);
            events.push(ScoreEvent::NoteChanged(*id));
            inverse.push(ScoreCommand::SetPart {
                notes: vec![*id],
                part: earlier_part,
                staff: earlier_staff,
            });
            inverse.push(ScoreCommand::SetVoice {
                notes: vec![*id],
                voice: earlier_voice,
            });
        }
        let outcome = Outcome::new(events, inverse);
        for id in notes {
            if let Some(note) = self.notes_mut().get_mut(id) {
                note.set_place(staff, voice);
            }
        }
        Ok(outcome)
    }
}

impl Score {
    /// Move a selection along the timeline, and to another staff.
    ///
    /// A spanner carries no onset, so a move leaves it where it is. A mark
    /// moves by the same tick count as a note.
    ///
    /// # Errors
    /// Returns what `resolve` returns, `ScoreError::MissingStaff` for a target
    /// staff the score does not hold, and `ScoreError::OverlappingNote` for a
    /// target that a note or a rest already fills.
    fn move_selection(
        &mut self,
        selection: &Selection,
        by: Ticks,
        to_staff: Option<StaffId>,
    ) -> Result<Outcome, ScoreError> {
        let resolved = self.resolve(selection)?;
        let skip = resolved.element_set();
        let mut taken: Vec<Taken> = Vec::new();
        let mut places = Vec::new();
        for id in resolved.elements() {
            let Some((staff, voice, onset, duration)) = self.place_of(id) else {
                continue;
            };
            let target_staff = to_staff.unwrap_or(staff);
            let target_voice = match to_staff {
                Some(_) => self.first_voice(target_staff)?,
                None => voice,
            };
            let moved = onset.saturating_add(by);
            self.check_free(target_staff, target_voice, moved, duration, &skip)?;
            let next = Taken::new(target_staff, target_voice, moved, duration);
            check_taken(&taken, &next)?;
            taken.push(next);
            places.push(Placement {
                id,
                staff: target_staff,
                voice: target_voice,
                onset: moved,
                earlier_staff: staff,
                earlier_voice: voice,
            });
        }
        let inverse = move_inverse(selection, by, to_staff, &places);
        let outcome = Outcome::new(vec![ScoreEvent::ElementsMoved(selection.clone())], inverse);
        for place in places {
            self.place_element(place.id, place.staff, place.voice, place.onset);
        }
        for mark in &resolved.marks {
            if let Some(record) = self.marks_mut().get_mut(mark) {
                let at = record.at();
                record.set_at(at.saturating_add(by));
            }
        }
        Ok(outcome)
    }

    /// Put one note or rest at a new staff, a new voice, and a new onset.
    fn place_element(&mut self, id: NoteId, staff: StaffId, voice: VoiceId, onset: Ticks) {
        if let Some(note) = self.notes_mut().get_mut(&id) {
            note.set_place(staff, voice);
            note.set_onset(onset);
            return;
        }
        if let Some(rest) = self.rests_mut().get_mut(&id) {
            rest.set_place(staff, voice);
            rest.set_onset(onset);
        }
    }

    /// Copy a selection to another tick and keep the original.
    ///
    /// Every copy keeps the staff and the voice of the element it came from. An
    /// empty selection changes nothing and answers an empty inverse.
    ///
    /// # Errors
    /// Returns what `resolve` and `paste_clipboard` return.
    fn duplicate(&mut self, selection: &Selection, at: Ticks) -> Result<Outcome, ScoreError> {
        let resolved = self.resolve(selection)?;
        let origin = self.origin_of(selection, &resolved);
        let clipboard = self.clipboard_of(&resolved, origin);
        self.paste_clipboard(&clipboard, at, None)
    }

    /// Remove a selection, and every spanner that names a removed element.
    ///
    /// The inverse is the pastes of the copy that this call takes first, so
    /// every removed element comes back under its own identifier. One `Paste`
    /// names one staff and one voice, so `restore_pastes` builds one for each
    /// `(staff, voice)` pair of the removed content and one more for the
    /// spanners and the marks. A removal that spans two staves therefore comes
    /// back into two staves.
    ///
    /// An empty selection removes nothing and answers an empty inverse.
    ///
    /// # Errors
    /// Returns what `resolve` returns.
    fn remove(&mut self, selection: &Selection) -> Result<Outcome, ScoreError> {
        let resolved = self.resolve(selection)?;
        let origin = self.origin_of(selection, &resolved);
        let clipboard = self.clipboard_of(&resolved, origin);
        let outcome = Outcome::new(
            vec![ScoreEvent::ElementsRemoved(selection.clone())],
            restore_pastes(&clipboard, origin),
        );
        for id in resolved.elements() {
            self.notes_mut().remove(&id);
            self.rests_mut().remove(&id);
        }
        for spanner in &resolved.spanners {
            self.spanners_mut().remove(spanner);
        }
        for mark in &resolved.marks {
            self.marks_mut().remove(mark);
        }
        Ok(outcome)
    }

    /// Insert a detached copy at one tick, in one staff and voice.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingStaff` for an absent staff or voice when the
    /// clipboard holds a note or a rest, and what `paste_clipboard` returns.
    fn paste(
        &mut self,
        clipboard: &Clipboard,
        at: Ticks,
        staff: StaffId,
        voice: VoiceId,
    ) -> Result<Outcome, ScoreError> {
        if clipboard.notes().is_empty() && clipboard.rests().is_empty() {
            return self.paste_clipboard(clipboard, at, None);
        }
        self.voice_in(staff, voice)?;
        self.paste_clipboard(clipboard, at, Some((staff, voice)))
    }

    /// Refuse a clipboard spanner that names a note the paste leaves dangling.
    ///
    /// `add_spanner` refuses a spanner that names a note the score does not
    /// hold, and this is that one rule at the other entry point into the
    /// spanner map. An endpoint is legal when the clipboard carries the note,
    /// because the paste inserts that note and remaps the endpoint onto it, or
    /// when the score already holds the note, because the endpoint then names
    /// it unchanged.
    ///
    /// `Score::copy` carries every spanner that touches a selected note,
    /// whatever staff the other endpoint stands in, so a clipboard holds
    /// spanners whose second endpoint it does not carry. Where the score has
    /// lost that note too, the paste would write a spanner that names nothing,
    /// and `canonical::write` and `canonical::read` would make the state
    /// durable. The refusal never fires on a well-ordered undo, because
    /// `restore_pastes` puts the spanners in the last `Paste` of the list, after
    /// every note that they can name is back.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingElement` for the first endpoint that neither
    /// the clipboard nor the score holds.
    fn check_paste_spanners(&self, clipboard: &Clipboard) -> Result<(), ScoreError> {
        let carried: BTreeSet<NoteId> = clipboard.notes().iter().map(Note::id).collect();
        let dangling = clipboard
            .spanners()
            .iter()
            .flat_map(|spanner| [spanner.from(), spanner.to()])
            .find(|endpoint| !carried.contains(endpoint) && !self.notes().contains_key(endpoint));
        dangling.map_or(Ok(()), |endpoint| {
            Err(ScoreError::MissingElement(ElementRef::Note(endpoint)))
        })
    }

    /// The identifier that `wanted` keeps, or a new one where it is taken.
    ///
    /// A kept identifier is reserved, so the counter stands above it and no
    /// later mint aliases it. `Score::reserve_id` states why.
    fn free_note_id(&mut self, wanted: NoteId, taken: &BTreeSet<NoteId>) -> NoteId {
        if self.notes().contains_key(&wanted)
            || self.rests().contains_key(&wanted)
            || taken.contains(&wanted)
        {
            return NoteId::new(self.mint_id());
        }
        self.reserve_id(wanted.get());
        wanted
    }

    /// Insert a detached copy at `at`.
    ///
    /// Every element keeps the identifier that the clipboard carries when the
    /// score does not already hold it, and takes a new one when the score does.
    /// A cut and a paste of one selection therefore return the score to its
    /// earlier content, and every reference into that selection survives.
    ///
    /// `to` names the staff and the voice that take every note and rest. A
    /// duplicate passes `None`, and every element then keeps its own. An empty
    /// clipboard changes nothing and answers an empty inverse.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingElement` for a clipboard spanner that names
    /// a note the paste leaves dangling, `ScoreError::OverlappingNote` for a
    /// target that a note or a rest already fills, and `OverlappingNote` for two
    /// clipboard elements that would cover one tick range of the target: a paste
    /// that names one staff and one voice sends every element of the clipboard
    /// there, whatever staff each came from.
    fn paste_clipboard(
        &mut self,
        clipboard: &Clipboard,
        at: Ticks,
        to: Option<(StaffId, VoiceId)>,
    ) -> Result<Outcome, ScoreError> {
        self.check_paste_spanners(clipboard)?;
        let shift = at.saturating_sub(clipboard.origin());
        let free = BTreeSet::new();
        let mut taken: Vec<Taken> = Vec::new();
        for note in clipboard.notes() {
            let (staff, voice) = to.unwrap_or_else(|| (note.staff(), note.voice()));
            let onset = note.onset().saturating_add(shift);
            self.check_free(staff, voice, onset, note.duration(), &free)?;
            let next = Taken::new(staff, voice, onset, note.duration());
            check_taken(&taken, &next)?;
            taken.push(next);
        }
        for rest in clipboard.rests() {
            let (staff, voice) = to.unwrap_or_else(|| (rest.staff(), rest.voice()));
            let onset = rest.onset().saturating_add(shift);
            self.check_free(staff, voice, onset, rest.duration(), &free)?;
            let next = Taken::new(staff, voice, onset, rest.duration());
            check_taken(&taken, &next)?;
            taken.push(next);
        }
        let pasted = self.mint_paste(clipboard, shift, to);
        let outcome = Outcome::new(pasted.events.clone(), pasted.inverse.clone());
        self.commit_paste(pasted);
        Ok(outcome)
    }

    /// Mint the identifiers of one paste and build its events and its inverse.
    fn mint_paste(
        &mut self,
        clipboard: &Clipboard,
        shift: Ticks,
        to: Option<(StaffId, VoiceId)>,
    ) -> Pasted {
        let mut taken: BTreeSet<NoteId> = BTreeSet::new();
        let mut renamed: BTreeMap<NoteId, NoteId> = BTreeMap::new();
        let mut notes = Vec::with_capacity(clipboard.notes().len());
        let mut rests = Vec::with_capacity(clipboard.rests().len());
        let mut events = Vec::new();
        for note in clipboard.notes() {
            let id = self.free_note_id(note.id(), &taken);
            taken.insert(id);
            renamed.insert(note.id(), id);
            let (staff, voice) = to.unwrap_or_else(|| (note.staff(), note.voice()));
            let mut placed = note.clone().with_id(id);
            placed.set_place(staff, voice);
            placed.set_onset(note.onset().saturating_add(shift));
            events.push(ScoreEvent::NoteInserted(id));
            notes.push(placed);
        }
        for rest in clipboard.rests() {
            let id = self.free_note_id(rest.id(), &taken);
            taken.insert(id);
            renamed.insert(rest.id(), id);
            let (staff, voice) = to.unwrap_or_else(|| (rest.staff(), rest.voice()));
            let mut placed = rest.clone().with_id(id);
            placed.set_place(staff, voice);
            placed.set_onset(rest.onset().saturating_add(shift));
            events.push(ScoreEvent::NoteInserted(id));
            rests.push(placed);
        }
        let spanners = self.mint_spanners(clipboard, &renamed, &mut events);
        let marks = self.mint_marks(clipboard, shift, &mut events);
        let inverse = paste_inverse(&notes, &rests, &spanners, &marks);
        Pasted {
            notes,
            rests,
            spanners,
            marks,
            events,
            inverse,
        }
    }

    /// Mint the spanners of one paste, with their two endpoints remapped.
    fn mint_spanners(
        &mut self,
        clipboard: &Clipboard,
        renamed: &BTreeMap<NoteId, NoteId>,
        events: &mut Vec<ScoreEvent>,
    ) -> Vec<Spanner> {
        let mut minted = Vec::with_capacity(clipboard.spanners().len());
        for spanner in clipboard.spanners() {
            let id = if self.spanners().contains_key(&spanner.id()) {
                SpannerId::new(self.mint_id())
            } else {
                self.reserve_id(spanner.id().get());
                spanner.id()
            };
            let from = renamed
                .get(&spanner.from())
                .copied()
                .unwrap_or_else(|| spanner.from());
            let to = renamed
                .get(&spanner.to())
                .copied()
                .unwrap_or_else(|| spanner.to());
            events.push(ScoreEvent::SpannerAdded(id));
            minted.push(spanner.clone().with_ids(id, from, to));
        }
        minted
    }

    /// Mint the marks of one paste, moved by `shift`.
    fn mint_marks(
        &mut self,
        clipboard: &Clipboard,
        shift: Ticks,
        events: &mut Vec<ScoreEvent>,
    ) -> Vec<ScoreMark> {
        let mut minted = Vec::with_capacity(clipboard.marks().len());
        for mark in clipboard.marks() {
            let id = if self.marks().contains_key(&mark.id()) {
                MarkId::new(self.mint_id())
            } else {
                self.reserve_id(mark.id().get());
                mark.id()
            };
            let mut placed = mark.clone().with_id(id);
            placed.set_at(mark.at().saturating_add(shift));
            events.push(ScoreEvent::MarkAdded(id));
            minted.push(placed);
        }
        minted
    }

    /// Put the elements of one paste into the score.
    fn commit_paste(&mut self, pasted: Pasted) {
        for note in pasted.notes {
            self.notes_mut().insert(note.id(), note);
        }
        for rest in pasted.rests {
            self.rests_mut().insert(rest.id(), rest);
        }
        for spanner in pasted.spanners {
            self.spanners_mut().insert(spanner.id(), spanner);
        }
        for mark in pasted.marks {
            self.marks_mut().insert(mark.id(), mark);
        }
    }
}
#[cfg(test)]
mod tests {
    use core::num::{NonZeroU8, NonZeroU16};
    use std::collections::BTreeSet;

    use duet_time::{Meter, NoteValue, Ticks};
    use smallvec::SmallVec;

    use super::Applied;
    use crate::command::{PitchEdit, ScoreCommand, Selection};
    use crate::error::ScoreError;
    use crate::event::ScoreEvent;
    use crate::ids::{
        ElementRef, LyricText, MarkId, MeasureId, NoteId, PartId, PartName, RehearsalText,
        SpannerId, StaffId, VerseNumber, VoiceId,
    };
    use crate::model::{
        Articulation, Clef, Duration, Dynamic, KeySignature, MarkKind, Note, Pitch, Score,
        SpannerKind, Step, TieState, Voice, VoiceType,
    };

    /// A part identifier that no score of this module mints.
    const ABSENT_PART: PartId = PartId::new(9_000);

    /// A staff identifier that no score of this module mints.
    const ABSENT_STAFF: StaffId = StaffId::new(9_001);

    /// A measure identifier that no score of this module mints.
    const ABSENT_MEASURE: MeasureId = MeasureId::new(9_002);

    /// A note identifier that no score of this module mints.
    const ABSENT_NOTE: NoteId = NoteId::new(9_003);

    /// A spanner identifier that no score of this module mints.
    const ABSENT_SPANNER: SpannerId = SpannerId::new(9_004);

    /// A mark identifier that no score of this module mints.
    const ABSENT_MARK: MarkId = MarkId::new(9_005);

    /// A voice identifier that no score of this module mints.
    const ABSENT_VOICE: VoiceId = VoiceId::new(9_006);

    /// The identifier of the second voice that `add_voice` writes into a staff.
    const SECOND_VOICE: VoiceId = VoiceId::new(8_000);

    /// A run of one measure.
    const ONE_MEASURE: NonZeroU16 = NonZeroU16::MIN;

    /// A run of two measures.
    const TWO_MEASURES: NonZeroU16 = NonZeroU16::MIN.saturating_add(1);

    /// Declare every arm of `ScoreCommand` once, for the match and for the list.
    ///
    /// The inverse property is the Resilient property of this chunk, and a
    /// command with no case is a command whose inverse nothing reads.
    /// `every_command_arm_has_an_inverse_case` holds the two case tables
    /// against `COMMAND_ARMS`, so `COMMAND_ARMS` is the denominator of that
    /// guard.
    ///
    /// **One invocation writes both items, so the denominator comes from a
    /// match that the compiler checks.** The match of `command_arm` is
    /// exhaustive, so a new arm of `ScoreCommand` stops this module from
    /// building until the invocation below names it, and naming it puts the arm
    /// into `COMMAND_ARMS` in the same edit. A hand-kept list can hold 27 names
    /// beside a 28th arm that no case covers, and the guard would then report a
    /// completeness it did not earn.
    macro_rules! command_arms {
        ($($arm:ident),+ $(,)?) => {
            /// Every arm of `ScoreCommand`, by the name that `command_arm` answers.
            const COMMAND_ARMS: &[&str] = &[$(stringify!($arm)),+];

            /// The name of the arm that `command` names.
            fn command_arm(command: &ScoreCommand) -> &'static str {
                match *command {
                    $(ScoreCommand::$arm { .. } => stringify!($arm),)+
                }
            }
        };
    }

    command_arms!(
        AddPart,
        RemovePart,
        AddStaff,
        RemoveStaff,
        SetClef,
        InsertMeasures,
        RemoveMeasures,
        SetKeySignature,
        SetTimeSignature,
        AddMark,
        RemoveMark,
        InsertNote,
        InsertRest,
        SetPitch,
        SetDuration,
        SetTie,
        SetArticulations,
        SetLyric,
        SetDynamic,
        AddSpanner,
        RemoveSpanner,
        SetVoice,
        SetPart,
        Move,
        Duplicate,
        Remove,
        Paste,
    );

    /// The tick count of one quarter note.
    const QUARTER_TICKS: i64 = 1_920;

    /// A score with one part and one staff, and the identifiers of each.
    struct Stage {
        /// The score under test.
        score: Score,
        /// The part that the stage added.
        part: PartId,
        /// The staff that the stage added to that part.
        staff: StaffId,
        /// The voice that the staff holds.
        voice: VoiceId,
    }

    /// One command that the score refuses, and the score it acts on.
    struct RefusalCase {
        /// The command that the case exercises.
        name: &'static str,
        /// The score that takes the command.
        score: Score,
        /// The command that the score refuses.
        command: ScoreCommand,
        /// The refusal that the command must answer with.
        refusal: ScoreError,
    }

    /// One command that the score accepts, and the score it acts on.
    struct InverseCase {
        /// The command that the case exercises.
        name: &'static str,
        /// The score that takes the command.
        score: Score,
        /// The command that the case applies and then undoes.
        command: ScoreCommand,
    }

    /// The part that `PartAdded` names in `applied`.
    fn part_of(applied: &Applied) -> PartId {
        applied
            .events
            .iter()
            .find_map(|event| {
                if let ScoreEvent::PartAdded(part) = *event {
                    Some(part)
                } else {
                    None
                }
            })
            .expect("an accepted AddPart reports PartAdded")
    }

    /// The staff that `StaffAdded` names in `applied`.
    fn staff_of(applied: &Applied) -> StaffId {
        applied
            .events
            .iter()
            .find_map(|event| {
                if let ScoreEvent::StaffAdded(staff) = *event {
                    Some(staff)
                } else {
                    None
                }
            })
            .expect("an accepted AddStaff reports StaffAdded")
    }

    /// The note that `NoteInserted` names in `applied`.
    fn note_of(applied: &Applied) -> NoteId {
        applied
            .events
            .iter()
            .find_map(|event| {
                if let ScoreEvent::NoteInserted(note) = *event {
                    Some(note)
                } else {
                    None
                }
            })
            .expect("an accepted InsertNote reports NoteInserted")
    }

    /// The mark that `MarkAdded` names in `applied`.
    fn mark_of(applied: &Applied) -> MarkId {
        applied
            .events
            .iter()
            .find_map(|event| {
                if let ScoreEvent::MarkAdded(mark) = *event {
                    Some(mark)
                } else {
                    None
                }
            })
            .expect("an accepted AddMark reports MarkAdded")
    }

    /// The spanner that `SpannerAdded` names in `applied`.
    fn spanner_of(applied: &Applied) -> SpannerId {
        applied
            .events
            .iter()
            .find_map(|event| {
                if let ScoreEvent::SpannerAdded(spanner) = *event {
                    Some(spanner)
                } else {
                    None
                }
            })
            .expect("an accepted AddSpanner reports SpannerAdded")
    }

    /// A quarter note with no dot and no tuplet.
    fn quarter() -> Duration {
        Duration::new(NoteValue::Quarter, 0, None)
    }

    /// A half note with no dot and no tuplet.
    fn half() -> Duration {
        Duration::new(NoteValue::Half, 0, None)
    }

    /// The pitch of `step` in octave four, with no alteration.
    fn natural(step: Step) -> Pitch {
        Pitch::new(4, step, 0)
    }

    /// A meter of three quarter beats in one bar.
    fn three_four() -> Meter {
        Meter::new(
            NonZeroU8::new(3).expect("three is not zero"),
            NoteValue::Quarter,
        )
    }

    /// The first verse.
    fn verse_one() -> VerseNumber {
        VerseNumber::new(NonZeroU8::MIN)
    }

    /// The lyric syllable that `text` spells.
    fn lyric(text: &str) -> LyricText {
        LyricText::new(text).expect("a syllable with text")
    }

    /// An `AddPart` command for a soprano part that `name` names.
    fn add_part(name: &str) -> ScoreCommand {
        ScoreCommand::AddPart {
            voice_type: VoiceType::Soprano,
            name: PartName::new(name).expect("a name with text"),
        }
    }

    /// A score with one part, one staff, and the voice of that staff.
    fn stage() -> Stage {
        let mut score = Score::new();
        let part_result = score
            .apply(add_part("Soprano 1"))
            .expect("AddPart is accepted");
        let part = part_of(&part_result);
        let staff_result = score
            .apply(ScoreCommand::AddStaff {
                part,
                clef: Clef::Treble,
            })
            .expect("AddStaff is accepted");
        let staff = staff_of(&staff_result);
        let voice = *score
            .staves()
            .get(&staff)
            .expect("the score holds the staff it just added")
            .voices()
            .first()
            .expect("AddStaff gives the staff one voice, because InsertNote names a voice");
        Stage {
            score,
            part,
            staff,
            voice,
        }
    }

    /// Insert one quarter note into `stage` and answer its identifier.
    fn insert_note(stage: &mut Stage, onset: i64, step: Step) -> NoteId {
        let inserted = stage
            .score
            .apply(ScoreCommand::InsertNote {
                staff: stage.staff,
                voice: stage.voice,
                onset: Ticks::new(onset),
                pitch: natural(step),
                duration: quarter(),
            })
            .expect("an InsertNote at a free onset is accepted");
        note_of(&inserted)
    }

    /// Add one more staff to the part of `stage`, and answer it with its voice.
    fn add_staff(stage: &mut Stage) -> (StaffId, VoiceId) {
        let added = stage
            .score
            .apply(ScoreCommand::AddStaff {
                part: stage.part,
                clef: Clef::Bass,
            })
            .expect("AddStaff is accepted");
        let staff = staff_of(&added);
        let voice = *stage
            .score
            .staves()
            .get(&staff)
            .expect("the score holds the staff it just added")
            .voices()
            .first()
            .expect("AddStaff gives the staff one voice");
        (staff, voice)
    }

    /// Give the staff of `stage` a second voice, and answer it.
    ///
    /// No command of the declared set mints a voice, so the fixture writes one
    /// through the crate-private mutators. A staff with two voices is the one
    /// shape in which a `SetVoice` moves a note from one voice of a staff into
    /// another.
    fn add_voice(stage: &mut Stage) -> VoiceId {
        let staff = stage.staff;
        stage.score.voices_mut().insert(
            SECOND_VOICE,
            Voice::new(
                SECOND_VOICE,
                NonZeroU8::new(2).expect("two is not zero"),
                false,
            ),
        );
        if let Some(record) = stage.score.staves_mut().get_mut(&staff) {
            record.push_voice(SECOND_VOICE);
        }
        SECOND_VOICE
    }

    /// The measures of `score`, ordered by start tick.
    fn measures_by_start(score: &Score) -> Vec<MeasureId> {
        let mut ordered: Vec<(Ticks, MeasureId)> = score
            .measures()
            .values()
            .map(|measure| (measure.start(), measure.id()))
            .collect();
        ordered.sort_unstable();
        ordered.into_iter().map(|(_, id)| id).collect()
    }

    /// The measure that stands second on the timeline of `score`.
    fn second_measure_of(score: &Score) -> MeasureId {
        *measures_by_start(score)
            .get(1)
            .expect("the fixture score holds at least two measures")
    }

    /// The start tick of every measure of `score`, in timeline order.
    fn starts_of(score: &Score) -> Vec<i64> {
        measures_by_start(score)
            .iter()
            .filter_map(|id| score.measures().get(id))
            .map(|measure| measure.start().get())
            .collect()
    }

    /// A score of two measures, and the measure that stands second.
    fn stage_with_two_measures() -> (Score, MeasureId) {
        let mut score = Score::new();
        let first = first_measure(&score);
        score
            .apply(ScoreCommand::InsertMeasures {
                after: first,
                count: ONE_MEASURE,
            })
            .expect("an InsertMeasures after the one measure of a new score is accepted");
        let second = *measures_by_start(&score)
            .get(1)
            .expect("the score holds two measures after the insert");
        (score, second)
    }

    /// Insert one quarter note into the named staff and voice of `stage`.
    fn insert_note_in(
        stage: &mut Stage,
        staff: StaffId,
        voice: VoiceId,
        onset: i64,
        step: Step,
    ) -> NoteId {
        let inserted = stage
            .score
            .apply(ScoreCommand::InsertNote {
                staff,
                voice,
                onset: Ticks::new(onset),
                pitch: natural(step),
                duration: quarter(),
            })
            .expect("an InsertNote at a free onset is accepted");
        note_of(&inserted)
    }

    /// A stage with two notes of one pitch, one quarter note apart.
    fn stage_with_two_notes() -> (Stage, NoteId, NoteId) {
        let mut stage = stage();
        let first = insert_note(&mut stage, 0, Step::C);
        let second = insert_note(&mut stage, QUARTER_TICKS, Step::C);
        (stage, first, second)
    }

    /// A stage with two notes one whole note apart, and the first of the two.
    ///
    /// `SetDuration` refuses a duration that makes a note reach the next note
    /// of the same staff and the same voice, so the note that grows to a half
    /// note needs a free half note after it.
    fn stage_with_a_distant_second_note() -> (Stage, NoteId) {
        let mut stage = stage();
        let first = insert_note(&mut stage, 0, Step::C);
        insert_note(&mut stage, QUARTER_TICKS * 4, Step::C);
        (stage, first)
    }

    /// The identifier of the one measure that a new score holds.
    fn first_measure(score: &Score) -> MeasureId {
        *score
            .measures()
            .keys()
            .next()
            .expect("a new score holds one measure")
    }

    /// The greatest identifier that `score` holds, over every keyspace.
    ///
    /// A score that holds none answers zero, which the first mint takes.
    fn greatest_identifier_of(score: &Score) -> u64 {
        let parts = score.parts().keys().map(|id| id.get());
        let staves = score.staves().keys().map(|id| id.get());
        let voices = score.voices().keys().map(|id| id.get());
        let measures = score.measures().keys().map(|id| id.get());
        let marks = score.marks().keys().map(|id| id.get());
        let notes = score.notes().keys().map(|id| id.get());
        let rests = score.rests().keys().map(|id| id.get());
        let spanners = score.spanners().keys().map(|id| id.get());
        parts
            .chain(staves)
            .chain(voices)
            .chain(measures)
            .chain(marks)
            .chain(notes)
            .chain(rests)
            .chain(spanners)
            .max()
            .unwrap_or(0)
    }

    /// The text of one block of a canonical document.
    fn text_of(block: &[u8], property: &str) -> String {
        match core::str::from_utf8(block) {
            Ok(text) => text.to_owned(),
            Err(error) => {
                panic!("{property}: the writer answers UTF-8 text, and it answered {error}")
            },
        }
    }

    /// Assert that two scores hold the same content, whatever their revision.
    fn assert_same_content(left: &Score, right: &Score, property: &str) {
        assert_eq!(left.parts(), right.parts(), "{property}: the parts match");
        assert_eq!(
            left.staves(),
            right.staves(),
            "{property}: the staves match"
        );
        assert_eq!(
            left.voices(),
            right.voices(),
            "{property}: the voices match"
        );
        assert_eq!(
            left.measures(),
            right.measures(),
            "{property}: the measures match"
        );
        assert_eq!(left.notes(), right.notes(), "{property}: the notes match");
        assert_eq!(left.rests(), right.rests(), "{property}: the rests match");
        assert_eq!(
            left.spanners(),
            right.spanners(),
            "{property}: the spanners match"
        );
        assert_eq!(left.marks(), right.marks(), "{property}: the marks match");
        assert_eq!(
            left.tempo_map(),
            right.tempo_map(),
            "{property}: the tempo map matches"
        );
        assert_eq!(
            left.extra(),
            right.extra(),
            "{property}: the bag of unknown fields matches, which the equality of Score reads and this helper must read too"
        );
    }

    /// The refusal that an insert over a sounding note answers with.
    fn overlap_refusal() -> RefusalCase {
        let mut overlap = stage();
        insert_note(&mut overlap, 0, Step::C);
        let staff = overlap.staff;
        let voice = overlap.voice;
        let onset = Ticks::new(0);
        RefusalCase {
            name: "InsertNote over a sounding note",
            score: overlap.score,
            command: ScoreCommand::InsertNote {
                staff,
                voice,
                onset,
                pitch: natural(Step::E),
                duration: quarter(),
            },
            refusal: ScoreError::OverlappingNote {
                staff,
                voice,
                onset,
            },
        }
    }

    /// The refusal that a tie between two pitches answers with.
    fn tie_refusal() -> RefusalCase {
        let mut tied = stage();
        let from = insert_note(&mut tied, 0, Step::C);
        let to = insert_note(&mut tied, QUARTER_TICKS, Step::D);
        RefusalCase {
            name: "SetTie across two pitches",
            score: tied.score,
            command: ScoreCommand::SetTie {
                note: from,
                tie: TieState::new(true, false),
            },
            refusal: ScoreError::TieAcrossPitch { from, to },
        }
    }

    /// The refusal that a remove of a filled measure answers with.
    fn filled_measure_refusal() -> RefusalCase {
        let mut filled = stage();
        insert_note(&mut filled, 0, Step::C);
        let measure = first_measure(&filled.score);
        RefusalCase {
            name: "RemoveMeasures over a filled measure",
            score: filled.score,
            command: ScoreCommand::RemoveMeasures {
                from: measure,
                count: ONE_MEASURE,
            },
            refusal: ScoreError::MeasureNotEmpty(measure),
        }
    }

    /// Every structure command of the set that names an absent element.
    fn absent_structure_commands() -> Vec<(&'static str, ScoreCommand, ScoreError)> {
        vec![
            (
                "AddStaff",
                ScoreCommand::AddStaff {
                    part: ABSENT_PART,
                    clef: Clef::Treble,
                },
                ScoreError::MissingPart(ABSENT_PART),
            ),
            (
                "RemovePart",
                ScoreCommand::RemovePart { part: ABSENT_PART },
                ScoreError::MissingPart(ABSENT_PART),
            ),
            (
                "RemoveStaff",
                ScoreCommand::RemoveStaff {
                    staff: ABSENT_STAFF,
                },
                ScoreError::MissingStaff(ABSENT_STAFF),
            ),
            (
                "SetClef",
                ScoreCommand::SetClef {
                    staff: ABSENT_STAFF,
                    at: Ticks::new(0),
                    clef: Clef::Bass,
                },
                ScoreError::MissingStaff(ABSENT_STAFF),
            ),
            (
                "SetKeySignature",
                ScoreCommand::SetKeySignature {
                    measure: ABSENT_MEASURE,
                    key: KeySignature::new(1, true),
                },
                ScoreError::MissingMeasure(ABSENT_MEASURE),
            ),
            (
                "SetTimeSignature",
                ScoreCommand::SetTimeSignature {
                    measure: ABSENT_MEASURE,
                    meter: three_four(),
                },
                ScoreError::MissingMeasure(ABSENT_MEASURE),
            ),
            (
                "InsertMeasures",
                ScoreCommand::InsertMeasures {
                    after: ABSENT_MEASURE,
                    count: ONE_MEASURE,
                },
                ScoreError::MissingMeasure(ABSENT_MEASURE),
            ),
            (
                "RemoveMeasures",
                ScoreCommand::RemoveMeasures {
                    from: ABSENT_MEASURE,
                    count: ONE_MEASURE,
                },
                ScoreError::MissingMeasure(ABSENT_MEASURE),
            ),
            (
                "RemoveMark",
                ScoreCommand::RemoveMark { mark: ABSENT_MARK },
                ScoreError::MissingElement(ElementRef::Mark(ABSENT_MARK)),
            ),
        ]
    }

    /// Every content command of the set that names an absent element.
    fn absent_content_commands(present: NoteId) -> Vec<(&'static str, ScoreCommand, ScoreError)> {
        vec![
            (
                "InsertNote",
                ScoreCommand::InsertNote {
                    staff: ABSENT_STAFF,
                    voice: ABSENT_VOICE,
                    onset: Ticks::new(0),
                    pitch: natural(Step::C),
                    duration: quarter(),
                },
                ScoreError::MissingStaff(ABSENT_STAFF),
            ),
            (
                "InsertRest",
                ScoreCommand::InsertRest {
                    staff: ABSENT_STAFF,
                    voice: ABSENT_VOICE,
                    onset: Ticks::new(0),
                    duration: quarter(),
                },
                ScoreError::MissingStaff(ABSENT_STAFF),
            ),
            (
                "SetDynamic",
                ScoreCommand::SetDynamic {
                    staff: ABSENT_STAFF,
                    onset: Ticks::new(0),
                    dynamic: Dynamic::Mf,
                },
                ScoreError::MissingStaff(ABSENT_STAFF),
            ),
            (
                "SetPitch",
                ScoreCommand::SetPitch {
                    notes: vec![ABSENT_NOTE],
                    pitch: PitchEdit::BySemitones(1),
                },
                ScoreError::MissingElement(ElementRef::Note(ABSENT_NOTE)),
            ),
            (
                "SetDuration",
                ScoreCommand::SetDuration {
                    notes: vec![ABSENT_NOTE],
                    duration: half(),
                },
                ScoreError::MissingElement(ElementRef::Note(ABSENT_NOTE)),
            ),
            (
                "SetTie",
                ScoreCommand::SetTie {
                    note: ABSENT_NOTE,
                    tie: TieState::new(true, false),
                },
                ScoreError::MissingElement(ElementRef::Note(ABSENT_NOTE)),
            ),
            (
                "SetLyric",
                ScoreCommand::SetLyric {
                    note: ABSENT_NOTE,
                    verse: verse_one(),
                    text: lyric("la"),
                },
                ScoreError::MissingElement(ElementRef::Note(ABSENT_NOTE)),
            ),
            (
                "AddSpanner",
                ScoreCommand::AddSpanner {
                    kind: SpannerKind::Slur,
                    from: present,
                    to: ABSENT_NOTE,
                },
                ScoreError::MissingElement(ElementRef::Note(ABSENT_NOTE)),
            ),
            (
                "RemoveSpanner",
                ScoreCommand::RemoveSpanner {
                    spanner: ABSENT_SPANNER,
                },
                ScoreError::MissingElement(ElementRef::Spanner(ABSENT_SPANNER)),
            ),
            (
                "SetPart",
                ScoreCommand::SetPart {
                    notes: vec![present],
                    part: ABSENT_PART,
                    staff: ABSENT_STAFF,
                },
                ScoreError::MissingPart(ABSENT_PART),
            ),
            (
                "Remove",
                ScoreCommand::Remove {
                    selection: Selection::Notes(vec![ABSENT_NOTE]),
                },
                ScoreError::MissingElement(ElementRef::Note(ABSENT_NOTE)),
            ),
            (
                "Move",
                ScoreCommand::Move {
                    selection: Selection::Notes(vec![ABSENT_NOTE]),
                    by: Ticks::new(QUARTER_TICKS),
                    to_staff: None,
                },
                ScoreError::MissingElement(ElementRef::Note(ABSENT_NOTE)),
            ),
            (
                "Duplicate",
                ScoreCommand::Duplicate {
                    selection: Selection::Notes(vec![ABSENT_NOTE]),
                    at: Ticks::new(QUARTER_TICKS * 2),
                },
                ScoreError::MissingElement(ElementRef::Note(ABSENT_NOTE)),
            ),
        ]
    }

    /// Every refusal that an absent element causes, over one prepared score.
    fn missing_refusals() -> Vec<RefusalCase> {
        let mut prepared = stage();
        let present = insert_note(&mut prepared, 0, Step::C);
        let score = prepared.score;
        absent_structure_commands()
            .into_iter()
            .chain(absent_content_commands(present))
            .map(|(name, command, refusal)| RefusalCase {
                name,
                score: score.clone(),
                command,
                refusal,
            })
            .collect()
    }

    /// Every refusal of the representative command set.
    fn refusal_cases() -> Vec<RefusalCase> {
        let mut cases = vec![overlap_refusal(), tie_refusal(), filled_measure_refusal()];
        cases.extend(missing_refusals());
        cases
    }

    /// The structure commands of the inverse set.
    fn structure_inverse_cases() -> Vec<InverseCase> {
        let part_stage = stage();
        let staff_stage = stage();
        let staff_part = staff_stage.part;
        let mark_stage = stage();
        vec![
            InverseCase {
                name: "AddPart",
                score: Score::new(),
                command: add_part("Alto 1"),
            },
            InverseCase {
                name: "AddStaff",
                score: staff_stage.score,
                command: ScoreCommand::AddStaff {
                    part: staff_part,
                    clef: Clef::Bass,
                },
            },
            InverseCase {
                name: "AddMark",
                score: mark_stage.score,
                command: ScoreCommand::AddMark {
                    at: Ticks::new(0),
                    kind: MarkKind::SystemBreak,
                },
            },
            InverseCase {
                name: "InsertNote",
                score: part_stage.score,
                command: ScoreCommand::InsertNote {
                    staff: part_stage.staff,
                    voice: part_stage.voice,
                    onset: Ticks::new(QUARTER_TICKS * 2),
                    pitch: natural(Step::G),
                    duration: quarter(),
                },
            },
        ]
    }

    /// The content commands of the inverse set.
    fn content_inverse_cases() -> Vec<InverseCase> {
        let rest_stage = stage();
        let (pitch_stage, pitch_note, _pitch_second) = stage_with_two_notes();
        let (duration_stage, duration_note) = stage_with_a_distant_second_note();
        let (tie_stage, tie_note, _tie_second) = stage_with_two_notes();
        let (lyric_stage, lyric_note, _lyric_second) = stage_with_two_notes();
        let (spanner_stage, spanner_from, spanner_to) = stage_with_two_notes();
        vec![
            InverseCase {
                name: "InsertRest",
                score: rest_stage.score,
                command: ScoreCommand::InsertRest {
                    staff: rest_stage.staff,
                    voice: rest_stage.voice,
                    onset: Ticks::new(QUARTER_TICKS * 2),
                    duration: quarter(),
                },
            },
            InverseCase {
                name: "SetPitch",
                score: pitch_stage.score,
                command: ScoreCommand::SetPitch {
                    notes: vec![pitch_note],
                    pitch: PitchEdit::Absolute(natural(Step::E)),
                },
            },
            InverseCase {
                name: "SetDuration",
                score: duration_stage.score,
                command: ScoreCommand::SetDuration {
                    notes: vec![duration_note],
                    duration: half(),
                },
            },
            InverseCase {
                name: "SetTie",
                score: tie_stage.score,
                command: ScoreCommand::SetTie {
                    note: tie_note,
                    tie: TieState::new(true, false),
                },
            },
            InverseCase {
                name: "SetLyric",
                score: lyric_stage.score,
                command: ScoreCommand::SetLyric {
                    note: lyric_note,
                    verse: verse_one(),
                    text: lyric("la"),
                },
            },
            InverseCase {
                name: "AddSpanner",
                score: spanner_stage.score,
                command: ScoreCommand::AddSpanner {
                    kind: SpannerKind::Slur,
                    from: spanner_from,
                    to: spanner_to,
                },
            },
        ]
    }

    /// The edit commands of the inverse set.
    fn edit_inverse_cases() -> Vec<InverseCase> {
        let (remove_stage, _remove_first, remove_second) = stage_with_two_notes();
        let (move_stage, _move_first, move_second) = stage_with_two_notes();
        let (copy_stage, copy_first, _copy_second) = stage_with_two_notes();
        vec![
            InverseCase {
                name: "Remove",
                score: remove_stage.score,
                command: ScoreCommand::Remove {
                    selection: Selection::Notes(vec![remove_second]),
                },
            },
            InverseCase {
                name: "Move",
                score: move_stage.score,
                command: ScoreCommand::Move {
                    selection: Selection::Notes(vec![move_second]),
                    by: Ticks::new(QUARTER_TICKS),
                    to_staff: None,
                },
            },
            InverseCase {
                name: "Duplicate",
                score: copy_stage.score,
                command: ScoreCommand::Duplicate {
                    selection: Selection::Notes(vec![copy_first]),
                    at: Ticks::new(QUARTER_TICKS * 2),
                },
            },
        ]
    }

    /// The measure and signature commands of the inverse set.
    fn signature_inverse_cases() -> Vec<InverseCase> {
        let clef_stage = stage();
        let clef_staff = clef_stage.staff;
        let key_score = Score::new();
        let key_measure = first_measure(&key_score);
        let insert_score = Score::new();
        let insert_after = first_measure(&insert_score);
        let (meter_score, meter_measure) = stage_with_two_measures();
        let mut mark_stage = stage();
        let mark = mark_of(
            &mark_stage
                .score
                .apply(ScoreCommand::AddMark {
                    at: Ticks::new(QUARTER_TICKS),
                    kind: MarkKind::Rehearsal(
                        RehearsalText::new("B").expect("a rehearsal text with text"),
                    ),
                })
                .expect("an AddMark is accepted"),
        );
        vec![
            InverseCase {
                name: "SetClef",
                score: clef_stage.score,
                command: ScoreCommand::SetClef {
                    staff: clef_staff,
                    at: Ticks::ZERO,
                    clef: Clef::Bass,
                },
            },
            InverseCase {
                name: "SetKeySignature",
                score: key_score,
                command: ScoreCommand::SetKeySignature {
                    measure: key_measure,
                    key: KeySignature::new(-3, false),
                },
            },
            InverseCase {
                name: "InsertMeasures",
                score: insert_score,
                command: ScoreCommand::InsertMeasures {
                    after: insert_after,
                    count: TWO_MEASURES,
                },
            },
            InverseCase {
                name: "SetTimeSignature",
                score: meter_score,
                command: ScoreCommand::SetTimeSignature {
                    measure: meter_measure,
                    meter: three_four(),
                },
            },
            InverseCase {
                name: "RemoveMark",
                score: mark_stage.score,
                command: ScoreCommand::RemoveMark { mark },
            },
        ]
    }

    /// The commands of the inverse set that move or mark one note.
    fn place_inverse_cases() -> Vec<InverseCase> {
        let (articulation_stage, articulation_note, _articulation_second) = stage_with_two_notes();
        let dynamic_stage = stage();
        let dynamic_staff = dynamic_stage.staff;
        let (mut spanner_stage, spanner_from, spanner_to) = stage_with_two_notes();
        let spanner = spanner_of(
            &spanner_stage
                .score
                .apply(ScoreCommand::AddSpanner {
                    kind: SpannerKind::Slur,
                    from: spanner_from,
                    to: spanner_to,
                })
                .expect("an AddSpanner between two notes the score holds is accepted"),
        );
        let mut voice_stage = stage();
        let voice_note = insert_note(&mut voice_stage, 0, Step::C);
        let second_voice = add_voice(&mut voice_stage);
        let mut part_stage = stage();
        let (lower_staff, lower_voice) = add_staff(&mut part_stage);
        let part_note = insert_note_in(&mut part_stage, lower_staff, lower_voice, 0, Step::G);
        let part_owner = part_stage.part;
        let part_target = part_stage.staff;
        let mut paste_stage = stage();
        let paste_note = insert_note(&mut paste_stage, 0, Step::C);
        let paste_clipboard = paste_stage
            .score
            .copy(&Selection::Notes(vec![paste_note]))
            .expect("a copy of one note the score holds is accepted");
        let paste_staff = paste_stage.staff;
        let paste_voice = paste_stage.voice;
        vec![
            InverseCase {
                name: "SetArticulations",
                score: articulation_stage.score,
                command: ScoreCommand::SetArticulations {
                    notes: vec![articulation_note],
                    articulations: SmallVec::from_slice(&[
                        Articulation::Staccato,
                        Articulation::Fermata,
                    ]),
                },
            },
            InverseCase {
                name: "SetDynamic",
                score: dynamic_stage.score,
                command: ScoreCommand::SetDynamic {
                    staff: dynamic_staff,
                    onset: Ticks::ZERO,
                    dynamic: Dynamic::Sfz,
                },
            },
            InverseCase {
                name: "RemoveSpanner",
                score: spanner_stage.score,
                command: ScoreCommand::RemoveSpanner { spanner },
            },
            InverseCase {
                name: "SetVoice",
                score: voice_stage.score,
                command: ScoreCommand::SetVoice {
                    notes: vec![voice_note],
                    voice: second_voice,
                },
            },
            InverseCase {
                name: "SetPart",
                score: part_stage.score,
                command: ScoreCommand::SetPart {
                    notes: vec![part_note],
                    part: part_owner,
                    staff: part_target,
                },
            },
            InverseCase {
                name: "Paste",
                score: paste_stage.score,
                command: ScoreCommand::Paste {
                    clipboard: Box::new(paste_clipboard),
                    at: Ticks::new(QUARTER_TICKS * 2),
                    staff: paste_staff,
                    voice: paste_voice,
                },
            },
        ]
    }

    /// Every command of the representative set that the score accepts.
    ///
    /// The three structural removals are NOT here: their inverse rebuilds
    /// equivalent structure under new identifiers, so they cannot pass the
    /// property. `structure_removal_arms` names them, and one test each pins
    /// what they do restore and what they do not.
    fn inverse_cases() -> Vec<InverseCase> {
        let mut cases = structure_inverse_cases();
        cases.extend(content_inverse_cases());
        cases.extend(edit_inverse_cases());
        cases.extend(signature_inverse_cases());
        cases.extend(place_inverse_cases());
        cases
    }

    /// The arms whose inverse rebuilds structure under new identifiers.
    ///
    /// No clipboard carries a part, a staff, a voice, or a measure, and no
    /// command of the declared set names an identifier for one, so these three
    /// restore the SHAPE of what they removed and not the thing itself.
    /// `escalation:T2-9` records the gap, and one test each holds the weaker
    /// property they do carry.
    fn structure_removal_arms() -> Vec<&'static str> {
        vec!["RemovePart", "RemoveStaff", "RemoveMeasures"]
    }

    /// Apply every inverse command of one transaction to `score`.
    fn undo_all(score: &mut Score, inverse: Vec<ScoreCommand>, property: &str) {
        for undo in inverse {
            if let Err(error) = score.apply(undo) {
                panic!("{property} inverts, and the undo answered {error}");
            }
        }
    }

    #[test]
    fn apply_rejects_overlapping_note() {
        let mut stage = stage();
        insert_note(&mut stage, 0, Step::C);
        let refusal = stage.score.apply(ScoreCommand::InsertNote {
            staff: stage.staff,
            voice: stage.voice,
            onset: Ticks::new(0),
            pitch: natural(Step::E),
            duration: quarter(),
        });
        assert_eq!(
            refusal,
            Err(ScoreError::OverlappingNote {
                staff: stage.staff,
                voice: stage.voice,
                onset: Ticks::new(0),
            }),
            "a second note at one onset in one staff and one voice gives OverlappingNote"
        );
    }

    #[test]
    fn apply_rejects_tie_across_pitch() {
        let mut stage = stage();
        let from = insert_note(&mut stage, 0, Step::C);
        let to = insert_note(&mut stage, QUARTER_TICKS, Step::D);
        let refusal = stage.score.apply(ScoreCommand::SetTie {
            note: from,
            tie: TieState::new(true, false),
        });
        assert_eq!(
            refusal,
            Err(ScoreError::TieAcrossPitch { from, to }),
            "a tie holds one pitch, so a tie to another pitch gives TieAcrossPitch"
        );
    }

    #[test]
    fn apply_rejects_remove_of_filled_measure() {
        let mut stage = stage();
        insert_note(&mut stage, 0, Step::C);
        let measure = first_measure(&stage.score);
        let refusal = stage.score.apply(ScoreCommand::RemoveMeasures {
            from: measure,
            count: ONE_MEASURE,
        });
        assert_eq!(
            refusal,
            Err(ScoreError::MeasureNotEmpty(measure)),
            "a measure that holds a note gives MeasureNotEmpty"
        );
    }

    #[test]
    fn apply_rejects_a_command_that_names_a_missing_element() {
        for case in missing_refusals() {
            let mut score = case.score;
            let before = score.clone();
            let refusal = score.apply(case.command);
            assert_eq!(
                refusal,
                Err(case.refusal),
                "{} names an element the score does not hold, so it refuses",
                case.name
            );
            assert_eq!(
                score, before,
                "{} changes nothing when it names a missing element",
                case.name
            );
        }
    }

    #[test]
    fn apply_leaves_no_partial_change() {
        for case in refusal_cases() {
            let mut score = case.score;
            let before = score.clone();
            let refusal = score.apply(case.command);
            assert_eq!(refusal, Err(case.refusal), "{} refuses", case.name);
            assert_eq!(
                score, before,
                "{} leaves the score equal to its own clone",
                case.name
            );
            assert_eq!(
                score.revision(),
                before.revision(),
                "{} leaves the revision unchanged",
                case.name
            );
            assert_eq!(
                score.next_id(),
                before.next_id(),
                "{} mints no identifier",
                case.name
            );
        }
    }

    #[test]
    fn apply_inverse_restores_the_score() {
        for case in inverse_cases() {
            let mut score = case.score;
            let before = score.clone();
            let applied = match score.apply(case.command) {
                Ok(applied) => applied,
                Err(error) => panic!("{} is accepted, and it answered {error}", case.name),
            };
            assert!(
                !applied.inverse.is_empty(),
                "{} gives at least one inverse command",
                case.name
            );
            undo_all(&mut score, applied.inverse, case.name);
            assert_same_content(&score, &before, case.name);
        }
    }

    #[test]
    fn every_command_arm_writes_a_document_that_the_reader_restores() {
        for case in inverse_cases() {
            let name = case.name;
            let mut score = case.score;
            if let Err(error) = score.apply(case.command) {
                panic!("{name} is accepted, and it answered {error}");
            }
            let document = match crate::canonical::write(&score) {
                Ok(document) => document,
                Err(error) => panic!("{name} writes a document, and the writer answered {error}"),
            };
            let meta = text_of(document.meta(), name);
            let notes = text_of(document.notes(), name);
            let spanners = text_of(document.spanners(), name);
            let restored = match crate::canonical::read(&meta, &notes, &spanners) {
                Ok(restored) => restored,
                Err(error) => panic!("{name} reads back, and the reader answered {error}"),
            };
            assert_same_content(&restored, &score, name);
            assert_eq!(
                restored.revision(),
                score.revision(),
                "{name}: the revision reaches the document and comes back"
            );
            let floor = greatest_identifier_of(&score).saturating_add(1);
            assert_eq!(
                restored.next_id(),
                score.next_id().max(floor),
                "{name}: the identifier counter reaches the document and comes back, raised above every identifier the document holds"
            );
        }
    }

    #[test]
    fn every_command_arm_has_an_inverse_case() {
        let mut covered: BTreeSet<&str> = inverse_cases()
            .iter()
            .map(|case| command_arm(&case.command))
            .collect();
        covered.extend(structure_removal_arms());
        let declared: BTreeSet<&str> = COMMAND_ARMS.iter().copied().collect();
        assert_eq!(
            covered, declared,
            "every arm of ScoreCommand stands in the inverse table or in the structure-removal table, so no broken inverse can hide in an arm that no test applies"
        );
    }

    #[test]
    fn the_inverse_of_a_part_removal_rebuilds_the_part_and_not_its_content() {
        let mut stage = stage();
        insert_note(&mut stage, 0, Step::C);
        let part = stage.part;

        let applied = stage
            .score
            .apply(ScoreCommand::RemovePart { part })
            .expect("a RemovePart of a part the score holds is accepted");
        assert!(
            !applied.inverse.is_empty(),
            "an accepted RemovePart carries an inverse"
        );
        undo_all(&mut stage.score, applied.inverse, "a RemovePart");

        assert_eq!(
            stage.score.parts().len(),
            1,
            "the inverse of a RemovePart rebuilds one part"
        );
        assert!(
            !stage.score.parts().contains_key(&part),
            "the rebuilt part takes a NEW identifier, which escalation:T2-9 records as the open gap"
        );
        let rebuilt = stage
            .score
            .parts()
            .values()
            .next()
            .expect("the score holds the rebuilt part");
        assert_eq!(
            rebuilt.voice_type(),
            VoiceType::Soprano,
            "the rebuilt part carries the voice type of the part that stood"
        );
        assert!(
            rebuilt.staves().is_empty(),
            "the inverse of a RemovePart restores no staff"
        );
        assert!(
            stage.score.staves().is_empty() && stage.score.voices().is_empty(),
            "the inverse of a RemovePart restores no staff record and no voice"
        );
        assert!(
            stage.score.notes().is_empty(),
            "the inverse of a RemovePart restores no note"
        );
    }

    #[test]
    fn the_inverse_of_a_staff_removal_rebuilds_the_staff_and_not_its_content() {
        let mut stage = stage();
        insert_note(&mut stage, 0, Step::C);
        let staff = stage.staff;
        let part = stage.part;

        let applied = stage
            .score
            .apply(ScoreCommand::RemoveStaff { staff })
            .expect("a RemoveStaff of a staff the score holds is accepted");
        undo_all(&mut stage.score, applied.inverse, "a RemoveStaff");

        assert_eq!(
            stage.score.staves().len(),
            1,
            "the inverse of a RemoveStaff rebuilds one staff"
        );
        assert!(
            !stage.score.staves().contains_key(&staff),
            "the rebuilt staff takes a NEW identifier, which escalation:T2-9 records as the open gap"
        );
        let rebuilt = stage
            .score
            .staves()
            .values()
            .next()
            .expect("the score holds the rebuilt staff");
        assert_eq!(
            rebuilt.clef(),
            Clef::Treble,
            "the rebuilt staff opens with the clef that the staff that stood opened with"
        );
        assert_eq!(
            rebuilt.voices().len(),
            1,
            "the rebuilt staff opens with one voice, because InsertNote names one"
        );
        assert_eq!(
            stage
                .score
                .parts()
                .get(&part)
                .expect("the part stands through a staff removal")
                .staves()
                .len(),
            1,
            "the part holds the rebuilt staff"
        );
        assert!(
            stage.score.notes().is_empty(),
            "the inverse of a RemoveStaff restores no note"
        );
    }

    #[test]
    fn the_inverse_of_a_measure_removal_rebuilds_the_measure_and_not_its_identifier() {
        let mut score = Score::new();
        let first = first_measure(&score);
        score
            .apply(ScoreCommand::InsertMeasures {
                after: first,
                count: TWO_MEASURES,
            })
            .expect("an InsertMeasures is accepted");
        let removed = second_measure_of(&score);
        let before = score.clone();

        let applied = score
            .apply(ScoreCommand::RemoveMeasures {
                from: removed,
                count: ONE_MEASURE,
            })
            .expect("a RemoveMeasures of one empty measure is accepted");
        assert_eq!(
            score.measures().len(),
            2,
            "the removal takes one measure of the three"
        );
        undo_all(&mut score, applied.inverse, "a RemoveMeasures");

        assert_eq!(
            score.measures().len(),
            before.measures().len(),
            "the inverse of a RemoveMeasures rebuilds one measure for each measure removed"
        );
        assert!(
            !score.measures().contains_key(&removed),
            "the rebuilt measure takes a NEW identifier, which escalation:T2-9 records as the open gap"
        );
        let starts: Vec<i64> = starts_of(&score);
        assert_eq!(
            starts,
            starts_of(&before),
            "every measure of the rebuilt grid stands where it stood"
        );
        assert_eq!(
            score.tempo_map(),
            before.tempo_map(),
            "the meter list comes back with the measure grid"
        );
    }

    #[test]
    fn an_added_part_takes_an_ordinal_that_no_part_of_its_voice_type_holds() {
        let mut score = Score::new();
        let first = part_of(
            &score
                .apply(add_part("Soprano 1"))
                .expect("AddPart is accepted"),
        );
        score
            .apply(add_part("Soprano 2"))
            .expect("AddPart is accepted");
        score
            .apply(ScoreCommand::RemovePart { part: first })
            .expect("a RemovePart is accepted");
        score
            .apply(add_part("Soprano 3"))
            .expect("AddPart is accepted");

        let mut ordinals: Vec<u16> = score
            .parts()
            .values()
            .map(|part| part.ordinal().get())
            .collect();
        ordinals.sort_unstable();
        assert_eq!(
            ordinals,
            vec![2, 3],
            "the ordinal is one above the greatest that the voice type carries, so a removal from the middle of the run leaves no two parts under one label"
        );
    }

    #[test]
    fn apply_increases_the_revision() {
        let mut stage = stage();
        let before = stage.score.revision().get();
        insert_note(&mut stage, 0, Step::C);
        let after_success = stage.score.revision().get();
        assert!(
            after_success > before,
            "a successful command increases the revision"
        );
        let refusal = stage.score.apply(ScoreCommand::InsertNote {
            staff: stage.staff,
            voice: stage.voice,
            onset: Ticks::new(0),
            pitch: natural(Step::E),
            duration: quarter(),
        });
        assert!(refusal.is_err(), "a second note at one onset is refused");
        assert_eq!(
            stage.score.revision().get(),
            after_success,
            "a refused command leaves the revision unchanged"
        );
    }

    #[test]
    fn copy_then_remove_is_a_cut() {
        let (mut stage, first, second) = stage_with_two_notes();
        let selection = Selection::Notes(vec![first, second]);
        let before = stage.score.clone();
        let clipboard = match stage.score.copy(&selection) {
            Ok(clipboard) => clipboard,
            Err(error) => panic!("a copy of two notes the score holds answered {error}"),
        };
        let copied: Vec<NoteId> = clipboard.notes().iter().map(Note::id).collect();
        assert_eq!(
            copied,
            vec![first, second],
            "the clipboard holds the selected notes"
        );
        assert_eq!(stage.score, before, "a copy leaves the score unchanged");
        let cut = stage.score.apply(ScoreCommand::Remove { selection });
        assert!(cut.is_ok(), "the remove half of a cut is accepted");
        assert!(
            stage.score.notes().is_empty(),
            "the remove half of a cut takes both notes out of the score"
        );
    }

    #[test]
    fn the_inverse_of_a_cross_staff_remove_restores_every_staff() {
        let mut stage = stage();
        let first = insert_note(&mut stage, 0, Step::C);
        let (lower_staff, lower_voice) = add_staff(&mut stage);
        let second = insert_note_in(&mut stage, lower_staff, lower_voice, 0, Step::G);
        let before = stage.score.clone();

        let applied = stage
            .score
            .apply(ScoreCommand::Remove {
                selection: Selection::Notes(vec![first, second]),
            })
            .expect("a Remove of two notes the score holds is accepted");
        assert!(
            stage.score.notes().is_empty(),
            "the remove takes the note of each staff out of the score"
        );
        assert_eq!(
            applied.inverse.len(),
            2,
            "the inverse holds one Paste for each (staff, voice) pair of the removed content"
        );

        undo_all(&mut stage.score, applied.inverse, "a cross-staff Remove");
        assert_same_content(&stage.score, &before, "a cross-staff Remove");
        let upper = stage
            .score
            .notes()
            .get(&first)
            .expect("the undo restores the note of the upper staff");
        assert_eq!(
            upper.staff(),
            stage.staff,
            "the note of the upper staff comes back into the upper staff"
        );
        let lower = stage
            .score
            .notes()
            .get(&second)
            .expect("the undo restores the note of the lower staff");
        assert_eq!(
            lower.staff(),
            lower_staff,
            "the note of the lower staff comes back into the lower staff, and not into the staff of the first Paste"
        );
    }

    #[test]
    fn set_voice_refuses_a_voice_that_the_staff_of_a_named_note_does_not_hold() {
        let mut stage = stage();
        let upper = insert_note(&mut stage, 0, Step::C);
        let (lower_staff, lower_voice) = add_staff(&mut stage);
        let lower = insert_note_in(&mut stage, lower_staff, lower_voice, 0, Step::G);
        let target = stage.voice;
        let before = stage.score.clone();

        let refusal = stage.score.apply(ScoreCommand::SetVoice {
            notes: vec![upper, lower],
            voice: target,
        });

        assert_eq!(
            refusal,
            Err(ScoreError::MissingStaff(lower_staff)),
            "a SetVoice checks the target voice against the staff of EVERY named note, and the lower staff does not hold the voice of the upper staff"
        );
        assert_eq!(
            stage.score, before,
            "the refused SetVoice leaves every note in the voice it held"
        );
    }

    #[test]
    fn set_voice_refuses_a_group_that_would_cover_one_tick_range() {
        let mut stage = stage();
        let upper = insert_note(&mut stage, 0, Step::C);
        let second_voice = add_voice(&mut stage);
        let staff = stage.staff;
        let lower = insert_note_in(&mut stage, staff, second_voice, 0, Step::G);
        let target = stage.voice;
        let before = stage.score.clone();

        let refusal = stage.score.apply(ScoreCommand::SetVoice {
            notes: vec![upper, lower],
            voice: target,
        });

        assert_eq!(
            refusal,
            Err(ScoreError::OverlappingNote {
                staff,
                voice: target,
                onset: Ticks::ZERO,
            }),
            "two notes of one staff that move into one voice at one onset cover one tick range, so the SetVoice refuses"
        );
        assert_eq!(
            stage.score, before,
            "the refused SetVoice leaves every note in the voice it held"
        );
    }

    #[test]
    fn set_part_refuses_a_group_that_would_cover_one_tick_range() {
        let mut stage = stage();
        let upper = insert_note(&mut stage, 0, Step::C);
        let (lower_staff, lower_voice) = add_staff(&mut stage);
        let lower = insert_note_in(&mut stage, lower_staff, lower_voice, 0, Step::G);
        let part = stage.part;
        let staff = stage.staff;
        let voice = stage.voice;
        let before = stage.score.clone();

        let refusal = stage.score.apply(ScoreCommand::SetPart {
            notes: vec![upper, lower],
            part,
            staff,
        });

        assert_eq!(
            refusal,
            Err(ScoreError::OverlappingNote {
                staff,
                voice,
                onset: Ticks::ZERO,
            }),
            "two notes of two staves that move into one staff at one onset cover one tick range, so the SetPart refuses"
        );
        assert_eq!(
            stage.score, before,
            "the refused SetPart leaves every note in the staff it held"
        );
    }

    #[test]
    fn a_move_into_one_staff_refuses_a_group_that_would_cover_one_tick_range() {
        let mut stage = stage();
        let upper = insert_note(&mut stage, 0, Step::C);
        let (lower_staff, lower_voice) = add_staff(&mut stage);
        let lower = insert_note_in(&mut stage, lower_staff, lower_voice, 0, Step::G);
        let staff = stage.staff;
        let voice = stage.voice;
        let before = stage.score.clone();

        let refusal = stage.score.apply(ScoreCommand::Move {
            selection: Selection::Notes(vec![upper, lower]),
            by: Ticks::ZERO,
            to_staff: Some(staff),
        });

        assert_eq!(
            refusal,
            Err(ScoreError::OverlappingNote {
                staff,
                voice,
                onset: Ticks::ZERO,
            }),
            "two notes of two staves that move into one staff at one onset cover one tick range, so the Move refuses"
        );
        assert_eq!(
            stage.score, before,
            "the refused Move leaves every note where it stood"
        );
    }

    #[test]
    fn a_paste_into_one_staff_refuses_a_clipboard_that_would_cover_one_tick_range() {
        let mut stage = stage();
        let upper = insert_note(&mut stage, 0, Step::C);
        let (lower_staff, lower_voice) = add_staff(&mut stage);
        let lower = insert_note_in(&mut stage, lower_staff, lower_voice, 0, Step::G);
        let staff = stage.staff;
        let voice = stage.voice;
        let selection = Selection::Notes(vec![upper, lower]);
        let clipboard = stage
            .score
            .copy(&selection)
            .expect("a copy of two notes the score holds is accepted");
        stage
            .score
            .apply(ScoreCommand::Remove { selection })
            .expect("the remove half of a cut is accepted");
        let before = stage.score.clone();

        let refusal = stage.score.apply(ScoreCommand::Paste {
            clipboard: Box::new(clipboard),
            at: Ticks::ZERO,
            staff,
            voice,
        });

        assert_eq!(
            refusal,
            Err(ScoreError::OverlappingNote {
                staff,
                voice,
                onset: Ticks::ZERO,
            }),
            "two clipboard notes that land in one staff and voice at one onset cover one tick range, so the Paste refuses"
        );
        assert_eq!(
            stage.score, before,
            "the refused Paste puts no element into the score"
        );
    }

    #[test]
    fn a_move_of_a_whole_group_is_accepted_where_the_group_vacates_the_target() {
        let (mut stage, first, second) = stage_with_two_notes();

        let moved = stage.score.apply(ScoreCommand::Move {
            selection: Selection::Notes(vec![first, second]),
            by: Ticks::new(QUARTER_TICKS),
            to_staff: None,
        });

        assert!(
            moved.is_ok(),
            "a moved element never collides with the earlier place of another moved element, and the answer was {moved:?}"
        );
        let onsets: Vec<i64> = [first, second]
            .iter()
            .map(|id| {
                stage
                    .score
                    .notes()
                    .get(id)
                    .expect("the move keeps both notes")
                    .onset()
                    .get()
            })
            .collect();
        assert_eq!(
            onsets,
            vec![QUARTER_TICKS, QUARTER_TICKS * 2],
            "each note of the moved group takes its own new onset"
        );
    }

    #[test]
    fn the_inverse_of_a_time_signature_change_restores_the_tempo_map() {
        let (mut score, second) = stage_with_two_measures();
        let before = score.clone();

        let applied = score
            .apply(ScoreCommand::SetTimeSignature {
                measure: second,
                meter: three_four(),
            })
            .expect("a SetTimeSignature on a measure the score holds is accepted");
        undo_all(
            &mut score,
            applied.inverse,
            "a SetTimeSignature on the second measure",
        );

        assert_eq!(
            score.tempo_map(),
            before.tempo_map(),
            "the inverse of a SetTimeSignature restores the tempo map the score held, and leaves no meter entry the score never carried"
        );
        assert_same_content(
            &score,
            &before,
            "a SetTimeSignature on the second measure inverts",
        );
    }

    #[test]
    fn a_time_signature_change_moves_the_bar_lines_and_no_content() {
        let mut stage = stage();
        let first = first_measure(&stage.score);
        stage
            .score
            .apply(ScoreCommand::InsertMeasures {
                after: first,
                count: TWO_MEASURES,
            })
            .expect("an InsertMeasures after the first measure of a new score is accepted");
        let held = note_of(
            &stage
                .score
                .apply(ScoreCommand::InsertNote {
                    staff: stage.staff,
                    voice: stage.voice,
                    onset: Ticks::new(QUARTER_TICKS * 2),
                    pitch: natural(Step::C),
                    duration: half(),
                })
                .expect("an InsertNote at a free onset is accepted"),
        );
        let before = stage.score.clone();
        assert_eq!(
            starts_of(&stage.score),
            vec![0, QUARTER_TICKS * 4, QUARTER_TICKS * 8],
            "three bars of four-four open at every fourth quarter note"
        );

        let applied = stage
            .score
            .apply(ScoreCommand::SetTimeSignature {
                measure: first,
                meter: three_four(),
            })
            .expect("a SetTimeSignature on a measure the score holds is accepted");

        assert_eq!(
            starts_of(&stage.score),
            vec![0, QUARTER_TICKS * 3, QUARTER_TICKS * 7],
            "the bar of three quarter notes moves every later bar line one quarter note earlier, and the grid stays one contiguous run"
        );
        let note = stage
            .score
            .notes()
            .get(&held)
            .expect("the meter change keeps the note");
        assert_eq!(
            (note.onset().get(), note.duration().ticks().get()),
            (QUARTER_TICKS * 2, QUARTER_TICKS * 2),
            "a note onset is absolute, so the meter change moves neither the onset nor the duration"
        );
        assert!(
            note.onset().get() < QUARTER_TICKS * 3
                && note.onset().saturating_add(note.duration().ticks()).get() > QUARTER_TICKS * 3,
            "the note now sounds across the bar line at three quarter notes, which it did not cross before"
        );
        let four_four = Meter::new(
            NonZeroU8::new(4).expect("four is not zero"),
            NoteValue::Quarter,
        );
        assert_eq!(
            stage
                .score
                .tempo_map()
                .meter_at(Ticks::ZERO)
                .expect("the map states a meter at tick zero")
                .meter(),
            three_four(),
            "the map states the new meter of the changed measure"
        );
        assert_eq!(
            stage
                .score
                .tempo_map()
                .meter_at(Ticks::new(QUARTER_TICKS * 3))
                .expect("the map states a meter at the start of the second measure")
                .meter(),
            four_four,
            "the map states the meter of the measure that follows at its new start tick, so the map and the measure list never disagree"
        );

        undo_all(
            &mut stage.score,
            applied.inverse,
            "a SetTimeSignature on the first of three measures",
        );
        assert_same_content(
            &stage.score,
            &before,
            "a SetTimeSignature on the first of three measures inverts",
        );
    }

    #[test]
    fn remove_measures_refuses_the_last_measure_of_the_score() {
        let mut score = Score::new();
        let only = first_measure(&score);
        let before = score.clone();

        let refusal = score.apply(ScoreCommand::RemoveMeasures {
            from: only,
            count: ONE_MEASURE,
        });

        assert_eq!(
            refusal,
            Err(ScoreError::MeasureNotEmpty(only)),
            "a score holds at least one measure, so a run that covers every measure is refused rather than accepted with an inverse that undoes nothing"
        );
        assert_eq!(
            score, before,
            "the refused RemoveMeasures leaves the measure grid as it stood"
        );
    }

    #[test]
    fn a_paste_refuses_a_spanner_whose_other_endpoint_the_score_lost() {
        let mut stage = stage();
        let held = insert_note(&mut stage, 0, Step::C);
        let other = insert_note(&mut stage, QUARTER_TICKS, Step::D);
        stage
            .score
            .apply(ScoreCommand::AddSpanner {
                kind: SpannerKind::Slur,
                from: held,
                to: other,
            })
            .expect("an AddSpanner over two notes the score holds is accepted");
        let clipboard = stage
            .score
            .copy(&Selection::Notes(vec![held]))
            .expect("a copy of one note the score holds is accepted");
        assert_eq!(
            clipboard.spanners().len(),
            1,
            "a copy of one endpoint carries the slur, and the other endpoint stays outside the clipboard"
        );
        for gone in [held, other] {
            stage
                .score
                .apply(ScoreCommand::Remove {
                    selection: Selection::Notes(vec![gone]),
                })
                .expect("a Remove of a note the score holds is accepted");
        }
        let before = stage.score.clone();

        let refusal = stage.score.apply(ScoreCommand::Paste {
            clipboard: Box::new(clipboard),
            at: Ticks::new(0),
            staff: stage.staff,
            voice: stage.voice,
        });

        assert_eq!(
            refusal,
            Err(ScoreError::MissingElement(ElementRef::Note(other))),
            "a paste whose spanner names a note that neither the clipboard nor the score holds is refused, because the paste would leave a spanner that names nothing"
        );
        assert_eq!(
            stage.score, before,
            "the refused Paste leaves the score as it stood"
        );
    }

    #[test]
    fn remove_measures_refuses_a_measure_that_a_longer_note_sounds_over() {
        let mut stage = stage();
        let first = first_measure(&stage.score);
        stage
            .score
            .apply(ScoreCommand::InsertMeasures {
                after: first,
                count: TWO_MEASURES,
            })
            .expect("an InsertMeasures after the first measure of a new score is accepted");
        stage
            .score
            .apply(ScoreCommand::InsertNote {
                staff: stage.staff,
                voice: stage.voice,
                onset: Ticks::new(QUARTER_TICKS * 3),
                pitch: natural(Step::C),
                duration: half(),
            })
            .expect("an InsertNote at a free onset is accepted");
        let second = second_measure_of(&stage.score);
        let before = stage.score.clone();

        let refusal = stage.score.apply(ScoreCommand::RemoveMeasures {
            from: second,
            count: ONE_MEASURE,
        });

        assert_eq!(
            refusal,
            Err(ScoreError::MeasureNotEmpty(second)),
            "a note that starts in one measure and sounds into the next fills the next one, because the emptiness check reads the half-open span that check_free reads and not the onset"
        );
        assert_eq!(
            stage.score, before,
            "the refused RemoveMeasures leaves the measure grid and the note as they stood"
        );
    }
}
