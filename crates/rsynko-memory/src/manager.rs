use crate::{DownloadEvent, DownloadOptions, Format, FormatCatalog, FormatChoice, role_accepts};
use crate::{RsyncSyntax, SyncCommand};
use rsynko_manager::*;
use rsynko_media::portable_file_name;
use rsynko_rsync::{
    RsyncEndpointExt, SyncCommandExt, SyncCommandViewAlg, SyncMode, SyncProfile, sync_profile,
};
use rsynko_ui::{DiscoveryState, FormatChoiceViewAlg, FormatLabelExt, FormatRolesExt};
use std::ops::Not;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Identifies one queue entry independently of list position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QueueId(pub u64);

/// Selects the deterministic in-memory interpretation of manager carriers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MemoryManager;

/// Denotes one requested source and optional explicit output path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceRequest {
    /// Names the source URL.
    pub source: String,
    /// Names an explicit output path when requested.
    pub output: Option<PathBuf>,
    /// Selects stream roles and format identity before execution begins.
    pub options: DownloadOptions,
}

impl SourceRequest {
    /// Constructs a source request with explicit output and download options.
    #[must_use]
    pub fn new(
        source: impl Into<String>,
        output: Option<PathBuf>,
        options: DownloadOptions,
    ) -> Self {
        Self {
            source: source.into(),
            output,
            options,
        }
    }
}

impl ManagerSorts for MemoryManager {
    type Id = ();
    type Source = SourceRequest;
    type Options = DownloadOptions;
    type Output = Option<PathBuf>;
    type Format = Format;
    type Change = PlannedChange;
    type Entry = ();
    type Downloads = ManagerState;
}

impl ManagerSorts for ManagerState {
    type Id = QueueId;
    type Source = SourceRequest;
    type Options = DownloadOptions;
    type Output = Option<PathBuf>;
    type Format = Format;
    type Change = PlannedChange;
    type Entry = QueueEntry;
    type Downloads = ();
}

impl DownloadsAlg for MemoryManager {
    fn empty_downloads(&self) -> Self::Downloads {
        ManagerState::downloads()
    }
}

impl SourceRequestAlg for MemoryManager {
    fn source(
        &self,
        input: impl Into<String>,
        output: Self::Output,
        options: Self::Options,
    ) -> Self::Source {
        SourceRequest::new(input, output, options)
    }
}

impl MediaOptionsAlg for MemoryManager {
    fn progressive(&self) -> Self::Options {
        DownloadOptions::progressive()
    }

    fn audio(&self) -> Self::Options {
        DownloadOptions::audio()
    }

    fn video(&self) -> Self::Options {
        DownloadOptions::video()
    }
}

impl SourceRecognitionAlg for MemoryManager {
    fn recognizes_source(&self, line: &str) -> bool {
        recognized(line.trim())
    }
}

impl SubmissionAlg for MemoryManager {
    fn submitted(&self, line: &str) -> Self::Source {
        submitted(line)
    }
}

impl OutputChoiceAlg for MemoryManager {
    fn suggested_output(&self) -> Self::Output {
        None
    }

    fn exact_output(&self, path: impl Into<PathBuf>) -> Self::Output {
        Some(path.into())
    }
}

impl SourceRequestAlg for ManagerState {
    fn source(
        &self,
        input: impl Into<String>,
        output: Self::Output,
        options: Self::Options,
    ) -> Self::Source {
        SourceRequest::new(input, output, options)
    }
}

impl MediaOptionsAlg for ManagerState {
    fn progressive(&self) -> Self::Options {
        DownloadOptions::progressive()
    }

    fn audio(&self) -> Self::Options {
        DownloadOptions::audio()
    }

    fn video(&self) -> Self::Options {
        DownloadOptions::video()
    }
}

impl SourceRecognitionAlg for ManagerState {
    fn recognizes_source(&self, line: &str) -> bool {
        recognized(line.trim())
    }
}

impl SubmissionAlg for ManagerState {
    fn submitted(&self, line: &str) -> Self::Source {
        submitted(line)
    }
}

impl OutputChoiceAlg for ManagerState {
    fn suggested_output(&self) -> Self::Output {
        None
    }

    fn exact_output(&self, path: impl Into<PathBuf>) -> Self::Output {
        Some(path.into())
    }
}

/// Denotes the mechanism-independent state of one transfer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransferState {
    phase: TransferPhase,
    destination: Option<PathBuf>,
    downloaded: u64,
    total: Option<u64>,
    elapsed: Duration,
    terminal: Option<DownloadEvent>,
    summary: Option<String>,
    detail: Option<String>,
    pause_supported: bool,
    resume_phase: Option<TransferPhase>,
}

impl TransferState {
    fn ready() -> Self {
        Self {
            phase: TransferPhase::Ready,
            destination: None,
            downloaded: 0,
            total: None,
            elapsed: Duration::ZERO,
            terminal: None,
            summary: None,
            detail: None,
            pause_supported: false,
            resume_phase: None,
        }
    }

    /// Observes the transfer phase.
    #[must_use]
    pub const fn phase(&self) -> TransferPhase {
        self.phase
    }

    /// Observes the final destination when known.
    #[must_use]
    pub fn destination(&self) -> Option<&Path> {
        self.destination.as_deref()
    }

    /// Observes retrieved bytes.
    #[must_use]
    pub const fn downloaded(&self) -> u64 {
        self.downloaded
    }

    /// Observes expected bytes when known.
    #[must_use]
    pub const fn total(&self) -> Option<u64> {
        self.total
    }

