# Duet v1 product requirements

Author: Product Manager. Date: 2026-09-20. Plan: `roadmap/duet-v1`.

Sources read: the shared brief, `roadmap/duet-v1/research/gpui-kit-audit.md` (sections 4, 5, 9),
`roadmap/duet-v1/research/ardour-concepts.md` (sections 3, 4, 11), `roadmap/duet-v1/research/crate-survey.md`,
and the current application skeleton `crates/bc_app/duet/lang_rust/src/main.rs` and `crates/bc_app/duet/lang_rust/src/app.rs`.

Code check: the application today opens one window and draws one counter button. No score, no audio,
no timeline, and no menu exists. Every requirement below is new work. No requirement contradicts
existing behaviour, because there is no existing product behaviour to contradict.

Terms used once and then kept:
- **Project**: one piece of music with its score, its audio, its mix, and its history.
- **Part**: one vocal line in the score, such as Soprano.
- **Track**: one audio lane that belongs to one part. A part holds many tracks.
- **Take**: one record pass on one track.
- **Region**: a view of one immutable audio source, with a position, a start, and a length.
- **Load state**: what the user sees while work is in progress.
- **Empty state**: what the user sees when a surface holds no data.
- **Error state**: what the user sees when an operation fails.
- **Insert point**: the place in the score that receives the next note.
- **Agent**: a terminal artificial intelligence (AI) tool such as Claude Code or Codex.
- **CLI**: the `duet` command-line interface that ships with the application.

Every story carries an identifier, a priority (MUST, SHOULD, LATER), and a primary user group.
Section 8 repeats every priority in one table and gives the reason for every LATER.

---

## 1. Users

Duet has three user groups. Two are people. One is a program. All three are first-class.

### 1.1 The vocalist-composer (primary, daily user)

Mental model: task-oriented. "I hear a line. I want it on the staff, sung by me, mixed, and sent out today."

Profile:
- One person. Writes the piece, sings every part, mixes it, and exports it.
- Works alone, at a desk, with a microphone, headphones, and often a small MIDI keyboard.
- Returns to the same project many times over many days.

What this group needs:
- The shortest path from an idea to a sounded note. A click adds a note. No dialog stands in the way.
- The four modes must feel like one room with the furniture moved, not four applications.
- The application must hold every setting across a restart: the window, the zoom, the selected part, the armed track.
- Mistakes must cost seconds, not minutes. Undo covers every score edit and every audio edit.

What breaks this group:
- A modal dialog on the common path.
- A lost take after a device fault.
- A mode switch that discards the scroll position or the selection.

### 1.2 The choir director (power user)

Mental model: diagnostic and part-oriented. "Show me the alto line against the tenor line. Give me a rehearsal file per part."

Profile:
- Works with four to eight parts and many singers.
- Reads notation fluently. Wants density, not simple hints.
- Cares about the score as a document that other people read and sing from.

What this group needs:
- Per-part staves visible together, with the ability to hide a part without deletion.
- A part list that shows, per part, the track count, the mute state, and the solo state at a glance.
- Export per part, so that each singer gets one file.
- Correct notation: key, time, clef, lyrics under the right syllable, and a legible print of the score.

What breaks this group:
- A score that shows only two staves when the piece has six parts.
- Lyrics that break when the note count changes.
- A mix that cannot solo a whole part in one action.

### 1.3 The terminal AI agent (first-class user)

Mental model: file-first and idempotent. "Read the state, change the state, prove the change, commit it."

An agent has no eyes and no mouse. It reads text, writes text, runs commands, and reads exit codes.
Duet must therefore expose every product capability as text plus a command. The graphical interface
is one client of the core. It is never the owner of state.

The agent has exactly three means: the files in the project bundle, git, and the `duet` CLI.
Duet must not require a fourth.

#### 1.3.1 What the agent must do with the application closed

With no window open, and with only the project directory on disk, the agent must be able to:

1. Create a project and set its title, its parts, its key, its time signature, and its tempo.
2. Read the whole score as text. The stored score is a text file that the agent can diff.
3. Add a note, change a note, delete a note, add a part, and set a lyric syllable.
4. Import a MIDI file into a named part.
5. Set a mix parameter, such as the gain of one track.
6. Render the project to an audio file, with no audio hardware present.
7. Read the measured loudness report of that render.
8. Commit, read the history, show a diff between two commits, check out an earlier commit, and make a branch.
9. Read the project state as one status command: the parts, the tracks, the take count, and the commit.

Every one of those actions must succeed with no audio device, no MIDI device, and no display.
Every one must return a non-zero exit code on failure and a one-line reason a person can read.
Every write must be idempotent in effect: a repeated identical command must not create a second copy.

#### 1.3.2 What the agent must do with the application open

The window may be open while the agent works. The user may be in the middle of an edit.
With the application open, the agent must be able to:

1. Run the same verbs as in 1.3.1, with the same names and the same output.
2. See its change appear in the open window without a manual reload by the user.
3. Learn, before it writes, that the user has unsaved changes, and stop rather than overwrite them.
4. Ask the application for its live state, such as the transport position and the armed track.
5. Be blocked from a destructive verb while a record pass runs, with a clear reason.

The user must always see what the agent changed. A change that the agent makes must appear in the
undo history with the agent named as the author. The user must be able to undo it like any other change.

#### 1.3.3 Agent stories

**A-01 — Headless project edit (MUST)**
As an agent, I edit a project with the application closed, so that I work while the user is away.
- Given a closed project on disk, when the agent runs a score edit verb, then the CLI writes the change and exits zero.
- Given a project on disk, when the agent runs the same edit verb twice, then the second run makes no second change.
- Given an invalid pitch value, when the agent runs the edit verb, then the CLI exits non-zero and prints one reason.
- Given no audio device, when the agent runs the render verb, then the render completes and writes the audio file.

**A-02 — Live edit with the window open (MUST)**
As an agent, I edit a project while the user has it open, so that the user sees my work at once.
- Given an open window, when the agent commits a score edit, then the window shows the change within two seconds.
- Given unsaved user edits, when the agent runs a write verb, then the CLI exits non-zero and names the conflict.
- Given a record pass in progress, when the agent runs a write verb, then the CLI refuses and names the record pass.
- Given an agent edit that the window applied, when the user presses undo, then the application reverses the agent edit.

**A-03 — Text-readable score (MUST)**
As an agent, I read and diff the score as text, so that I reason about music without a viewer.
- Given any project, when the agent reads the score file, then the file is text and holds every note, key, time, and lyric.
- Given two commits, when the agent asks for a diff, then the output names each changed measure and part.
- Given a score the application wrote, when a MusicXML reader opens it, then the reader accepts it.

**A-04 — Status in one call (MUST)**
As an agent, I read the whole project state in one command, so that I plan my next action.
- Given a project, when the agent runs the status verb, then the output lists the parts, the tracks, the takes, and the commit.
- Given an open window, when the agent runs the status verb, then the output also gives the transport state and the armed track.

