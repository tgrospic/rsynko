use crate::ManagerSorts;
use crate::*;
use ambassador::delegatable_trait;
use std::path::{Path, PathBuf};

/// Specifies ordered queue observation and stable selection.
#[delegatable_trait]
pub trait QueueCatalogAlg: ManagerSorts {
    /// Observes stable identities in collection order.
    fn queue_ids(&self) -> impl Iterator<Item = Self::Id>;
    /// Observes the selected stable identity.
    fn selected_queue_id(&self) -> Option<Self::Id>;
    /// Sets the selected stable identity.
    fn set_selected_queue_id(&mut self, selected: Option<Self::Id>);
    /// Observes one entry by stable identity.
    fn queue_entry(&self, id: Self::Id) -> Option<&Self::Entry>;
}

/// Specifies the queue-entry observations used by manager derivations.
pub trait QueueEntryAlg {
    /// Observes the submitted source until extraction supplies a title, and that title after.
    fn label(&self) -> &str;
    /// Observes the source the request was submitted with.
    fn source(&self) -> &str;
    /// Observes the lifecycle phase the request has reached.
    fn phase(&self) -> TransferPhase;
    /// Observes the rehearsal mode, and that the request has none when it has none.
    fn dry_run(&self) -> Option<bool>;
    /// Observes exactly what an interpreter would run to perform the request.
    ///
    /// A request an interpreter performs by naming a program states that program and everything
    /// it would be given, so a reader sees what pressing Space does before pressing it. A request
    /// retrieved by fetching states nothing here: there is no command to read.
    fn stated_command(&self) -> Option<String>;
    /// Observes the explicit output path when present.
    fn output(&self) -> Option<&Path>;
    /// Observes how the output is named, which decides what an edited name is taken to be.
    fn output_naming(&self) -> OutputNaming;
    /// Observes whether request choices remain editable.
    fn is_editable(&self) -> bool;
    /// Derives visible details controls from transfer state.
    fn detail_controls(&self) -> Vec<DetailControl>;
    /// Derives the currently valid meaning of Space.
    fn space_action(&self) -> Option<SpaceAction>;
    /// Observes the notes stated about this request, in the order they were stated.
    fn download_log(&self) -> impl Iterator<Item = &str>;
}

/// Denotes how one request's output is named.
///
/// The two are not interchangeable. A name this application invents for a file it produces has to
/// be one every filesystem accepts, and is normalized until it is. A path somebody stated is
/// already the answer, and normalizing it would replace the separators that make it a path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputNaming {
    /// Names one file this application produces, made portable before it is used.
    Portable,
    /// Names one path somebody stated, taken exactly as it was written.
    Stated,
}

/// Specifies ordered queue insertion.
#[delegatable_trait]
pub trait QueueAppendAlg: ManagerSorts {
    /// Appends sources and returns their stable identities in source order.
    fn append_sources(&mut self, requests: Vec<Self::Source>) -> Vec<Self::Id>;
}

/// Specifies queue removal.
#[delegatable_trait]
pub trait QueueRemoveAlg: ManagerSorts {
    /// Removes one stable identity.
    fn remove_queue_entry(&mut self, id: Self::Id);
}

/// Specifies cooperative pause transitions.
#[delegatable_trait]
pub trait QueuePauseAlg: ManagerSorts {
    /// Toggles one active transfer between paused and resumed state.
    fn toggle_queue_pause(&mut self, id: Self::Id);
}

/// Specifies fresh-request duplication.
#[delegatable_trait]
pub trait QueueDuplicateAlg: ManagerSorts {
    /// Duplicates one source as a fresh request and returns its identity.
    fn duplicate_queue_entry(&mut self, id: Self::Id) -> Option<Self::Id>;
}

/// Specifies ready-request format-choice editing.
#[delegatable_trait]
pub trait QueueFormatEditAlg: ManagerSorts {
    /// Selects an adjacent choice: a preferred stream role, or one discovered identity.
    fn cycle_queue_format(&mut self, id: Self::Id, forward: bool);
}

/// Specifies explicit output assignment.
#[delegatable_trait]
pub trait QueueOutputAlg: ManagerSorts {
    /// Fixes one editable queue entry's explicit output file name.
    fn set_queue_output(&mut self, id: Self::Id, output: PathBuf);
}

/// Specifies input replacement.
#[delegatable_trait]
pub trait QueueSourceAlg: ManagerSorts {
    /// Replaces one request's input, while that request still permits it.
    fn set_queue_source(&mut self, id: Self::Id, source: String);
}

/// Specifies format-catalog requests and external observations.
#[delegatable_trait]
pub trait FormatCatalogStateAlg: ManagerSorts {
    /// Marks one editable entry as waiting for source inspection.
    fn request_format_catalog(&mut self, id: Self::Id);
    /// Applies one format-catalog observation.
    fn apply_format_catalog_event(&mut self, id: Self::Id, event: FormatDiscoveryOp<Self::Format>);
}

/// Specifies extracted source naming observations.
#[delegatable_trait]
pub trait SourceMetadataAlg: ManagerSorts {
    /// Applies extracted identity and title observations.
    fn apply_source_metadata(&mut self, id: Self::Id, media_id: String, title: Option<String>);
}

/// Specifies transfer scheduling and observations by stable identity.
#[delegatable_trait]
pub trait TransferStateAlg: ManagerSorts {
    /// Marks one entry as waiting for an interpreter.
    fn set_waiting(&mut self, id: Self::Id);
    /// Marks one failed entry as waiting without changing its fixed request.
    fn restart_waiting(&mut self, id: Self::Id);
    /// Applies one transfer observation.
    fn apply_transfer_event(&mut self, id: Self::Id, event: TransferObservationOp);
}