    /// Observes elapsed transfer time.
    #[must_use]
    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Observes the terminal download event when one occurred.
    #[must_use]
    pub const fn terminal(&self) -> Option<&DownloadEvent> {
        self.terminal.as_ref()
    }

    /// Observes the concise user-facing failure summary.
    #[must_use]
    pub fn summary(&self) -> Option<&str> {
        self.summary.as_deref()
    }

    /// Observes detailed diagnostics.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl TransferViewAlg for TransferState {
    fn transferred(&self) -> u64 {
        self.downloaded
    }

    fn transfer_total(&self) -> Option<u64> {
        self.total
    }

    fn transfer_elapsed(&self) -> Duration {
        self.elapsed
    }

    fn transfer_complete(&self) -> bool {
        self.phase == TransferPhase::Complete
    }

    fn transfer_destination(&self) -> Option<&Path> {
        self.destination()
    }

    fn transfer_summary(&self) -> Option<&str> {
        self.summary()
    }

    fn transfer_detail(&self) -> Option<&str> {
        self.detail()
    }
}

/// Denotes one source in the in-memory manager collection.
#[derive(Clone, Debug, PartialEq)]
pub struct QueueEntry {
    id: QueueId,
    request: SourceRequest,
    transfer: TransferState,
    formats: FormatCatalog,
    media_id: Option<String>,
    title: Option<String>,
    output_custom: bool,
    log: Vec<String>,
    rehearsal: Rehearsal,
}

/// Denotes what rehearsal has stated about one request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rehearsal {
    /// Denotes a request no one has rehearsed.
    Unrehearsed,
    /// Denotes a rehearsal an interpreter is currently performing.
    Rehearsing,
    /// Denotes the changes a rehearsal stated, in the order it stated them.
    Reported(Vec<PlannedChange>),
    /// Denotes a rehearsal failure suitable for display.
    Failed(String),
}

/// Denotes one change a rehearsal stated the transfer would make.
#[derive(Clone, Debug, PartialEq, Eq, derive_new::new)]
pub struct PlannedChange {
    /// Names the path relative to the transferred folder.
    pub path: String,
    /// States what would happen to that path.
    pub kind: ChangeKind,
    /// States the byte count the change moves, when the rehearsal stated one.
    pub size: Option<u64>,
}

impl PlannedChangeAlg for PlannedChange {
    fn change_path(&self) -> &str {
        &self.path
    }

    fn change_kind(&self) -> ChangeKind {
        self.kind
    }

    fn change_size(&self) -> Option<u64> {
        self.size
    }
}

impl QueueEntry {
    /// Observes stable queue identity.
    #[must_use]
    pub const fn id(&self) -> QueueId {
        self.id
    }