**A-05 — MIDI file import (MUST)**
As an agent, I import a MIDI file into a part, so that I bring material from another tool.
- Given a standard MIDI file and a named part, when the agent runs the import verb, then the notes appear in that part.
- Given a MIDI file with several channels, when the agent imports it, then the CLI reports which channel went to which part.
- Given a file that is not a MIDI file, when the agent imports it, then the CLI exits non-zero and names the fault.

---

## 2. Compose mode

Primary users: the vocalist-composer and the choir director.

Compose mode is the front door. A first-time user meets it before any other surface. It must teach
the first action without a manual, and it must stay fast for the daily user who knows it well.

### 2.1 Project open and create

**C-01 — Start surface (MUST)**
As a first-time user, I open Duet and know what to do first, so that I do not stall.
- Given no open project, when the application starts, then the window shows a create action and a recent project list.
- Given no recent project, when the application starts, then the window shows one create action and one open action.
- Given a recent project row, when the user clicks it, then the application opens that project.
- Given a recent project that no longer exists, when the user clicks it, then the row shows a not-found message.
- Given a not-found row, when the user asks, then the application removes that row from the list.

**C-02 — Create a project (MUST)**
As a vocalist-composer, I create a project in a few seconds, so that I capture an idea.
- Given the start surface, when the user starts a create action, then the application asks for a title and a part set.
- Given the create form, when the user opens it, then the form also asks for a key, a time signature, and a tempo.
- Given the create form, when the user accepts the defaults, then the application creates a project with one treble staff.
- Given a create action, when the application creates the project, then it writes the project bundle to disk and makes the first commit.
- Given a create action on a full disk, when the write fails, then the application keeps the form open and shows the reason.

**C-03 — Open a project (MUST)**
As a returned user, I open a project and find it as I left it, so that I continue work.
- Given a saved project, when the user opens it, then the application restores the mode, the zoom, and the scroll position.
- Given a project that another process changed on disk, when the user opens it, then the application loads the current file content.
- Given a damaged state file, when the user opens the project, then the application offers the last good commit.
- Given a damaged state file, when the application detects it, then the application names the fault in plain words.
- Given a large project, when the application loads it, then the window shows the score outline within one second and fills detail after.

**C-04 — Save and dirty state (MUST)**
As a user, I always know if my work is safe, so that I never guess.
- Given an unsaved change, when the user looks at the title bar, then the title bar marks the project as changed.
- Given an unsaved change, when the user presses the save shortcut, then the application writes the project and clears the mark.
- Given an unsaved change, when the user closes the window, then the application asks to save, to discard, or to cancel.
- Given no change for thirty seconds, when the timer fires, then the application saves the working state without a commit.

### 2.2 Staff display

**C-05 — Treble, bass, and grand staff (MUST)**
As a vocalist-composer, I see the right staff for the voice I write, so that I read the music correctly.
- Given a part with a treble clef, when the score draws, then that part shows one five-line staff with a treble clef.
- Given a part with a bass clef, when the score draws, then that part shows one five-line staff with a bass clef.
- Given a piano part, when the score draws, then that part shows a grand staff with a brace and two clefs.
- Given a note above or below the staff, when the score draws, then the application draws the correct ledger lines.

**C-06 — Per-part staves (MUST)**
As a choir director, I see every part at once, so that I compare the lines.
- Given four parts, when the score draws, then each part gets its own staff in one system, in part order.
- Given a part name, when the score draws the first system, then the application shows that name at the left.
- Given eight parts and a narrow window, when the score draws, then the system fits the width and the user scrolls down.
- Given a part, when the user hides it, then the staff disappears from every system and the notes stay in the project.

**C-07 — Part management (MUST)**
As a choir director, I add and order parts, so that the score matches my ensemble.
- Given a project, when the user adds a part, then the application asks for a name, a clef, and a voice range.
- Given the part list, when the user reorders a part, then every system redraws in the new order.
- Given a part with notes, when the user deletes it, then the application asks to confirm and names the note count.
- Given a deleted part, when the user presses undo, then the part and its notes return.

### 2.3 Note entry

**C-08 — Note entry by click with the default quarter note (MUST)**
As a vocalist-composer, I click on a staff and get a note, so that I write with one hand.
- Given Compose mode and an empty measure, when the user clicks a staff, then the application adds a quarter note at that pitch.
- Given a click between two staff lines, when the application adds the note, then the pitch is the nearest line or space.
- Given a pointer over a staff, when the user moves it, then the application shows a faint preview note at the target.
- Given a click, when the application adds the note, then the window shows the new note within 100 milliseconds.
- Given a full measure, when the user clicks in it, then the application adds the note in the next measure.
- Given a full measure, when the application adds a note, then the measure never holds more beats than the time signature allows.

**C-09 — Held duration keys (MUST)**
As a vocalist-composer, I hold a key and click, so that I choose a duration without a menu.
- Given the user holds `w`, when the user clicks on a staff, then the application adds a whole note.
- Given the user holds `h`, when the user clicks on a staff, then the application adds a half note.
- Given the user holds `e`, when the user clicks on a staff, then the application adds an eighth note.
- Given the user holds a duration key, when the application knows the key is down, then the top bar shows the pending duration.
- Given a held duration key, when the window loses focus, then the application clears every held key and uses the quarter note.
- Given the user holds two duration keys, when the user clicks, then the application uses the key that went down last.

UX note for the Architect. The GPUI Kit audit, section 4, records that no interface reports the set of
held letter keys. The staff view must track key down and key up itself. Two user-visible rules follow,
and both are acceptance criteria above: the pending duration must be visible before the click, and a
lost focus must clear the held state. A stuck key that silently adds whole notes is a Critical defect.

**C-10 — Note entry from a MIDI keyboard with no setup (MUST)**
As a vocalist-composer, I plug in a keyboard and play, so that I enter notes at speed.
- Given no prior configuration, when the user connects a MIDI keyboard, then the application opens it within two seconds and shows its name.
- Given a connected keyboard and an insert point, when the user presses a key, then the application writes that pitch there.
- Given a note from the keyboard, when the application writes it, then the insert point advances by the current duration.
- Given a connected keyboard, when the user presses a key, then the application sounds the pitch within 20 milliseconds.
- Given several pressed keys held together, when the user releases them, then the application writes one chord.
- Given a disconnected keyboard, when the device goes away, then the application shows a quiet notice and keeps every entered note.
- Given two connected keyboards, when the user plays either one, then the application accepts notes from both.

**C-11 — Insert point and step entry (MUST)**
As a user, I control where the next note lands, so that entry is predictable.
- Given a score, when the user clicks an empty beat, then the application sets the insert point there and marks it.
- Given an insert point, when the user presses the right arrow key, then the insert point advances by the current duration.
- Given an insert point at the end of the score, when the user enters a note, then the application adds a measure.
- Given an insert point, when the user presses the rest key, then the application writes a rest of the current duration and advances.

**C-12 — MIDI file import in the user interface (SHOULD)**
As a vocalist-composer, I import a MIDI file, so that I start from an existing sketch.
- Given a MIDI file, when the user imports it, then the application shows a per-track preview with a part target for each track.
- Given the preview, when the user accepts it, then the application writes the notes into the named parts in one undo step.
- Given a very large MIDI file, when the import runs, then the window stays responsive and shows progress.

