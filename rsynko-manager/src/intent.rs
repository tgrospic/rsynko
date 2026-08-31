use crate::{FormatDiscoveryOp, RehearsalObservationOp};
use alux_sdk::trait_algebra;
use std::path::PathBuf;
use std::time::Duration;

/// Defines the first-order observation stream affecting one transfer.
///
/// A variant is a method: the reified stream is generated from this vocabulary rather than written
/// out, so an interpreter states what an observation *means* and never restates the shape.
#[trait_algebra(derive(Clone, Debug, PartialEq, Eq))]
pub trait TransferObservation {
    /// Observes whether the active interpreter can cooperatively pause retrieval.
    fn pause_capability(&self, supported: bool);

    /// Denotes that an interpreter started extraction.
    fn started(&self);

    /// Observes elapsed transfer time.
    fn elapsed(&self, elapsed: Duration);

    /// Observes incremental byte progress toward one intended final path.
    fn progress(&self, destination: PathBuf, downloaded: u64, total: Option<u64>);

    /// Observes terminal completion at a published path.
    fn completed(&self, destination: PathBuf, bytes: u64);

    /// Observes terminal failure before publication completed.
    fn failed(&self, destination: PathBuf, message: String);

    /// Denotes failure outside the resource-download program.
    fn program_failed(&self, summary: String, detail: String);
}

/// Defines the first-order intent stream one manager interprets.
///
/// An intention is a method, so the reified stream is generated from this vocabulary rather than
/// written out. An interpreter states what each intention *means* and never restates the shape.
#[trait_algebra(derive(Clone, Debug, PartialEq, Eq))]
pub trait ManagerIntent {
    /// Represents one stable queue identity.
    type Id;
    /// Represents one submitted source.
    type Source;
    /// Represents one selectable format description.
    type Format;
    /// Represents one change a rehearsal states.
    type Change;

    /// Adds explicit source requests to the collection.
    fn add_sources(&self, requests: Vec<Self::Source>);

    /// Replaces the editor draft.
    fn set_draft(&self, draft: String);

    /// Inserts text at the active editor cursor.
    fn insert_text(&self, text: String);

    /// Replaces the output-file-name editor draft.
    fn set_output_draft(&self, draft: String);

    /// Replaces the input editor draft.
    fn set_input_draft(&self, draft: String);

    /// Supplies extracted identity and title observations.
    fn source_metadata(&self, id: Self::Id, media_id: String, title: Option<String>);

    /// Applies one format-discovery observation to a stable queue identity.
    fn format_catalog(&self, id: Self::Id, event: FormatDiscoveryOp<Self::Format>);

    /// Applies one rehearsal observation to a stable queue identity.
    fn rehearsal(&self, id: Self::Id, event: RehearsalObservationOp<Self::Change>);

    /// Applies one transfer observation to a stable queue identity.
    fn transfer(&self, id: Self::Id, event: TransferObservationOp);

    /// Opens the add-sources editor.
    fn open_add_sources(&self);

    /// Submits newline-separated sources from the editor draft.
    fn submit_draft(&self);

    /// Deletes the Unicode scalar before the active editor cursor.
    fn delete_before_cursor(&self);

    /// Deletes the Unicode scalar at the active editor cursor.
    fn delete_at_cursor(&self);

    /// Moves the active editor cursor left by one Unicode scalar.
    fn move_cursor_left(&self);

    /// Moves the active editor cursor right by one Unicode scalar.
    fn move_cursor_right(&self);

    /// Moves the active editor cursor to its logical line beginning.
    fn move_cursor_home(&self);

    /// Moves the active editor cursor to its logical line end.
    fn move_cursor_end(&self);

    /// Selects the preceding collection entry with wraparound.
    fn select_previous(&self);

    /// Selects the following collection entry with wraparound.
    fn select_next(&self);

    /// Expands details for the selected entry while preserving row focus.
    fn open_selected(&self);

    /// Selects the preceding visible details control.
    fn select_previous_detail(&self);

    /// Selects the following visible details control.
    fn select_next_detail(&self);

    /// Activates the selected control, or collapses details while the row has focus.
    fn activate_detail(&self);

    /// Opens the concrete-format selector for the selected editable entry.
    fn open_formats(&self);

    /// Opens output-file-name editing for the selected editable entry.
    fn open_output(&self);

    /// Opens input editing for the selected entry, while its input may still be changed.
    fn open_input(&self);

    /// Applies the input editor draft.
    fn submit_input(&self);

    /// Opens the report of what the selected request would do.
    fn open_report(&self);

    /// Turns the rehearsal mode of the selected request on or off.
    fn toggle_dry_run(&self);

    /// Applies the normalized output-file-name editor draft.
    fn submit_output(&self);

    /// Navigates to the parent page.
    fn back(&self);

    /// Applies the selected entry's currently valid space action.
    fn apply_selected_space(&self);

    /// Selects the preceding compatible format choice.
    fn select_previous_format(&self);

    /// Selects the following compatible format choice.
    fn select_next_format(&self);

    /// Duplicates the selected source as a fresh editable request.
    fn duplicate_selected(&self);

    /// Removes the selected entry.
    fn remove_selected(&self);

    /// Marks the selected failed entry as waiting with its fixed options.
    fn restart_selected(&self);

    /// Requests exit after active work reaches a safe boundary.
    fn safe_exit_requested(&self);
}

/// Denotes the valid meaning of Space for one queue entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpaceAction {
    /// Fixes and queues one ready request.
    Start,
    /// Queues one ready request to state what it would do instead of doing it.
    Rehearse,
    /// Pauses a cooperatively pausable active transfer.
    Pause,
    /// Resumes a paused transfer.
    Resume,
}

/// Denotes one manager page independently of rendering mechanism.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManagerPage<Id> {
    /// Denotes the queue collection and its list selector.
    Collection,
    /// Denotes the add-sources editor.
    AddSources,
    /// Denotes one queue identity expanded within the collection.
    Details(Id),
    /// Denotes format selection for one editable queue identity.
    Formats(Id),
    /// Denotes output-file-name editing for one editable queue identity.
    Output(Id),
    /// Denotes input editing for one queue identity whose input may still be changed.
    Input(Id),
    /// Denotes the record of what was observed about one queue identity.
    Log(Id),
    /// Denotes the report of what one queue identity would do.
    Report(Id),
    /// Denotes the command one queue identity would run, stated so it can be taken away.
    Command(Id),
}

/// Identifies one visible, selectable control in expanded details.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetailControl {
    /// Opens input editing.
    Input,
    /// Opens output-file-name editing.
    Output,
    /// Opens concrete-format selection.
    Format,
    /// Opens the command the request would run, stated whole.
    Command,
    /// Restarts one failed request with its fixed options.
    Restart,
    /// Duplicates the source into a fresh editable request.
    Duplicate,
    /// Turns the rehearsal mode on or off.
    DryRun,
    /// Opens the report of what the request would do.
    Report,
    /// Opens the record of what was observed about the request.
    Log,
    /// Deletes the queue entry.
    Delete,
}

/// Denotes one breadcrumb segment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Breadcrumb {
    /// Names the segment.
    pub label: String,
}