    /// Observes the requested source.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.request.source
    }

    /// Observes the explicit output path when present.
    #[must_use]
    pub fn output(&self) -> Option<&Path> {
        self.request.output.as_deref()
    }

    /// Observes the extracted media title when known.
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Observes transfer state.
    #[must_use]
    pub const fn transfer(&self) -> &TransferState {
        &self.transfer
    }

    /// Observes choices fixed when execution first starts.
    #[must_use]
    pub const fn options(&self) -> &DownloadOptions {
        &self.request.options
    }

    /// Observes discovered selectable formats.
    #[must_use]
    pub const fn formats(&self) -> &FormatCatalog {
        &self.formats
    }

    /// Observes whether request choices remain editable.
    #[must_use]
    pub fn is_editable(&self) -> bool {
        self.transfer.phase == TransferPhase::Ready
    }

    /// Derives visible details controls from transfer state.
    #[must_use]
    pub fn detail_controls(&self) -> Vec<DetailControl> {
        let rehearsed = matches!(self.rehearsal, Rehearsal::Unrehearsed)
            .not()
            .then_some(DetailControl::Report);
        let rehearsal = self
            .request
            .options
            .dry_run()
            .is_some()
            .then_some(DetailControl::DryRun);
        // A request performed by naming a program states that command; one fetched states none.
        let commanded = self
            .stated_command()
            .is_some()
            .then_some(DetailControl::Command);
        match self.transfer.phase {
            TransferPhase::Ready => [
                // An input already read cannot change: everything derived from it would be a lie.
                (self.output_naming() == OutputNaming::Stated).then_some(DetailControl::Input),
                Some(DetailControl::Output),
                // A media item chooses a representation; a folder chooses a way of transferring.
                (self.request.options.media_streams().is_some()
                    || self.request.options.profile().is_some())
                .then_some(DetailControl::Format),
                commanded,
                rehearsed,
                Some(DetailControl::Log),
                rehearsal,
                Some(DetailControl::Duplicate),
                Some(DetailControl::Delete),
            ]
            .into_iter()
            .flatten()
            .collect(),
            // The cursor walks what is shown, and the record is a field above the actions.
            TransferPhase::Failed => [
                commanded,
                rehearsed,
                Some(DetailControl::Log),
                Some(DetailControl::Restart),
                Some(DetailControl::Duplicate),
                Some(DetailControl::Delete),
            ]
            .into_iter()
            .flatten()
            .collect(),
            TransferPhase::Rehearsing
            | TransferPhase::Waiting
            | TransferPhase::Extracting
            | TransferPhase::Downloading
            | TransferPhase::Paused
            | TransferPhase::Publishing
            | TransferPhase::Complete => [
                commanded,
                rehearsed,
                Some(DetailControl::Log),
                Some(DetailControl::Duplicate),
                Some(DetailControl::Delete),
            ]
            .into_iter()
            .flatten()
            .collect(),
        }
    }

    /// Derives the currently valid meaning of Space.
    #[must_use]
    pub fn space_action(&self) -> Option<SpaceAction> {
        match self.transfer.phase {
            TransferPhase::Ready if self.output().is_some() => Some(match self.dry_run() {
                Some(true) => SpaceAction::Rehearse,
                Some(false) | None => SpaceAction::Start,
            }),
            TransferPhase::Extracting | TransferPhase::Downloading
                if self.transfer.pause_supported =>
            {
                Some(SpaceAction::Pause)
            }
            TransferPhase::Paused => Some(SpaceAction::Resume),
            TransferPhase::Ready
            | TransferPhase::Waiting
            | TransferPhase::Rehearsing
            | TransferPhase::Extracting
            | TransferPhase::Downloading
            | TransferPhase::Publishing
            | TransferPhase::Complete
            | TransferPhase::Failed => None,
        }
    }

    /// States exactly the command an interpreter would run to perform this request.
    ///
    /// Only a transfer is performed by naming a program. The command is derived from the two
    /// ends, the way of transferring, and the rehearsal mode, so changing any of them changes it.
    /// An interpreter runs *this* command, and a reader reads *this* command: one derivation, so
    /// what is shown and what is run cannot drift apart.
    #[must_use]
    pub fn transfer_command(&self) -> Option<SyncCommand> {
        let dry_run = self.request.options.dry_run()?;
        let destination = self.output()?;
        let mode = SyncMode::transfer()
            .rehearsed(dry_run)
            .profiled(self.request.options.profile()?);
        Some(RsyncSyntax.transfer_command(
            &RsyncSyntax.read_endpoint(&self.request.source),
            &RsyncSyntax.read_endpoint(&destination.display().to_string()),
            mode,
        ))
    }

    /// States what an interpreter would run, as a reader reads it.
    #[must_use]
    pub fn stated_command(&self) -> Option<String> {
        let command = self.transfer_command()?;
        Some(format!(
            "{} {}",
            RsyncSyntax.command_program(&command),
            RsyncSyntax
                .command_arguments(&command)
                .collect::<Vec<_>>()
                .join(" ")
        ))
    }

    /// Observes the notes stated about this request, in the order they were stated.
    pub fn log(&self) -> impl DoubleEndedIterator<Item = &str> {
        self.log.iter().map(String::as_str)
    }

    /// States one note about this request.
    fn note(&mut self, note: impl Into<String>) {
        self.log.push(note.into());
    }

    /// Forgets what a rehearsal stated, because it no longer states what would happen.
    ///
    /// A report describes one command. Change what would be run — either end, or the way of
    /// running it — and the report describes a command nobody is going to run.
    fn forget_rehearsal(&mut self) {
        if matches!(self.rehearsal, Rehearsal::Unrehearsed) {
            return;
        }
        self.rehearsal = Rehearsal::Unrehearsed;
        self.note("the report was forgotten: it no longer states what would happen");
    }

    /// Derives the source input before inspection and the media title afterward.
    #[must_use]
    pub fn label(&self) -> &str {
        self.title
            .as_deref()
            .filter(|title| !title.is_empty())
            .unwrap_or(&self.request.source)
    }
}

impl TransferViewAlg for QueueEntry {
    fn transferred(&self) -> u64 {
        self.transfer.transferred()
    }

    fn transfer_total(&self) -> Option<u64> {
        self.transfer.transfer_total()
    }

    fn transfer_elapsed(&self) -> Duration {
        self.transfer.transfer_elapsed()
    }

    fn transfer_complete(&self) -> bool {
        self.transfer.transfer_complete()
    }

    fn transfer_destination(&self) -> Option<&Path> {
        self.transfer.transfer_destination()
    }

    fn transfer_summary(&self) -> Option<&str> {
        self.transfer.transfer_summary()
    }

    fn transfer_detail(&self) -> Option<&str> {
        self.transfer.transfer_detail()
    }
}

impl QueueEntryAlg for QueueEntry {
    fn label(&self) -> &str {
        self.label()
    }

    fn source(&self) -> &str {
        self.source()
    }

    fn phase(&self) -> TransferPhase {
        self.transfer.phase()
    }

    fn dry_run(&self) -> Option<bool> {
        self.request.options.dry_run()
    }

    fn output_naming(&self) -> OutputNaming {
        // A transferred path is stated by whoever asked for it; a downloaded file is named here.
        if self.request.options.profile().is_some() {
            OutputNaming::Stated
        } else {
            OutputNaming::Portable
        }
    }

    fn stated_command(&self) -> Option<String> {
        self.stated_command()
    }

    fn output(&self) -> Option<&Path> {
        self.output()
    }

    fn is_editable(&self) -> bool {
        self.is_editable()
    }

    fn detail_controls(&self) -> Vec<DetailControl> {
        self.detail_controls()
    }

    fn download_log(&self) -> impl Iterator<Item = &str> {
        self.log()
    }

    fn space_action(&self) -> Option<SpaceAction> {
        self.space_action()
    }
}

impl FormatChoiceViewAlg for QueueEntry {
    type Format = Format;