### 2.4 Note edits

**C-13 — Note context menu (MUST)**
As a user, I right-click a note and change it, so that every edit is near the note.
- Given a note, when the user right-clicks it, then a menu offers pitch, duration, accidental, tie, dot, voice, and delete.
- Given the pitch item, when the user picks a pitch, then the note moves to that pitch and the score reflows.
- Given the duration item, when the user picks a duration, then the note takes that duration and the later notes shift.
- Given the accidental item, when the user picks sharp, flat, or natural, then the application shows that accidental and sounds that pitch.
- Given two notes of one pitch in sequence, when the user applies a tie, then the application sounds one longer note.
- Given a note, when the user applies a dot, then the duration grows by half and the measure count stays correct.
- Given a note, when the user sets a voice, then the application draws the stem in the direction of that voice.
- Given a note, when the user picks delete, then the application removes the note and leaves a rest of the same length.
- Given any context menu action, when the user presses Escape, then the menu closes and nothing changes.

**C-14 — Direct note edits by keyboard and drag (SHOULD)**
As a daily user, I change a note without a menu, so that the edit costs one action.
- Given a selected note, when the user presses the up arrow key, then the pitch rises by one step.
- Given a selected note, when the user drags it up or down, then the pitch follows the pointer.
- Given a dragged note, when the user releases outside the staff, then the note returns to its first pitch.

### 2.5 Notation options

**C-15 — Top bar of notation options (MUST)**
As a user, I reach every notation option from one bar, so that I do not hunt through menus.
- Given Compose mode, when the window draws, then a top bar shows clef, key, time, tempo, dynamics, articulation, and lyrics.
- Given the top bar, when the user looks at it, then each control shows the value that applies at the insert point.
- Given a selection, when the user picks a top bar value, then the application applies it to the selection.
- Given no selection, when the user picks a top bar value, then the application applies it from the insert point forward.
- Given a narrow window, when the top bar does not fit, then the application groups the extra controls behind one overflow button.

**C-16 — Clef, key, and time changes (MUST)**
As a choir director, I change the clef, the key, or the time mid-piece, so that the score is correct.
- Given a measure, when the user sets a new key, then the application draws the key change at that measure.
- Given a key change, when the application draws it, then the application respells the accidentals after it.
- Given a measure, when the user sets a new time signature, then the application draws the change and rebars the later music.
- Given a part, when the user sets a new clef, then the application draws the clef change and keeps every pitch.
- Given a key change, when the user presses undo, then the application restores the old key and the old accidentals.

**C-17 — Tempo (MUST)**
As a user, I set the tempo, so that the playback matches the piece.
- Given a project, when the user sets a tempo at measure one, then the playback uses that tempo.
- Given a measure, when the user adds a tempo change, then the application draws the marking and the playback follows it.
- Given a tempo change, when the audio timeline draws, then the audio stays at its recorded sample position and does not move.

**C-18 — Dynamics and articulation (MUST)**
As a user, I mark dynamics and articulation, so that the score reads as music.
- Given a selected note, when the user picks a dynamic, then the application draws the mark below the staff.
- Given a selected note, when the user picks an articulation, then the application draws the mark beside the note head.
- Given a dynamic mark, when the user plays the score, then the built-in voice changes level at that mark.
- Given a range selection, when the user adds a crescendo, then the application draws one hairpin over the range.

**C-19 — Lyrics (MUST)**
As a choir director, I type lyrics under the notes, so that the singers read words and music together.
- Given a selected note, when the user starts lyric entry, then the application places a text caret under that note.
- Given lyric entry, when the user types a syllable and presses the space key, then the application moves to the next note.
- Given lyric entry, when the user types a hyphen, then the application draws a hyphen between the two syllables.
- Given a lyric line, when the user changes a note duration, then the syllables stay attached to their notes.
- Given lyric entry, when the user presses Escape, then the application ends lyric entry and keeps the typed text.
- Given two lyric verses, when the user adds the second verse, then the application draws it on a second line under the first.

### 2.6 Selection, undo, and playback

**C-20 — Selection (MUST)**
As a user, I select what I want to change, so that an edit hits the right notes.
- Given a note, when the user clicks it, then the application selects that note alone and marks it.
- Given a selected note, when the user shift-clicks a later note, then the application selects the range between them.
- Given a selected note, when the user control-clicks or command-clicks another note, then the application adds that note to the selection.
- Given a staff, when the user drags a box over notes, then the application selects every note inside the box.
- Given a selection, when the user presses Escape, then the application clears the selection.
- Given a selection of 500 notes, when the user applies one edit, then the application applies it as one undo step.

**C-21 — Undo and redo (MUST)**
As a user, I reverse any mistake, so that I explore without fear.
- Given any score edit, when the user presses undo, then the application reverses that edit and restores the selection.
- Given an undone edit, when the user presses redo, then the application applies it again.
- Given twenty edits, when the user presses undo twenty times, then the application returns to the first state.
- Given an undo, when the application reverses the edit, then the scroll position does not jump.
- Given a mode switch, when the user presses undo, then the application reverses the last edit and does not reverse the mode switch.

**C-22 — Playback with a built-in voice (MUST)**
As a vocalist-composer, I hear my score at once, so that I judge the music before I sing it.
- Given a score, when the user presses the play key, then the application sounds every part with a built-in voice.
- Given no extra download and no setup, when the user first presses play, then the built-in voice sounds.
- Given playback, when the transport runs, then a playhead moves over the score in time with the sound.
- Given playback, when the playhead reaches the end of a system, then the view follows to the next system.
- Given playback, when the user presses the play key again, then the transport stops and the playhead stays.
- Given playback, when the user clicks a note, then the transport continues and the click does not add a note.
- Given a part, when the user mutes or solos it, then the playback follows within one beat.

---

## 3. Record mode

Primary users: the vocalist-composer, then the choir director.

Record mode is the moment the user commits a performance. Every requirement here protects the take.
A lost take is the worst outcome this product can produce.

### 3.1 Mode and layout

**R-01 — Mode transition (MUST)**
As a vocalist-composer, I move from Compose to Record and keep my place, so that I sing what I just wrote.
- Given Compose mode at measure 30, when the user switches to Record mode, then the view stays at measure 30.
- Given Record mode, when the window draws, then one audio lane appears under each staff line of each part.
- Given Record mode, when the window draws, then the audio lane uses the same line breaks as the staff above it.
- Given a mode switch, when the application redraws, then the switch completes within 200 milliseconds.
- Given Record mode, when the user switches back to Compose, then the audio lanes collapse and the notes stay in place.

**R-02 — Part and track selection (MUST)**
As a choir director, I pick the part I record, so that the take lands in the right place.
- Given Record mode, when the window draws, then a part list shows Soprano, Alto, Tenor, and Bass with their track counts.
- Given a part, when the user adds a track, then the application creates an empty track under that part.
- Given a new track, when the application names it, then the name holds the part name and a number.
- Given a part with five tracks, when the window draws, then each track shows its own lane under that part staff.
- Given a track, when the user renames it, then the new name appears in the lane header and in the mix strip.
- Given a track with takes, when the user deletes it, then the application asks to confirm and names the take count.

**R-03 — Show one track or all tracks (MUST)**
As a user, I control how many lanes I see, so that the window stays readable.
- Given a part with many tracks, when the user picks the single-track view, then the lane shows only the selected track.
- Given the single-track view, when the user picks the all-track view, then every track of that part shows one lane each.
- Given the all-track view and eight tracks, when the window draws, then the lanes shrink in height and each keeps a readable waveform.
- Given a view choice, when the user restarts the application, then the application restores that view choice.

### 3.2 Input, arm, and monitor

**R-04 — Input selection and monitor (MUST)**
As a vocalist-composer, I pick my microphone and hear myself, so that I sing in tune.
- Given a track, when the user opens the input control, then the application lists every available input channel by name.
- Given no chosen input, when the user arms a track, then the application selects the default system input and names it.
- Given an armed track with the monitor on, when the user sings, then the user hears the input through the output.
- Given an armed track, when the user turns the monitor off, then the input no longer sounds and the level meter still moves.
- Given an input with no signal for ten seconds while armed, then the application shows a quiet hint that names the input.

**R-05 — Arm a track (MUST)**
As a user, I arm exactly the tracks I want, so that I do not record over the wrong part.
- Given a track, when the user clicks the arm control, then the control shows the armed state clearly.
- Given an armed track, when the user starts the transport, then the application records only the armed tracks.
- Given no armed track, when the user presses record, then the application refuses and names the missing arm.
- Given several armed tracks, when the user records, then the application writes one take per armed track.

**R-06 — Count-in (MUST)**
As a vocalist-composer, I get a count-in, so that I enter on the beat.
- Given a count-in of one bar, when the user presses record, then the application sounds one bar of clicks before it records.
- Given a count-in, when the clicks sound, then the window shows the remaining beats.
- Given a count-in, when the user presses stop during it, then the application cancels the record pass and writes no take.
- Given a count-in setting, when the user changes it to zero, then the record pass starts at once.

### 3.3 Record

**R-07 — Record a layered take (MUST)**
As a vocalist-composer, I record a take without loss of the last one, so that I compare them.
- Given an armed track with one take, when the user records again, then the new take sits above the old one.
- Given a layered record pass, when it ends, then the old take stays in the take list.
- Given a record pass, when the audio arrives, then the lane draws the waveform as it records.
- Given a record pass, when the transport runs, then the window stays responsive and the playhead stays smooth.
- Given a record pass, when the user presses stop, then the application writes the take and makes it selectable within one second.
- Given a finished record pass, when the user presses undo, then the application removes the take and keeps the audio file.

**R-08 — Overwrite record (MUST)**
As a user, I replace a bad section, so that I fix one phrase and keep the rest.
- Given the overwrite mode and a track with audio, when the user records over a range, then the new audio replaces that range.
- Given an overwrite pass, when it ends, then the audio outside the range is unchanged.
- Given an overwrite pass, when the user presses undo, then the replaced audio returns.
- Given the overwrite mode, when the window draws, then the record control names the mode in plain words.

**R-09 — Layered takes and take selection (MUST)**
As a vocalist-composer, I pick the best take, so that the part uses my best performance.
- Given a track with four takes, when the user opens the list, then each row shows a number, a length, and a date.
- Given the take list, when the user picks a take, then the track plays that take at once.
- Given a take, when the user renames it, then the new name shows in the list and in the lane.
- Given a selected take, when the user restarts the application, then the same take stays selected.
- Given a take, when the user deletes it, then the application asks to confirm and keeps the audio file on disk.

**R-10 — Double a part (SHOULD)**
As a choir director, I stack two passes of one part, so that the part sounds fuller.
- Given the double mode, when the user records over existing audio, then both passes sound together.
- Given a doubled part, when the user opens the take list, then both passes appear and each one can be muted.

### 3.4 Audio edit

**R-11 — Cut, splice, and trim (MUST)**
As a user, I cut a take into pieces and join them, so that I build one good performance.
- Given a region and a playhead inside it, when the user cuts, then the application makes two regions from one source.
- Given a cut, when the application makes the two regions, then the audio file on disk does not change.
- Given two next-to-each-other regions, when the user splices them, then the application joins them with no click at the joint.
- Given a region edge, when the user drags it inward, then the region gets shorter and the audio file stays whole.
- Given a trimmed region, when the user drags the edge outward again, then the removed audio returns.
- Given any cut, splice, or trim, when the user presses undo, then the application restores the region to its last state.
- Given a cut at a point with signal, when the application plays the joint, then a short fade runs and no click sounds.

**R-12 — Region move and snap (SHOULD)**
As a user, I move a region in time, so that the audio lines up with the notes.
- Given a region, when the user drags it, then the region follows the pointer and shows its new start.
- Given snap on, when the user drops a region near a beat, then the region starts on that beat.
- Given snap off, when the user drops a region, then the region starts at the exact pointer position.

### 3.5 Faults

**R-13 — The audio device disappears (MUST)**
As a vocalist-composer, I never lose a take when my device unplugs, so that I trust the application.
- Given a record pass, when the audio device disappears, then the application stops the transport and keeps every sample it wrote.
- Given a lost device, when the application stops, then a banner names the device and offers one retry action.
- Given a lost device, when the user reconnects it, then the application detects it and the banner offers to arm again.
- Given a lost device, when the application shows the banner, then the window stays usable and the score stays editable.
- Given a lost device, when the take ends early, then the take list shows the take with its true length.
- Given a lost device during playback only, when the device goes away, then the application stops the transport and shows the same banner.

**R-14 — Disk and performance warnings (SHOULD)**
> Correction note (2026-09-21, architecture B126, critic N22I-5 and N22I-6): v1 meets the free-space threshold (B126, 1 GB) and the drop report; the per-drop time list is deferred as accepted debt, filed at B126.
As a user, I learn about a risk before it costs me a take, so that I act in time.
- Given less than one gigabyte of free space, when the user arms a track, then the application warns the user.
- Given the free space warning, when it shows, then the warning names the free space in gigabytes.
- Given a dropped audio buffer in a record pass, when the pass ends, then the application marks the take.
- Given a marked take, when the user opens it, then the application names the time of each drop.

**R-15 — Pitch track overlay (SHOULD)**
As a singer, I see the pitch I sang against the note I was meant to sing, so that I judge a take by eye.
- Given a finished take, when the analysis completes, then the audio lane shows a pitch line over the waveform.
- Given a pitch line, when a sung pitch is more than 25 cents from the notated pitch, then that span shows in the warning color.
- Given a take still in progress, when the user looks at the lane, then the lane shows the waveform without a pitch line.
- Given the pitch overlay, when the user hides it with one toggle, then the lane shows the waveform only and the toggle state persists.
- Given a take with no clear pitch (noise or silence), when the analysis runs, then the lane shows no line for that span and no error.

---

## 4. Mix mode

Primary users: the vocalist-composer, then the choir director.

Mix mode drops the notation and gives the user one view of level, tone, and space.