    fn discovery(&self) -> DiscoveryState<'_> {
        match &self.formats {
            FormatCatalog::Unknown => DiscoveryState::Unrequested,
            FormatCatalog::Waiting => DiscoveryState::Waiting,
            FormatCatalog::Inspecting => DiscoveryState::Inspecting,
            FormatCatalog::Available(_) => DiscoveryState::Described,
            FormatCatalog::Failed(message) => DiscoveryState::Failed(message),
        }
    }

    fn described_formats(&self) -> impl Iterator<Item = &Self::Format> {
        self.formats.available().unwrap_or_default().iter()
    }
}

impl RehearsalViewAlg for QueueEntry {
    type Change = PlannedChange;

    fn rehearsal(&self) -> RehearsalState<'_> {
        match &self.rehearsal {
            Rehearsal::Unrehearsed => RehearsalState::Unrehearsed,
            Rehearsal::Rehearsing => RehearsalState::Rehearsing,
            Rehearsal::Reported(_) => RehearsalState::Reported,
            Rehearsal::Failed(message) => RehearsalState::Failed(message),
        }
    }

    fn planned_changes(&self) -> impl Iterator<Item = &Self::Change> {
        match &self.rehearsal {
            Rehearsal::Reported(changes) => changes.as_slice(),
            Rehearsal::Unrehearsed | Rehearsal::Rehearsing | Rehearsal::Failed(_) => &[],
        }
        .iter()
    }
}

impl RequestOptionsAlg for QueueEntry {
    fn performer(&self) -> Performer {
        self.request.options.performer()
    }

    fn media_streams(&self) -> Option<MediaStreams> {
        self.request.options.media_streams()
    }

    fn chosen_choice(&self) -> Option<&str> {
        // A folder always transfers one stated way; a media item may leave its format to ranking.
        if let Some(profile) = self.request.options.profile() {
            return Some(sync_profile::to(profile));
        }
        match self.request.options.format() {
            FormatChoice::Best => None,
            FormatChoice::Id(id) => Some(id),
        }
    }

    fn selectable_choices(&self) -> impl Iterator<Item = &str> {
        let ways = self
            .request
            .options
            .profile()
            .map(|_| sync_profile::ALL)
            .unwrap_or_default()
            .iter()
            .copied()
            .map(sync_profile::to)
            .collect::<Vec<_>>();
        ways.into_iter().chain(
            self.formats
                .available()
                .unwrap_or_default()
                .iter()
                .map(|format| format.id.as_str()),
        )
    }

    fn choice_summary(&self, choice: &str) -> Option<&str> {
        // A described format is read through what it carries; a way of transferring says what it
        // does, because nothing else about it would tell a reader.
        sync_profile::from(choice).map(SyncProfile::summary)
    }
}

/// Denotes the complete renderer-neutral manager application state.
#[derive(Clone, Debug, PartialEq)]
pub struct ManagerState {
    page: ManagerPage<QueueId>,
    queue: Vec<QueueEntry>,
    selected: Option<QueueId>,
    detail_control: Option<DetailControl>,
    draft: String,
    draft_cursor: usize,
    output_draft: String,
    output_draft_cursor: usize,
    input_draft: String,
    input_draft_cursor: usize,
    message: Option<String>,
    exit_requested: bool,
    next_id: u64,
}

impl ManagerState {
    /// Constructs the downloads collection page.
    #[must_use]
    pub const fn downloads() -> Self {
        Self {
            page: ManagerPage::Collection,
            queue: Vec::new(),
            selected: None,
            detail_control: None,
            draft: String::new(),
            draft_cursor: 0,
            output_draft: String::new(),
            output_draft_cursor: 0,
            input_draft: String::new(),
            input_draft_cursor: 0,
            message: None,
            exit_requested: false,
            next_id: 0,
        }
    }

    /// Observes the current page.
    #[must_use]
    pub const fn page(&self) -> ManagerPage<QueueId> {
        self.page
    }

    /// Observes queue entries in collection order.
    #[must_use]
    pub fn queue(&self) -> &[QueueEntry] {
        &self.queue
    }

    /// Observes the selected stable identity.
    #[must_use]
    pub const fn selected_id(&self) -> Option<QueueId> {
        self.selected
    }

    /// Observes the selected visible details control.
    #[must_use]
    pub const fn detail_control(&self) -> Option<DetailControl> {
        self.detail_control
    }

    /// Observes the selected entry.
    #[must_use]
    pub fn selected(&self) -> Option<&QueueEntry> {
        let selected = self.selected?;
        self.entry(selected)
    }

    /// Observes one entry by stable identity.
    #[must_use]
    pub fn entry(&self, id: QueueId) -> Option<&QueueEntry> {
        self.queue.iter().find(|entry| entry.id == id)
    }

    /// Observes the add-sources editor draft.
    #[must_use]
    pub fn draft(&self) -> &str {
        &self.draft
    }

    /// Observes the output-file-name editor draft.
    #[must_use]
    pub fn output_draft(&self) -> &str {
        &self.output_draft
    }

    /// Observes the manager status message.
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Observes all waiting entries in collection order.
    pub fn waiting(&self) -> impl Iterator<Item = &QueueEntry> {
        self.queue
            .iter()
            .filter(|entry| entry.transfer.phase == TransferPhase::Waiting)
    }

    /// Observes the first format inspection waiting for an interpreter.
    #[must_use]
    pub fn first_waiting_format_catalog(&self) -> Option<&QueueEntry> {
        self.queue
            .iter()
            .find(|entry| entry.formats == FormatCatalog::Waiting)
    }