**M-01 — Mode transition (MUST)**
As a vocalist-composer, I move to Mix and see my whole balance, so that I judge the sound.
- Given Record mode, when the user switches to Mix, then the notation collapses and strips appear for tracks and buses.
- Given Mix mode, when the window draws, then the timeline is linear and does not wrap.
- Given Mix mode, when the user switches back, then the notation returns at the same measure.
- Given a mode switch, when the transport runs, then the sound does not break and the playhead does not jump.

**M-02 — Track strips and part bus strips (MUST)**
As a choir director, I balance a part in one action, so that I do not move eight faders.
- Given four parts with many tracks, when Mix mode draws, then each track has one strip and each part has one bus strip.
- Given a part bus fader, when the user moves it, then every track of that part changes level together.
- Given a strip, when the window is narrow, then the application scrolls the strips sideways and keeps the master strip in view.
- Given a strip, when the user looks at it, then the strip shows the name, the fader, the pan, and a meter.
- Given a strip, when the user looks at it, then the strip also shows the part, the mute, and the solo.

**M-03 — The vocal tool set (MUST)**
As a vocalist-composer, I shape a vocal track with the tools a voice needs, so that I do not hunt for a plugin.
- Given a track strip, when the user opens the tool set, then it offers gain, high-pass, de-esser, compressor, and equalizer.
- Given the same tool set, when the user opens it, then it also offers reverb send, delay send, pan, mute, and solo.
- Given a tool, when the user changes a value, then the sound changes within one audio buffer.
- Given a changed value, when the application applies it, then the strip shows the new value.
- Given a tool, when the user turns it off, then the tool stops its effect and keeps its settings.
- Given a compressor, when audio passes through it, then the strip shows the gain reduction as a live meter.
- Given a de-esser, when the user sets its frequency, then the application shows which band it acts on.
- Given an equalizer, when the user drags a band, then the application draws the curve as it moves.
- Given a tool value, when the user double-clicks the control, then the value returns to its default.
- Given any tool change, when the user presses undo, then the application restores the last value.

**M-04 — Reverb and delay buses (MUST)**
As a user, I send several tracks to one reverb, so that the parts sit in one space.
- Given a project, when Mix mode first draws, then one reverb bus and one delay bus already exist.
- Given a track, when the user raises the reverb send, then that track sounds in the shared reverb.
- Given a bus, when the user changes its settings, then every sent track follows.
- Given a bus, when the user mutes it, then no track sounds through it and the sends keep their values.

**M-05 — Metering (MUST)**
As a user, I read the level at a glance, so that I know if the mix is too loud or too soft.
- Given audio on a track, when it plays, then the strip meter shows peak and root mean square (RMS) level.
- Given a peak above the ceiling, when it occurs, then the meter holds a clear over mark until the user clears it.
- Given the master strip, when audio plays, then the application shows the loudness in loudness units relative to full scale (LUFS).
- Given a meter, when the transport stops, then the meter falls to silence and keeps the peak mark.

**M-06 — Solo and mute (MUST)**
As a user, I listen to one part alone, so that I find a problem.
- Given a soloed track, when the transport plays, then only that track and its buses sound.
- Given a soloed part bus, when the transport plays, then every track of that part sounds and no other part sounds.
- Given any solo in the project, when the window draws, then a clear indicator shows that a solo is active.
- Given an active solo, when the user clicks the clear-solo indicator, then every solo turns off.

**M-07 — Automation (SHOULD)**
As a vocalist-composer, I change a level over time, so that one quiet phrase comes forward.
- Given a track parameter, when the user opens its automation lane, then the lane shows a line over the timeline.
- Given an automation lane, when the user adds a point, then the parameter follows the line during playback.
- Given automation in write mode, when the user moves the fader during playback, then the application records the movement.
- Given automation in read mode, when the user moves the fader, then the fader returns to the line.
- Given an automation edit, when the user presses undo, then the application restores the last curve.

**M-08 — A and B compare (LATER)**
As a user, I compare two settings of one tool, so that I choose with my ears.
- Given a tool with two stored setting sets, when the user switches between them, then the sound changes and no other setting moves.

---

## 5. Master mode

Primary user: the vocalist-composer.

Master mode makes one finished file that meets a stated loudness target.

**MA-01 — Loudness target and true-peak ceiling (MUST)**
As a vocalist-composer, I set a loudness target, so that my file matches the place I send it.
- Given Master mode, when the window draws, then the application shows a loudness target in LUFS and a true-peak ceiling in decibels.
- Given a target, when the user picks a preset, then the application sets the target and the ceiling together.
- Given a target, when the user plays the project, then the application shows the current loudness against the target.
- Given a true-peak ceiling, when the master output would go above it, then the master limiter holds the output below the ceiling.

**MA-02 — Master chain (MUST)**
As a user, I apply the final tone and level, so that the piece sounds finished.
- Given Master mode, when the window draws, then the master chain shows an equalizer, a compressor, and a limiter in that order.
- Given a master tool, when the user changes a value, then the sound changes and the loudness value updates.
- Given the master chain, when the user turns off one tool, then the sound skips that tool only.
- Given any master change, when the user presses undo, then the application restores the last value.

**MA-03 — Measured report (MUST)**
As a user, I read the measured loudness of my file, so that I prove that it meets the target.
- Given a finished project, when the user runs the measure action, then the application reports integrated loudness, loudness range, and true peak.
- Given a measure action, when it runs, then the window stays responsive and shows progress.
- Given a report, when the measured loudness misses the target by more than one LUFS, then the application says so in plain words.
- Given a measure action, when the user cancels it, then the application stops the work and keeps the last report.

**MA-04 — Export (MUST)**
As a vocalist-composer, I export a file, so that I send my music out.
- Given a finished project, when the user exports, then the application offers WAV, RF64, and FLAC.
- Given an export over four gigabytes, when the user picks WAV, then the application advises RF64 and names the reason.
- Given an export, when the application writes the file, then the window shows progress and a cancel action.
- Given a cancelled export, when the user cancels, then the application deletes the part file and says so.
- Given a finished export, when the file is written, then the application writes a report file beside it with the measured loudness.
- Given a finished export, when the user asks, then the application shows the file in the system file browser.
- Given a folder with no write permission, when the export fails, then the application keeps the settings and names the fault.

**MA-05 — Per-part export (SHOULD)**
As a choir director, I export one file per part, so that each singer rehearses alone.
- Given four parts, when the user exports per part, then the application writes one file per part with the part name.
- Given a per-part export, when it runs, then the application shows which part it writes now.

**MA-06 — Sample rate and bit depth choice (SHOULD)**
As a user, I pick the file format detail, so that the file matches the target system.
- Given an export, when the user opens the format control, then the application offers the sample rate and the bit depth.
- Given a sample rate that differs from the project rate, when the user exports, then the report names the conversion.

---

## 6. History

Primary users: all three groups. The agent depends on this section more than any other.

Duet stores the project as a git repository. The score and the session state are text. Audio sources
are immutable files that the project adds once and never rewrites, so history growth stays bounded.