    /// Observes whether any transfer is active.
    #[must_use]
    pub fn has_active_transfer(&self) -> bool {
        self.queue.iter().any(|entry| {
            matches!(
                entry.transfer.phase,
                TransferPhase::Extracting
                    | TransferPhase::Downloading
                    | TransferPhase::Paused
                    | TransferPhase::Publishing
            )
        })
    }
}

impl DownloadCollectionAlg for ManagerState {
    fn add_sources(mut self, sources: impl IntoIterator<Item = Self::Source>) -> Self {
        let requests: Vec<Self::Source> = sources.into_iter().collect();
        self.apply_manager_event(ManagerIntentOp::AddSources { requests });
        self
    }
}

impl NavigationStateAlg for ManagerState {
    fn page(&self) -> ManagerPage<Self::Id> {
        self.page
    }

    fn set_page(&mut self, page: ManagerPage<Self::Id>) {
        self.page = page;
    }
}

impl QueueCatalogAlg for ManagerState {
    fn queue_ids(&self) -> impl Iterator<Item = Self::Id> {
        self.queue.iter().map(|entry| entry.id)
    }

    fn selected_queue_id(&self) -> Option<Self::Id> {
        self.selected
    }

    fn set_selected_queue_id(&mut self, selected: Option<Self::Id>) {
        self.selected = selected;
    }

    fn queue_entry(&self, id: Self::Id) -> Option<&Self::Entry> {
        self.entry(id)
    }
}

impl QueueAppendAlg for ManagerState {
    fn append_sources(&mut self, requests: Vec<Self::Source>) -> Vec<Self::Id> {
        requests
            .into_iter()
            .map(|request| {
                let id = QueueId(self.next_id);
                self.next_id = self.next_id.saturating_add(1);
                let output_custom = request.output.is_some();
                let request_source = request.source.clone();
                let formats = if request.options.performer() == Performer::Retrieval {
                    FormatCatalog::Waiting
                } else {
                    FormatCatalog::Unknown
                };
                self.queue.push(QueueEntry {
                    id,
                    request,
                    transfer: TransferState::ready(),
                    formats,
                    media_id: None,
                    title: None,
                    output_custom,
                    log: vec![format!("added {}", request_source)],
                    rehearsal: Rehearsal::Unrehearsed,
                });
                refresh_default_output(self.queue.last_mut().expect("the request was appended"));
                id
            })
            .collect()
    }
}

impl QueueRemoveAlg for ManagerState {
    fn remove_queue_entry(&mut self, id: Self::Id) {
        self.queue.retain(|entry| entry.id != id);
    }
}

impl QueuePauseAlg for ManagerState {
    fn toggle_queue_pause(&mut self, id: Self::Id) {
        let Some(entry) = self.queue.iter_mut().find(|entry| entry.id == id) else {
            return;
        };
        match entry.space_action() {
            Some(SpaceAction::Pause) => {
                entry.transfer.resume_phase = Some(entry.transfer.phase);
                entry.transfer.phase = TransferPhase::Paused;
            }
            Some(SpaceAction::Resume) => {
                entry.transfer.phase = entry
                    .transfer
                    .resume_phase
                    .take()
                    .unwrap_or(TransferPhase::Downloading);
            }
            // Pausing names an active transfer; starting and rehearsing are not that.
            Some(SpaceAction::Start | SpaceAction::Rehearse) | None => {}
        }
    }
}

impl QueueDuplicateAlg for ManagerState {
    fn duplicate_queue_entry(&mut self, id: Self::Id) -> Option<Self::Id> {
        let entry = self.queue.iter().find(|entry| entry.id == id)?.clone();
        let source = entry.source().to_owned();
        // A fresh request states what it would do before it does it, however it came to exist.
        let options = entry.request.options.clone().rehearsing();
        // A transfer is the pair of ends it names, so duplicating one keeps both. A download
        // produces a file of its own, and two of them may not produce the same one.
        let output = entry
            .request
            .options
            .profile()
            .and(entry.request.output.clone());
        let stated = output.is_some();
        let formats = match entry.formats {
            FormatCatalog::Available(formats) => FormatCatalog::Available(formats),
            FormatCatalog::Unknown
            | FormatCatalog::Waiting
            | FormatCatalog::Inspecting
            | FormatCatalog::Failed(_) => FormatCatalog::Unknown,
        };
        let duplicate = QueueId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.queue.push(QueueEntry {
            id: duplicate,
            request: SourceRequest::new(source, output, options),
            transfer: TransferState::ready(),
            formats,
            media_id: entry.media_id,
            title: entry.title,
            output_custom: stated,
            log: vec![format!("duplicated from download {}", entry.id.0)],
            rehearsal: Rehearsal::Unrehearsed,
        });
        refresh_default_output(self.queue.last_mut().expect("duplicate was appended"));
        Some(duplicate)
    }
}