**H-01 — Commit from the user interface (MUST)**
As a vocalist-composer, I save a named version, so that I can return to it.
- Given changes in the working project, when the user opens the commit surface, then the application lists what changed in plain words.
- Given the commit surface, when the user types a message and confirms, then the application makes one commit.
- Given no change, when the user opens the commit surface, then the application says that nothing changed and offers no commit.
- Given a commit, when it completes, then the title bar shows the project as saved and names the commit.
- Given a long commit of large audio, when it runs, then the window stays responsive and shows progress.

**H-02 — Browse the history (MUST)**
As a user, I read the history of my project, so that I find the version I want.
- Given a project with commits, when the user opens the history, then each row shows a message, an author, and a date.
- Given a commit row, when the user selects it, then the application shows what that commit changed, by part and by measure.
- Given a commit made by an agent, when the list draws, then the row names the agent as the author.
- Given a project with only the first commit, when the user opens the history, then the surface says one version exists.

**H-03 — Check out an earlier commit (MUST)**
As a user, I return to an earlier version, so that I recover work I preferred.
- Given a commit, when the user checks it out, then the application loads that version of the score, the session, and the mix.
- Given uncommitted changes, when the user checks out a commit, then the application asks to commit, to discard, or to cancel.
- Given a checked-out earlier commit, when the window draws, then a clear banner says that the project is at an earlier version.
- Given an earlier version, when the user returns to the latest version, then the application restores it in one action.
- Given a checkout, when audio files differ between versions, then the application keeps every audio file on disk.

**H-04 — Branch (MUST)**
As a vocalist-composer, I try a different arrangement without loss of the first, so that I compare two ideas.
- Given a project, when the user makes a branch, then the application asks for a name and switches to it.
- Given two branches, when the user switches between them, then the score, the takes, and the mix follow the branch.
- Given a branch, when the window draws, then the title bar shows the branch name.
- Given uncommitted work on a branch, when the user switches away, then the application asks to commit, to discard, or to cancel.
- Given a branch, when the user deletes it, then the application asks to confirm and names the commits that only that branch holds.

**H-05 — Merge a branch (LATER)**
As a user, I join two arrangements, so that I keep the best of both.
- Given two branches that changed different parts, when the user merges, then the application joins both sets of changes.
- Given two branches that changed one measure, when the user merges, then the application shows both versions and asks the user.

**H-06 — Agent history control (MUST)**
As an agent, I commit, branch, and check out from the terminal, so that I keep a record of my work.
- Given a project, when the agent runs the commit verb with a message, then the CLI makes one commit and prints its identifier.
- Given a project, when the agent runs the history verb, then each printed line holds an identifier, an author, and a message.
- Given a commit identifier, when the agent runs the checkout verb, then the CLI loads that version and prints the new state.
- Given uncommitted user changes, when the agent runs the checkout verb, then the CLI exits non-zero and names the changes.
- Given a branch name, when the agent runs the branch verb, then the CLI makes the branch and prints its name.
- Given an open window, when the agent commits, then the window history surface shows the new commit within two seconds.
- Given any agent commit, when the CLI writes it, then the commit author names the agent and not the user.

---

## 7. Cross-cutting requirements

### 7.1 The unified timeline

**X-01 — One timeline for notes and audio (MUST)**
As a user, I see my notes and my audio on one ruler, so that I trust what lines up.
- Given a project, when the window draws a timeline, then one ruler shows bars and beats and, on request, minutes and seconds.
- Given a tempo change, when the user adds it, then the notes move with the beats and the audio keeps its samples.
- Given a region and a note at the same beat, when the user looks at them, then they align on the screen.
- Given the ruler, when the user clicks it, then the playhead moves to that position in every mode.

**X-02 — The playhead (MUST)**
As a user, I always know where the sound is, so that I follow the music.
- Given playback, when the transport runs, then the playhead moves smoothly and does not stutter.
- Given playback in Compose or Record, when the playhead reaches the end of a system, then the view moves to the next system.
- Given playback in Mix or Master, when the playhead reaches the right edge, then the view scrolls and keeps the playhead in view.
- Given a stopped transport, when the user drags the playhead, then the ruler shows the new bar and beat.
- Given playback, when the user scrolls away by hand, then the view stops and waits for the user to ask again.

**X-03 — Zoom (MUST)**
As a user, I zoom in and out, so that I see the whole piece or one note.
- Given any mode, when the user zooms, then the content grows or shrinks around the playhead or the pointer.
- Given a zoom change, when the view redraws, then the redraw completes within 100 milliseconds.
- Given a zoom level, when the user leaves a mode and returns, then the application restores that zoom level.
- Given a zoom-to-fit action, when the user runs it, then the whole project fits the window width.

**X-04 — Wrapped timeline in Compose and Record (MUST)**
As a user, I read my music as lines on a page, so that the layout matches how I read music.
- Given Compose or Record mode, when the score draws, then the music breaks into systems that fit the window width.
- Given Record mode, when a system breaks, then each audio lane breaks at the same bar as the staff above it.
- Given a window resize, when the width changes, then the application rebreaks the systems and keeps the playhead in view.
- Given a region that crosses a line break, when the lane draws, then the application draws both halves and marks the join.

**X-05 — Linear timeline in Mix and Master (MUST)**
As a user, I see the whole project on one line, so that I judge the shape of the piece.
- Given Mix or Master mode, when the window draws, then the timeline runs left to right with no wrap.
- Given Mix mode, when the user scrolls sideways, then the strip headers stay in view.

**X-06 — Keyboard shortcuts (MUST)**
As a daily user, I drive the common work from the keyboard, so that my hands stay in place.
- Given any mode, when the user presses the space key, then the transport starts or stops.
- Given any mode, when the user presses the undo or redo shortcut, then the application reverses or repeats the last edit.
- Given any mode, when the user presses the save shortcut, then the application saves the project.
- Given any mode, when the user presses the shortcut for a mode, then the application switches to that mode.
- Given the application, when the user opens the shortcut list, then the list shows every shortcut by mode.
- Given macOS, when the user reads a shortcut, then the shortcut uses the command key where the platform expects it.
- Given Linux, when the user reads the same shortcut, then the shortcut uses the control key.
- Given a text field with focus, when the user types a letter, then the letter enters the field.
- Given a text field with focus, when the user types a letter, then no keyboard shortcut runs.

**X-07 — Menus and platform fit (MUST)**
As a user, I find the standard commands where my system puts them, so that the application feels native.
- Given macOS, when the application runs, then the system menu bar holds the file, edit, view, transport, and help menus.
- Given Linux, when the application runs, then the same menus appear in the window.
- Given any platform, when the user opens a menu, then each item shows its keyboard shortcut.

### 7.2 State that survives

**X-08 — Context and layout persistence (MUST)**
As a returned user, I find the application as I left it, so that I continue without setup.
- Given a closed project, when the user reopens it, then the mode, the zoom, and the scroll position return.
- Given a closed project, when the user reopens it, then the selected part and the selected take return.
- Given a resized window, when the user restarts the application, then the window opens at the same size and position.
- Given an armed track, when the user restarts the application, then no track is armed and the application says so.
- Given a mode switch, when the user returns to the first mode, then the scroll position and the selection are unchanged.

### 7.3 The window never blocks

**X-09 — Background work (MUST)**
As a user, I continue my work while the application does slow work, so that I never wait at a frozen window.
- Given any work longer than 100 milliseconds, when it runs, then the window stays responsive.
- Given background work, when it runs, then the window shows the work, its progress, and a cancel action.
- Given a cancelled task, when the user cancels, then the application stops the work and says what it kept.
- Given a finished task, when the user has moved to another mode, then the application shows a notice and does not steal focus.
- Given a finished task whose target no longer exists, when it completes, then the application discards the result and logs the reason.

Background work classes for the Architect:
- **Background task**: project load, project save, audio render, export, loudness measure, waveform build, pitch analysis, MIDI file import, git commit, git checkout.
- **Synchronous update**: note entry, note edit, selection, fader move, mute, solo, zoom, mode switch. Each must complete in one frame.
- **Periodic task**: MIDI and audio device list refresh, project directory watch, disk space check, autosave.

### 7.4 Load, empty, and error states

**X-10 — Load states (MUST)**
- Given a project load, when it runs, then the application draws the score outline first and fills the detail after.
- Given a waveform that is not built, when the lane draws, then the lane shows a flat placeholder.
- Given a placeholder waveform, when the peak data arrives, then the lane draws the true waveform.
- Given any load over one second, when it runs, then the application shows progress with a named step.
- Given a load state, when the data arrives, then the view does not jump and the scroll position stays.

**X-11 — Empty states (MUST)**
- Given a new project, when Compose draws, then the score shows empty measures and one hint that names the first action.
- Given a part with no track, when Record draws, then the part row offers one add-track action.
- Given a track with no take, when Record draws, then the lane says that the track is empty and offers the arm action.
- Given no mix change, when Mix draws, then the strips show default values and no empty panel.
- Given no export, when Master draws, then the surface shows the target and one export action.
- Given a history with one commit, when the surface draws, then it says that the project has one version.
- Given any empty state, when it draws, then it names one next action and uses no jargon.

**X-12 — Error states (MUST)**
- Given any error, when the application shows it, then the message names what failed, why, and what the user can do.
- Given a recoverable error, when it occurs, then the application shows an inline message and keeps the user's work.
- Given a mid-edit error, when it occurs, then the application does not close the surface that holds the edit.
- Given a failed background task, when it fails, then the application shows a notice with a retry action.
- Given a repeated identical error, when it occurs again within ten seconds, then the application does not stack a second notice.
- Given a project file that the application cannot read, when it opens, then the application offers the last good commit.
- Given a missing audio file, when the lane draws, then the lane marks the gap by name and the project still opens.
- Given a lost audio device, when it happens in any mode, then the application follows story R-13.

**X-13 — Destructive actions (MUST)**
- Given a delete of a part, a track, a take, or a branch, when the user asks, then the application asks to confirm.
- Given a confirm question, when it shows, then the question names what the application removes.
- Given a confirmed delete, when it completes, then the application keeps the audio files on disk.
- Given any non-destructive action, when the user runs it, then the application does not ask to confirm.

**X-14 — Accessible presentation (SHOULD)**
- Given any state that colour marks, when the user looks at it, then a shape or a label carries the same meaning.
- Given any text over its background, when it draws, then the contrast meets a 4.5 to 1 ratio.
- Given the application, when the user changes the system theme, then the application follows the light or dark theme.

---

## 8. Cut lines for v1

MUST: v1 does not ship without it. SHOULD: v1 ships better with it, and v1 can ship without it.
LATER: v1 ships without it by decision.

| Identifier | Story | Priority |
|---|---|---|
| A-01 | Headless project edit | MUST |
| A-02 | Live edit with the window open | MUST |
| A-03 | Text-readable score | MUST |
| A-04 | Status in one call | MUST |
| A-05 | MIDI file import by agent | MUST |
| C-01 | Start surface | MUST |
| C-02 | Create a project | MUST |
| C-03 | Open a project | MUST |
| C-04 | Save and dirty state | MUST |
| C-05 | Treble, bass, and grand staff | MUST |
| C-06 | Per-part staves | MUST |
| C-07 | Part management | MUST |
| C-08 | Note entry by click | MUST |
| C-09 | Held duration keys | MUST |
| C-10 | MIDI keyboard entry with no setup | MUST |
| C-11 | Insert point and step entry | MUST |
| C-12 | MIDI file import in the user interface | SHOULD |
| C-13 | Note context menu | MUST |
| C-14 | Direct note edits by keyboard and drag | SHOULD |
| C-15 | Top bar of notation options | MUST |
| C-16 | Clef, key, and time changes | MUST |
| C-17 | Tempo | MUST |
| C-18 | Dynamics and articulation | MUST |
| C-19 | Lyrics | MUST |
| C-20 | Selection | MUST |
| C-21 | Undo and redo | MUST |
| C-22 | Playback with a built-in voice | MUST |
| R-01 | Mode transition | MUST |
| R-02 | Part and track selection | MUST |
| R-03 | Show one track or all tracks | MUST |
| R-04 | Input selection and monitor | MUST |
| R-05 | Arm a track | MUST |
| R-06 | Count-in | MUST |
| R-07 | Record a layered take | MUST |
| R-08 | Overwrite record | MUST |
| R-09 | Layered takes and take selection | MUST |
| R-10 | Double a part | SHOULD |
| R-11 | Cut, splice, and trim | MUST |
| R-12 | Region move and snap | SHOULD |
| R-13 | The audio device disappears | MUST |
| R-14 | Disk and performance warnings | SHOULD |
| R-15 | Pitch track overlay | SHOULD |
| M-01 | Mix mode transition | MUST |
| M-02 | Track strips and part bus strips | MUST |
| M-03 | The vocal tool set | MUST |
| M-04 | Reverb and delay buses | MUST |
| M-05 | Metering | MUST |
| M-06 | Solo and mute | MUST |
| M-07 | Automation | SHOULD |
| M-08 | A and B compare | LATER |
| MA-01 | Loudness target and true-peak ceiling | MUST |
| MA-02 | Master chain | MUST |
| MA-03 | Measured report | MUST |
| MA-04 | Export to WAV, RF64, and FLAC | MUST |
| MA-05 | Per-part export | SHOULD |
| MA-06 | Sample rate and bit depth choice | SHOULD |
| H-01 | Commit from the user interface | MUST |
| H-02 | Browse the history | MUST |
| H-03 | Check out an earlier commit | MUST |
| H-04 | Branch | MUST |
| H-05 | Merge a branch | LATER |
| H-06 | Agent history control | MUST |
| X-01 | One timeline for notes and audio | MUST |
| X-02 | The playhead | MUST |
| X-03 | Zoom | MUST |
| X-04 | Wrapped timeline in Compose and Record | MUST |
| X-05 | Linear timeline in Mix and Master | MUST |
| X-06 | Keyboard shortcuts | MUST |
| X-07 | Menus and platform fit | MUST |
| X-08 | Context and layout persistence | MUST |
| X-09 | Background work | MUST |
| X-10 | Load states | MUST |
| X-11 | Empty states | MUST |
| X-12 | Error states | MUST |
| X-13 | Destructive actions | MUST |
| X-14 | Accessible presentation | SHOULD |

### 8.1 Reason for every LATER