impl QueueFormatEditAlg for ManagerState {
    fn cycle_queue_format(&mut self, id: Self::Id, forward: bool) {
        let Some(entry) = self.queue.iter_mut().find(|entry| entry.id == id) else {
            return;
        };
        if !entry.is_editable() {
            return;
        }
        // A folder's choices are the ways it may be transferred, and nothing else.
        if entry.request.options.profile().is_some() {
            let ways = sync_profile::ALL;
            let current = entry
                .request
                .options
                .profile()
                .and_then(|profile| ways.iter().position(|way| *way == profile))
                .unwrap_or(0);
            let next = if forward {
                (current + 1) % ways.len()
            } else if current == 0 {
                ways.len() - 1
            } else {
                current - 1
            };
            entry.request.options = entry.request.options.clone().with_profile(ways[next]);
            entry.forget_rehearsal();
            return;
        }
        // One list states the choices: prefer a role, or fix one identity extraction discovered.
        // A request choosing no media role offers no roles to cycle through, only identities.
        let offered = entry
            .request
            .options
            .media_streams()
            .map(|_| entry.offered_streams())
            .unwrap_or_default();
        let choices: Vec<DownloadOptions> = offered
            .into_iter()
            .map(|streams| entry.request.options.clone().with_media_streams(streams))
            .chain(
                entry
                    .formats
                    .available()
                    .unwrap_or_default()
                    .iter()
                    .map(|format| {
                        entry
                            .request
                            .options
                            .clone()
                            .with_format(FormatChoice::Id(format.id.clone()))
                    }),
            )
            .collect();
        let current = choices
            .iter()
            .position(|choice| *choice == entry.request.options)
            .unwrap_or(0);
        let next = if forward {
            (current + 1) % choices.len()
        } else if current == 0 {
            choices.len() - 1
        } else {
            current - 1
        };
        entry.request.options = choices[next].clone();
        refresh_default_output(entry);
    }
}

impl FormatCatalogStateAlg for ManagerState {
    fn request_format_catalog(&mut self, id: Self::Id) {
        if let Some(entry) = self.queue.iter_mut().find(|entry| entry.id == id)
            && entry.is_editable()
            // A folder has nothing to inspect: the ways it may be transferred are always offered.
            && entry.request.options.performer() == Performer::Retrieval
            && matches!(
                entry.formats,
                FormatCatalog::Unknown | FormatCatalog::Failed(_)
            )
        {
            entry.formats = FormatCatalog::Waiting;
        }
    }

    fn apply_format_catalog_event(&mut self, id: Self::Id, event: FormatDiscoveryOp<Self::Format>) {
        let Some(entry) = self.queue.iter_mut().find(|entry| entry.id == id) else {
            return;
        };
        entry.formats = match event {
            FormatDiscoveryOp::Started {} => {
                entry.note("inspecting source formats");
                FormatCatalog::Inspecting
            }
            FormatDiscoveryOp::Available { formats } => {
                entry.note(format!("extraction described {} formats", formats.len()));
                for format in &formats {
                    entry.note(described_format(format));
                }
                FormatCatalog::Available(formats)
            }
            FormatDiscoveryOp::Failed { message } => {
                entry.note(format!("format inspection failed: {message}"));
                FormatCatalog::Failed(message)
            }
        };
        // A request cannot go on asking for something the source turned out not to have. A tweet
        // states one file carrying both streams, so sound alone is not among the things it holds;
        // one carrying only pictures states no streams at all, and is asked for by name instead.
        // A request that already named what it wants is left alone: it asked for that.
        if matches!(entry.request.options.format(), FormatChoice::Best)
            && let Some(asked) = entry.request.options.media_streams()
        {
            let offered = entry.offered_streams();
            if let Some(carried) = offered.first().copied() {
                if !offered.contains(&asked) {
                    entry.request.options =
                        entry.request.options.clone().with_media_streams(carried);
                }
            } else if let Some(named) = entry
                .formats
                .available()
                .and_then(|described| described.first())
                .map(|format| format.id.clone())
            {
                entry.request.options = entry
                    .request
                    .options
                    .clone()
                    .with_format(FormatChoice::Id(named));
            }
        }
        refresh_default_output(entry);
    }
}

impl OutputDraftAlg for ManagerState {
    fn output_draft(&self) -> &str {
        &self.output_draft
    }

    fn set_output_draft(&mut self, draft: String) {
        self.output_draft_cursor = draft.len();
        self.output_draft = draft;
    }
}

impl InputDraftAlg for ManagerState {
    fn input_draft(&self) -> &str {
        &self.input_draft
    }

    fn set_input_draft(&mut self, draft: String) {
        self.input_draft_cursor = draft.len();
        self.input_draft = draft;
    }
}

impl QueueSourceAlg for ManagerState {
    fn set_queue_source(&mut self, id: Self::Id, source: String) {
        if let Some(entry) = self.queue.iter_mut().find(|entry| entry.id == id)
            && entry.is_editable()
        {
            entry.note(format!("input changed to {source}"));
            entry.request.source = source;
            entry.forget_rehearsal();
            refresh_default_output(entry);
        }
    }
}

impl QueueOutputAlg for ManagerState {
    fn set_queue_output(&mut self, id: Self::Id, output: PathBuf) {
        if let Some(entry) = self.queue.iter_mut().find(|entry| entry.id == id)
            && entry.is_editable()
        {
            entry.request.output = Some(output);
            entry.output_custom = true;
            entry.forget_rehearsal();
        }
    }
}

impl DownloadLogAlg for ManagerState {
    fn note_download(&mut self, id: Self::Id, note: String) {
        if let Some(entry) = self.queue.iter_mut().find(|entry| entry.id == id) {
            entry.note(note);
        }
    }
}

impl SourceMetadataAlg for ManagerState {
    fn apply_source_metadata(&mut self, id: Self::Id, media_id: String, title: Option<String>) {
        if let Some(entry) = self.queue.iter_mut().find(|entry| entry.id == id) {
            entry.note(match &title {
                Some(title) => format!("extracted {media_id}: {title}"),
                None => format!("extracted {media_id}"),
            });
            entry.media_id = Some(media_id);
            entry.title = title;
            refresh_default_output(entry);
        }
    }
}