- **M-08, A and B compare.** The user can reach the same result with undo and redo. The feature needs
  a second settings store per tool, which touches every tool. The cost is high and the daily user
  can work without it.
- **H-05, merge a branch.** A merge of two scores needs a music-aware three-way merge and a conflict
  surface. No such tool exists in Rust today, and a text merge of a score produces a broken file.
  Branch and checkout give the user the value of the feature. The merge can wait until the storage
  format is stable.

### 8.2 Features explicitly out of v1

These are not stories. They are named so that nobody adds them by accident.

- Plugin hosts of any kind. The operator ruled CLAP, VST3, AU, and LV2 out of scope.
- Print and page layout of the score. v1 shows the score on screen and exports MusicXML.
- Pitch correction and time alignment of a vocal take.
- Score entry from sung audio, which means audio to notation.
- Multi-user edits over a network.
- Video.
- A Windows build. The operator named macOS and Linux.

---

## 9. Open product questions

Nine questions need an operator answer. None blocks the start of work. Each carries a recommended
default that the team can build against now, and each is cheap to reverse later.

**Q1. Which duration keys exist beyond `w`, `h`, and `e`?**
The brief names three. Music needs a sixteenth note, a thirty-second note, and a triplet.
*Recommended default:* add `s` for the sixteenth note and `t` for the thirty-second note, keep the
same held-key rule, and put triplets in the note context menu only.

**Q2. How does the agent reach the open application?**
The application may watch the project directory, or it may listen on a local socket.
*Recommended default:* the CLI is the only interface the agent learns. When a window has the project
open, the CLI talks to it over a local socket. When no window is open, the CLI works on the files
directly. The verb names and the output are identical in both cases.

**Q3. Does the git history hold the audio files?**
Audio is large. A history that rewrites audio grows without limit.
*Recommended default:* yes, the history holds the audio. An audio source is immutable and the project
writes it once. Every edit changes only region numbers in the text state. Growth stays bounded.

**Q4. What sound does the built-in voice make?**
A sung vowel sample set is expensive to license and large to ship. A simple synthesized tone is small.
*Recommended default:* ship one small synthesized vowel-like tone that follows the dynamics marks.
The user needs pitch and rhythm to judge the music, not a realistic voice.

**Q5. What is the default part set for a new project?**
The brief names Soprano, Alto, Tenor, and Bass for Record mode. Compose mode may start smaller.
*Recommended default:* a new project offers two templates. "Solo voice" gives one treble staff.
"SATB" gives four staves. The create form defaults to "Solo voice", because one person writes alone
more often than a director writes for four parts.

**Q6. How many parts must v1 support at full speed?**
Density and draw cost depend on this number.
*Recommended default:* eight parts and 32 tracks, with 300 bars, at 60 frames per second. State this
as the performance budget and test against it.

**Q7. Which loudness presets ship with Master mode?**
Different targets suit different destinations.
*Recommended default:* three presets. Streaming at minus 14 LUFS with a minus 1 decibel true-peak
ceiling. Broadcast at minus 23 LUFS with a minus 1 decibel ceiling. A custom option with free values.

**Q8. What does a check out of an earlier commit do to the current work?**
The user may lose work if the application discards it.
*Recommended default:* the application never discards work in silence. It offers to commit the work
to a new branch, to discard it, or to cancel. The default button is the commit to a new branch.

**Q9. Does v1 ship a part print or a portable document format (PDF) export?**
The choir director wants paper for singers.
*Recommended default:* no. v1 exports MusicXML, which every notation program prints. State this in the
export surface so the director knows the path. Revisit after v1 measures how many users ask.

---

## 10. UX risks that the plan must answer

These are not stories. They are places where a technical choice will cost the user, based on the
research files. The Architect must answer each one in the specification.

1. **Held keys have no live query.** The GPUI Kit audit, section 4, records the gap. The staff view
   tracks key down and key up itself. A lost focus, a system dialog, or a mode switch can strand a key
   in the down state. Story C-09 makes the pending duration visible and makes a focus loss clear the
   state. Do not ship without both.

2. **No timeline, waveform, or meter component exists.** The audit, section 5, records the gap. Every
   surface in Record, Mix, and Master needs new draw code. The greatest risk is a level meter that
   redraws the whole view at 60 hertz. Keep the playhead and each meter in their own small surface.

3. **No toolbar component exists.** The audit, section 9, records the gap. The top bar of story C-15
   is new work. It must show current values, not only buttons, or the user cannot tell the key of the
   piece without a click.

4. **Painted paths and glyphs are not assertable in a test.** The audit, section 12, records the gap.
   The score layout must be a pure calculation that a test can check. The user-visible promise is that
   a note lands on the correct line, and the plan must be able to prove it without a screenshot.

5. **A take must survive every fault.** The Ardour notes, section 4, show that a record pass writes
   through a ring buffer to disk. Story R-13 demands that a device loss keeps every written sample.
   The design must write audio to disk as it records, not at the end of the pass.

6. **Non-destructive edit is a user promise, not only a design.** The Ardour notes, section 3, give the
   region model: a position, a start, and a length over an immutable source. Story R-11 promises that a
   trim is reversible. That promise holds only while no edit rewrites an audio file.

7. **Ardour has no notation at all.** The Ardour notes, section 11, state this plainly. Every score
   requirement in section 2 is new design with no reference implementation. Plan schedule and review
   effort for it.

---

## 11. Orchestrator decisions on the open questions (2026-09-20)

The Automated Orchestrator decided each open question. The recommended default stands unless a line below says otherwise.

| Question | Decision | Reason |
|---|---|---|
| Q1 duration keys | `w` whole, `h` half, `q` quarter (explicit), `e` eighth, `s` sixteenth, `t` thirty-second. Triplets live in the context menu. | One letter per duration keeps the rule the operator stated. |
| Q2 agent route | The `duet` command-line interface is the one interface the agent learns. It works on files when no window is open and over a local socket when one is. `duet mcp` serves the same verbs over the Model Context Protocol on stdio. | One verb list, two transports, no second vocabulary. |
| Q3 audio in history | Yes, by reference. A commit records every take by content hash in the media manifest; the audio bytes live in the content store inside the bundle, outside the git index (architecture section 4.2). A checkout restores every referenced take. | Growth is bounded by the count of takes, not by the count of edits, and git objects stay small. |
| Q4 built-in voice | One small synthesized vowel-like tone that follows dynamics. | Pitch and rhythm are what the user judges. |
| Q5 default parts | Two templates, "Solo voice" and "SATB"; the create form defaults to "Solo voice". | One person composes alone more often. |
| Q6 performance budget | 8 parts, 32 tracks, 300 bars, 60 frames per second. | A stated budget is testable. |
| Q7 loudness presets | Streaming (-14 LUFS, -1 dBTP), Broadcast (-23 LUFS, -1 dBTP), Custom. | Covers the two common destinations. |
| Q8 checkout with unsaved work | Never discard in silence. Offer commit to a new branch, discard, or cancel. Default is commit to a new branch. | Data loss is a Critical defect class. |
| Q9 print or PDF | No. Export MusicXML and say so on the export surface. | Every notation program prints MusicXML. |