fn refresh_default_output(entry: &mut QueueEntry) {
    if entry.output_custom {
        return;
    }
    // A transfer naming only where it comes from comes to rest under the name it already has.
    // Naming a folder is safer than naming the one somebody happens to be standing in, which is
    // what mirroring into it would empty.
    if entry.request.options.performer() == Performer::Program {
        let source = RsyncSyntax.read_endpoint(&entry.request.source);
        entry.request.output = Some(PathBuf::from(RsyncSyntax.endpoint_name(&source)));
        return;
    }
    let Some(media_id) = entry.media_id.as_deref() else {
        return;
    };
    entry.request.output = Some(portable_file_name(
        entry.title.as_deref(),
        media_id,
        selected_extension(entry),
    ));
}

fn selected_extension(entry: &QueueEntry) -> Option<&str> {
    let formats = entry.formats.available()?;
    match entry.request.options.format() {
        FormatChoice::Best => formats
            .iter()
            .rev()
            .find(|format| {
                entry
                    .request
                    .options
                    .media_streams()
                    .is_some_and(|streams| role_accepts(streams, format))
            })
            .and_then(Format::extension),
        FormatChoice::Id(id) => formats
            .iter()
            .find(|format| format.id == *id)
            .and_then(Format::extension),
    }
}

impl TransferStateAlg for ManagerState {
    fn set_waiting(&mut self, id: Self::Id) {
        if let Some(entry) = self.queue.iter_mut().find(|entry| entry.id == id)
            && entry.is_editable()
            && entry.output().is_some()
        {
            entry.transfer = TransferState::ready();
            entry.transfer.phase = TransferPhase::Waiting;
        }
    }

    fn restart_waiting(&mut self, id: Self::Id) {
        if let Some(entry) = self.queue.iter_mut().find(|entry| entry.id == id)
            && entry.transfer.phase == TransferPhase::Failed
        {
            entry.transfer = TransferState::ready();
            entry.transfer.phase = TransferPhase::Waiting;
        }
    }

    fn apply_transfer_event(&mut self, id: Self::Id, event: TransferObservationOp) {
        let Some(entry) = self.queue.iter_mut().find(|entry| entry.id == id) else {
            return;
        };
        event.interpret(&mut EntryObservation(entry));
    }
}

/// Applies one transfer observation to the entry it addresses.
///
/// The dispatch is the generated fold, so this states only what each observation *means*.
struct EntryObservation<'a>(&'a mut QueueEntry);

impl TransferObservationInterpreter for EntryObservation<'_> {
    fn pause_capability(&mut self, supported: bool) {
        self.0.transfer.pause_supported = supported;
    }

    fn started(&mut self) {
        self.0.note("retrieval started");
        self.0.transfer.phase = TransferPhase::Extracting;
    }

    fn elapsed(&mut self, elapsed: Duration) {
        self.0.transfer.elapsed = elapsed;
    }

    fn progress(&mut self, destination: PathBuf, downloaded: u64, total: Option<u64>) {
        let observed_phase = if total.is_some_and(|total| downloaded >= total) {
            TransferPhase::Publishing
        } else {
            TransferPhase::Downloading
        };
        if self.0.transfer.phase == TransferPhase::Paused {
            self.0.transfer.resume_phase = Some(observed_phase);
        } else {
            self.0.transfer.phase = observed_phase;
        }
        self.0.transfer.destination = Some(destination);
        self.0.transfer.downloaded = downloaded;
        self.0.transfer.total = total;
    }

    fn completed(&mut self, destination: PathBuf, bytes: u64) {
        self.0.note(format!(
            "published {} ({bytes} bytes)",
            destination.display()
        ));
        self.0.transfer.phase = TransferPhase::Complete;
        self.0.transfer.terminal = Some(DownloadEvent::Succeeded { destination, bytes });
    }

    fn failed(&mut self, destination: PathBuf, message: String) {
        self.0.note(format!(
            "retrieval failed at {}: {message}",
            destination.display()
        ));
        self.0.transfer.phase = TransferPhase::Failed;
        self.0.transfer.terminal = Some(DownloadEvent::Failed {
            destination,
            message,
        });
    }

    fn program_failed(&mut self, summary: String, detail: String) {
        self.0.note(format!("{summary}: {detail}"));
        self.0.transfer.phase = TransferPhase::Failed;
        self.0.transfer.summary = Some(summary);
        self.0.transfer.detail = Some(detail);
    }
}

impl ManagerState {
    /// Makes one request a transfer, the way a submitted line naming two ends does.
    ///
    /// A submitted line states what a request is; this is how a test states one without a line
    /// that would.
    pub fn transfer_request(&mut self, id: QueueId) {
        if let Some(entry) = self.queue.iter_mut().find(|entry| entry.id == id) {
            entry.request.options = DownloadOptions::transfer();
        }
    }
}

impl QueueDryRunAlg for ManagerState {
    fn set_queue_dry_run(&mut self, id: Self::Id, dry_run: bool) {
        if let Some(entry) = self.queue.iter_mut().find(|entry| entry.id == id)
            && entry.is_editable()
        {
            entry.request.options.set_dry_run(dry_run);
            entry.note(if dry_run {
                "dry run enabled: the next run will state what it would do and change nothing"
            } else {
                "dry run disabled: the next run will perform the transfer"
            });
        }
    }
}

impl RehearsalStateAlg for ManagerState {
    fn apply_rehearsal_event(&mut self, id: Self::Id, event: RehearsalObservationOp<Self::Change>) {
        let Some(entry) = self.queue.iter_mut().find(|entry| entry.id == id) else {
            return;
        };
        match event {
            RehearsalObservationOp::Started {} => {
                entry.note("rehearsing what the transfer would do");
                entry.transfer.phase = TransferPhase::Rehearsing;
                entry.rehearsal = Rehearsal::Rehearsing;
            }
            RehearsalObservationOp::Reported { changes } => {
                let altered = changes.iter().filter(|change| change.kind.alters()).count();
                entry.note(format!(
                    "the rehearsal stated {altered} of {} paths would change",
                    changes.len()
                ));
                // A rehearsal is not a start: the request stays exactly as editable as it was.
                entry.transfer.phase = TransferPhase::Ready;
                entry.rehearsal = Rehearsal::Reported(changes);
            }
            RehearsalObservationOp::Failed { message } => {
                entry.note(format!("rehearsal failed: {message}"));
                entry.transfer.phase = TransferPhase::Ready;
                entry.rehearsal = Rehearsal::Failed(message);
            }
        }
    }
}

impl DetailSelectionAlg for ManagerState {
    fn selected_detail_control(&self) -> Option<DetailControl> {
        self.detail_control
    }

    fn set_selected_detail_control(&mut self, control: Option<DetailControl>) {
        self.detail_control = control;
    }
}

impl DraftStateAlg for ManagerState {
    fn draft(&self) -> &str {
        &self.draft
    }

    fn set_draft(&mut self, draft: String) {
        self.draft_cursor = draft.len();
        self.draft = draft;
    }
}

impl TextEditorStateAlg for ManagerState {
    fn active_text_editor(&self) -> Option<(&str, usize)> {
        match self.page {
            ManagerPage::AddSources => Some((&self.draft, self.draft_cursor)),
            ManagerPage::Output(_) => Some((&self.output_draft, self.output_draft_cursor)),
            ManagerPage::Input(_) => Some((&self.input_draft, self.input_draft_cursor)),
            ManagerPage::Collection
            | ManagerPage::Details(_)
            | ManagerPage::Formats(_)
            | ManagerPage::Log(_)
            | ManagerPage::Report(_)
            | ManagerPage::Command(_) => None,
        }
    }

    fn set_active_text_editor(&mut self, text: String, cursor: usize) {
        let mut cursor = cursor.min(text.len());
        while !text.is_char_boundary(cursor) {
            cursor = cursor.saturating_sub(1);
        }
        match self.page {
            ManagerPage::AddSources => {
                self.draft = text;
                self.draft_cursor = cursor;
            }
            ManagerPage::Output(_) => {
                self.output_draft = text;
                self.output_draft_cursor = cursor;
            }
            ManagerPage::Input(_) => {
                self.input_draft = text;
                self.input_draft_cursor = cursor;
            }
            ManagerPage::Collection
            | ManagerPage::Details(_)
            | ManagerPage::Formats(_)
            | ManagerPage::Log(_)
            | ManagerPage::Report(_)
            | ManagerPage::Command(_) => {}
        }
    }
}

impl ManagerStatusAlg for ManagerState {
    fn manager_message(&self) -> Option<&str> {
        self.message()
    }

    fn set_manager_message(&mut self, message: Option<String>) {
        self.message = message;
    }
}

impl SafeExitAlg for ManagerState {
    fn exit_requested(&self) -> bool {
        self.exit_requested
    }

    fn request_safe_exit(&mut self) {
        self.exit_requested = true;
    }
}

/// States what one submitted line names.
///
/// A line naming two ends names a transfer between them. A line naming one end names a transfer
/// when that end is on another machine or ends with the separator a folder ends with, and a media
/// item otherwise. What is transferred may be one file or a whole folder; either way it is
/// rehearsed before it is performed, while a media item is extracted and downloaded.
fn submitted(line: &str) -> SourceRequest {
    let trimmed = line.trim();
    // A line some source claims is retrieved from it; everything else is a path, whether it names
    // one end or two.
    if recognized(trimmed) {
        return SourceRequest::new(trimmed, None, DownloadOptions::progressive());
    }
    if let Some((source, destination)) = RsyncSyntax.read_transfer(line) {
        return SourceRequest::new(
            RsyncSyntax.endpoint_text(&source),
            Some(PathBuf::from(RsyncSyntax.endpoint_text(&destination))),
            DownloadOptions::transfer(),
        );
    }
    SourceRequest::new(trimmed, None, DownloadOptions::transfer())
}

/// Names the schemes a source fetches from, rather than paths a transfer walks.
///
/// Youtube reads its own watch, shortened, embedded, and short-form links out of a web address;
/// the deterministic fixture names its own scheme; and direct retrieval reads any other web
/// address. Nothing else here is a scheme: a host and a path, a folder, and a file are all paths.
const RETRIEVED_SCHEMES: [&str; 3] = ["https://", "http://", "fixture://"];

/// States whether one submitted line names something a source retrieves for itself.
///
/// This application transfers paths. A line reads as anything else only because a source claimed
/// it, so `/home/dev/music`, `nas.local:/srv/data`, and `rsync://nas.local/data` are transfers,
/// and adding a source to this composition is adding what it recognizes here.
fn recognized(line: &str) -> bool {
    RETRIEVED_SCHEMES
        .iter()
        .any(|scheme| line.starts_with(scheme))
}

/// States one described format the way the record reads it back.
fn described_format(format: &Format) -> String {
    format!("  {}", format.format_label())
}
