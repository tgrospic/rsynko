//! The context every specification law scenario runs against.
//!
//! One value ties every sort at once, but it states almost nothing itself: each capability is
//! delegated to the component interpreter that already provides it. The struct is generic in those
//! components, so the composition is structural rather than hard-wired, and with zero-sized
//! syntax interpreters the compiler leaves no trace of it.

use crate::{
    Extraction, Format, InfoRecord, ManagerState, Media, MediaSyntax, ProcessingStep,
    ProcessingSyntax, ProcessorId, REFERENCE_MEDIA_EXTENSION, REFERENCE_MEDIA_URL,
    REFERENCE_VIDEO_ID, REFERENCE_WATCH_URL, ReferenceDownloadEnv, ReferenceExtractor,
    ReferenceExtractorRegistry, ReferenceYoutubeDownloadEnv, ReferenceYoutubeEnv, YoutubeRequest,
};
use crate::{
    MemoryManager, PlannedChange, QueueEntry, ReferenceSyncEnv, RsyncEndpoint, SyncCommand,
    SyncObservation, TextScreen,
};
use ambassador::Delegate;
use rsynko_download::*;
use rsynko_manager::*;
use rsynko_media::*;
use rsynko_rsync::*;
use rsynko_ui::ScreenLawFixture;
use rsynko_yt::*;
use std::path::{Path, PathBuf};

const LAW_RESOURCE: &str = "memory://law-resource";
const LAW_BYTES: &[u8] = b"exact bytes";

/// Denotes a processor the reference context refuses to apply.
#[derive(Debug, thiserror::Error)]
#[error("processor {0} refused")]
pub struct ReferenceProcessingRefusal(String);

/// Composes reference interpreters into one context that satisfies every law scenario.
// Several capabilities delegate to the same component, which Clippy reads as a repeated attribute.
#[allow(
    clippy::duplicated_attributes,
    reason = "one delegation per capability, not per target"
)]
#[derive(Debug, Delegate)]
#[delegate(MetadataAlg, target = "syntax")]
#[delegate(FormatAlg, target = "syntax")]
#[delegate(FormatViewAlg, target = "syntax")]
#[delegate(ArtifactAlg, target = "syntax")]
#[delegate(ExtractionAlg, target = "syntax")]
#[delegate(ExtractionViewAlg, target = "syntax")]
#[delegate(MediaViewAlg, target = "syntax")]
#[delegate(FormatPredicateAlg, target = "syntax")]
#[delegate(FormatSelectionAlg, target = "syntax")]
#[delegate(FormatPredicateMatchAlg, target = "syntax")]
#[delegate(FormatSelectionApplyAlg, target = "syntax")]
#[delegate(OutputNameAlg, target = "syntax")]
#[delegate(ProcessingProgramAlg, target = "programs")]
#[delegate(
    ExtractionCatalogAlg,
    target = "catalog",
    where = "Catalog: ExtractionCatalogAlg + MediaSorts<Extractor = <Syntax as MediaSorts>::Extractor>, Syntax: MediaSorts"
)]
#[delegate(
    ExtractionApplyAlg,
    target = "catalog",
    where = "Catalog: ExtractionApplyAlg + MediaSorts<Extractor = <Syntax as MediaSorts>::Extractor, Extraction = <Syntax as MediaSorts>::Extraction>, Syntax: MediaSorts"
)]
#[delegate(DraftStateAlg, target = "manager")]
#[delegate(OutputDraftAlg, target = "manager")]
#[delegate(InputDraftAlg, target = "manager")]
#[delegate(TextEditorStateAlg, target = "manager")]
#[delegate(NavigationStateAlg, target = "manager")]
#[delegate(DetailSelectionAlg, target = "manager")]
#[delegate(ManagerStatusAlg, target = "manager")]
#[delegate(SafeExitAlg, target = "manager")]
#[delegate(QueueCatalogAlg, target = "manager")]
#[delegate(QueueAppendAlg, target = "manager")]
#[delegate(QueueRemoveAlg, target = "manager")]
#[delegate(QueuePauseAlg, target = "manager")]
#[delegate(QueueDuplicateAlg, target = "manager")]
#[delegate(QueueFormatEditAlg, target = "manager")]
#[delegate(QueueOutputAlg, target = "manager")]
#[delegate(QueueSourceAlg, target = "manager")]
#[delegate(FormatCatalogStateAlg, target = "manager")]
#[delegate(QueueDryRunAlg, target = "manager")]
#[delegate(RehearsalStateAlg, target = "manager")]
#[delegate(SourceMetadataAlg, target = "manager")]
#[delegate(DownloadLogAlg, target = "manager")]
#[delegate(TransferStateAlg, target = "manager")]
#[delegate(SourceRequestAlg, target = "manager")]
#[delegate(SourceRecognitionAlg, target = "manager")]
#[delegate(SubmissionAlg, target = "manager")]
#[delegate(MediaOptionsAlg, target = "manager")]
#[delegate(OutputChoiceAlg, target = "manager")]
#[delegate(YoutubeUrlAlg, target = "youtube")]
#[delegate(
    YoutubeRequestAlg,
    target = "youtube",
    where = "Youtube: YoutubeRequestAlg + YoutubeSorts"
)]
#[delegate(YoutubeProgramAlg, target = "youtube")]
#[delegate(YoutubeClientAlg, target = "youtube")]
#[delegate(
    YoutubeRequestBytesAlg,
    target = "youtube",
    where = "Youtube: YoutubeRequestBytesAlg + YoutubeSorts"
)]
#[delegate(YoutubeResponseAlg, target = "youtube")]
#[delegate(
    YoutubeChallengeAlg,
    target = "youtube",
    where = "Youtube: YoutubeChallengeAlg + YoutubeSorts"
)]
#[delegate(
    YoutubeSolutionAlg,
    target = "youtube",
    where = "Youtube: YoutubeSolutionAlg + YoutubeSorts"
)]
#[delegate(AtomicPublishAlg, target = "downloads")]
#[delegate(RsyncEndpointAlg, target = "sync")]
#[delegate(RsyncEndpointViewAlg, target = "sync")]
#[delegate(SyncCommandAlg, target = "sync")]
#[delegate(SyncCommandViewAlg, target = "sync")]
#[delegate(SyncChangeAlg, target = "sync")]
#[delegate(SyncObservationAlg, target = "sync")]
#[delegate(SyncObservationViewAlg, target = "sync")]
#[delegate(SyncRunAlg, target = "sync")]
#[delegate(SyncWatchAlg, target = "sync")]
#[delegate(DownloadObservationAlg, target = "downloads")]
#[delegate(DownloadReportAlg, target = "downloads")]
#[delegate(DownloadProgressAlg, target = "downloads")]
pub struct LawEnv<Syntax, Programs, Catalog, Downloads, Youtube, Manager> {
    syntax: Syntax,
    programs: Programs,
    catalog: Catalog,
    downloads: Downloads,
    youtube: Youtube,
    manager: Manager,
    sync: ReferenceSyncEnv,
    applied: Vec<String>,
    refused: Option<String>,
}

impl<Syntax, Programs, Catalog, Downloads, Youtube> RehearsalLawFixture
    for LawEnv<Syntax, Programs, Catalog, Downloads, Youtube, ManagerState>
{
    fn law_change(&self, path: &str, kind: ChangeKind, size: Option<u64>) -> Self::Change {
        PlannedChange::new(path.to_owned(), kind, size)
    }

    fn law_offer_rehearsal(&mut self, id: Self::Id) {
        self.manager.transfer_request(id);
    }
}

impl<Syntax, Programs, Catalog, Downloads, Youtube, Manager> RsyncSorts
    for LawEnv<Syntax, Programs, Catalog, Downloads, Youtube, Manager>
{
    type Endpoint = RsyncEndpoint;
    type Command = SyncCommand;
    type Observation = SyncObservation;
    type Change = PlannedChange;
}

impl<Syntax, Programs, Catalog, Downloads, Youtube, Manager> SubmissionLawFixture
    for LawEnv<Syntax, Programs, Catalog, Downloads, Youtube, Manager>
{
    fn law_folder_submission(&self) -> (String, String, String) {
        let input = "backup@nas.local:/volume1/photos/2026".to_owned();
        let output = "/home/dev/photos/2026".to_owned();
        (format!("{input} {output}"), input, output)
    }

    fn law_media_source(&self) -> String {
        "https://www.youtube.com/watch?v=VIDEO_ID".to_owned()
    }

    fn law_unrecognized_source(&self) -> String {
        "/home/dev/music".to_owned()
    }

    fn law_lone_submission(&self) -> (String, String) {
        (
            "backup@nas.local:/volume1/photos/2026/".to_owned(),
            "2026".to_owned(),
        )
    }
}

impl<Syntax, Programs, Catalog, Downloads, Youtube, Manager> SyncLawFixture
    for LawEnv<Syntax, Programs, Catalog, Downloads, Youtube, Manager>
{
    fn law_transcript(&mut self, lines: Vec<String>) {
        self.sync.register_transcript(lines);
    }

    fn law_ran(&self, command: &Self::Command) -> bool {
        self.sync.commands().last() == Some(command)
    }
}

impl<Syntax, Programs, Catalog, Downloads, Youtube, Manager> ScreenLawFixture
    for LawEnv<Syntax, Programs, Catalog, Downloads, Youtube, Manager>
{
    type Syntax = TextScreen;

    fn law_screen_syntax(&self) -> Self::Syntax {
        TextScreen
    }
}

/// Names the reference composition every law scenario runs against.
pub type ReferenceLaws = LawEnv<
    MediaSyntax,
    ProcessingSyntax,
    ReferenceExtractorRegistry,
    ReferenceDownloadEnv,
    ReferenceYoutubeEnv,
    ManagerState,
>;

/// Names the composition the Youtube law scenarios run against.
///
/// Youtube retrieval consumes the reified request rather than a URL, so this composition differs
/// from the generic one in exactly one component: the interpreter that locates a request's bytes.
pub type ReferenceYoutubeLaws = LawEnv<
    MediaSyntax,
    ProcessingSyntax,
    ReferenceExtractorRegistry,
    ReferenceYoutubeDownloadEnv,
    ReferenceYoutubeEnv,
    ManagerState,
>;

impl Default for ReferenceYoutubeLaws {
    fn default() -> Self {
        let mut downloads = ReferenceYoutubeDownloadEnv::default();
        downloads.register_resource(REFERENCE_MEDIA_URL, LAW_BYTES.to_vec());
        Self {
            syntax: MediaSyntax,
            programs: ProcessingSyntax,
            catalog: ReferenceExtractorRegistry::default(),
            downloads,
            youtube: ReferenceYoutubeEnv::default(),
            manager: ManagerState::downloads(),
            sync: ReferenceSyncEnv::default(),
            applied: Vec::default(),
            refused: None,
        }
    }
}

impl Default for ReferenceLaws {
    fn default() -> Self {
        let mut catalog = ReferenceExtractorRegistry::default();
        for key in ["first", "second"] {
            catalog.push(ReferenceExtractor::succeeds(
                key,
                "https://example.test/",
                Extraction::Media(Media::new(
                    key.to_owned(),
                    InfoRecord::default(),
                    Vec::default(),
                )),
            ));
        }
        let mut downloads = ReferenceDownloadEnv::default();
        downloads.register_resource(LAW_RESOURCE, LAW_BYTES.to_vec());
        Self {
            syntax: MediaSyntax,
            programs: ProcessingSyntax,
            catalog,
            downloads,
            youtube: ReferenceYoutubeEnv::default(),
            manager: ManagerState::downloads(),
            sync: ReferenceSyncEnv::default(),
            applied: Vec::default(),
            refused: None,
        }
    }
}

// The composition states no sort of its own: it reads each one off the component that interprets
// it, so the components must agree on the sorts they share.
impl<
    Syntax,
    Programs,
    Catalog,
    Downloads,
    Youtube,
    Manager,
    Value,
    Metadata,
    Format,
    Artifact,
    Media,
    Extraction,
    Extractor,
    Predicate,
    Selection,
    Output,
> MediaSorts for LawEnv<Syntax, Programs, Catalog, Downloads, Youtube, Manager>
where
    Syntax: MediaSorts<
            Value = Value,
            Metadata = Metadata,
            Format = Format,
            Artifact = Artifact,
            Media = Media,
            Extraction = Extraction,
            Extractor = Extractor,
            Predicate = Predicate,
            Selection = Selection,
            Output = Output,
        >,
{
    type Value = Value;
    type Metadata = Metadata;
    type Format = Format;
    type Artifact = Artifact;
    type Media = Media;
    type Extraction = Extraction;
    type Extractor = Extractor;
    type Predicate = Predicate;
    type Selection = Selection;
    type Output = Output;
}

impl<
    Syntax,
    Programs,
    Catalog,
    Downloads,
    Youtube,
    Manager,
    Id,
    Requested,
    Options,
    Named,
    Described,
    Planned,
    Entry,
    Collection,
> ManagerSorts for LawEnv<Syntax, Programs, Catalog, Downloads, Youtube, Manager>
where
    Manager: ManagerSorts<
            Id = Id,
            Source = Requested,
            Options = Options,
            Output = Named,
            Format = Described,
            Change = Planned,
            Entry = Entry,
            Downloads = Collection,
        >,
{
    type Id = Id;
    type Source = Requested;
    type Options = Options;
    type Output = Named;
    type Format = Described;
    type Change = Planned;
    type Entry = Entry;
    type Downloads = Collection;
}

impl<Syntax, Programs, Catalog, Downloads, Youtube, Manager, Request, Solutions> YoutubeSorts
    for LawEnv<Syntax, Programs, Catalog, Downloads, Youtube, Manager>
where
    Youtube: YoutubeSorts<Request = Request, Solutions = Solutions>,
{
    type Request = Request;
    type Solutions = Solutions;
}

impl<Syntax, Programs, Catalog, Downloads, Youtube, Manager, Processor, Step, Program>
    ProcessingSorts for LawEnv<Syntax, Programs, Catalog, Downloads, Youtube, Manager>
where
    Programs: ProcessingSorts<Processor = Processor, Step = Step, Program = Program>,
{
    type Processor = Processor;
    type Step = Step;
    type Program = Program;
}

// Associated functions carry no receiver, so the view is projected rather than delegated.
impl<Syntax, Programs, Catalog, Downloads, Youtube, Manager> ProcessingProgramViewAlg
    for LawEnv<Syntax, Programs, Catalog, Downloads, Youtube, Manager>
where
    Programs: ProcessingProgramViewAlg,
{
    fn processing_steps<'a>(program: &'a Self::Program) -> impl Iterator<Item = &'a Self::Step>
    where
        Self::Step: 'a,
    {
        Programs::processing_steps(program)
    }

    fn processing_stage(step: &Self::Step) -> ProcessingStage {
        Programs::processing_stage(step)
    }
}

// Retrieval is delegated by hand: the fixture states its source sort, and the component
// interprets the default one.
impl<Syntax, Programs, Catalog, Downloads, Youtube, Manager, FetchError, Stream>
    FetchStreamAlg<String> for LawEnv<Syntax, Programs, Catalog, Downloads, Youtube, Manager>
where
    Downloads: FetchStreamAlg<Error = FetchError, Stream = Stream>,
{
    type Error = FetchError;
    type Stream = Stream;

    fn open_fetch(&self, source: &String) -> Result<FetchStream<Self::Stream>, Self::Error> {
        self.downloads.open_fetch(source)
    }

    fn read_fetch(
        &self,
        stream: &mut Self::Stream,
        buffer: &mut [u8],
    ) -> Result<usize, Self::Error> {
        self.downloads.read_fetch(stream, buffer)
    }
}

// The context records its own application trace, so these it states rather than delegates.
impl<Syntax, Programs, Catalog, Downloads, Youtube, Manager> ProcessingApplyAlg
    for LawEnv<Syntax, Programs, Catalog, Downloads, Youtube, Manager>
where
    Programs: ProcessingSorts<Processor = ProcessorId, Step = ProcessingStep>,
{
    type Error = ReferenceProcessingRefusal;

    fn apply_processing_step(&mut self, step: &Self::Step) -> Result<(), Self::Error> {
        let processor = step.processor.0.clone();
        if self.refused.as_ref() == Some(&processor) {
            return Err(ReferenceProcessingRefusal(processor));
        }
        self.applied.push(processor);
        Ok(())
    }
}

impl<Syntax, Programs, Catalog, Downloads, Youtube, Manager> ProcessingLawFixture
    for LawEnv<Syntax, Programs, Catalog, Downloads, Youtube, Manager>
{
    fn law_applied(&self) -> Vec<String> {
        self.applied.clone()
    }

    fn refuse_law_processor(&mut self, id: &str) {
        self.refused = Some(id.to_owned());
    }
}

// The fixture reaches this context's own catalog, so it is stated for the reference composition.
impl ExtractionLawFixture for ReferenceLaws {
    fn accepted_law_url(&self) -> String {
        "https://example.test/video".to_owned()
    }

    fn unsupported_law_url(&self) -> String {
        "https://unsupported.test/video".to_owned()
    }

    fn law_applied_extractors(&self) -> Vec<String> {
        self.catalog
            .applications()
            .iter()
            .map(|key| key.0.clone())
            .collect()
    }
}

// A format states where its bytes rest, so locating one is reading that observation back.
impl<Syntax, Programs, Catalog, Downloads, Youtube, Manager> FormatSourceAlg<String>
    for LawEnv<Syntax, Programs, Catalog, Downloads, Youtube, Manager>
where
    Syntax: MediaSorts<Format = Format>,
{
    fn format_source(&self, format: &Format) -> Option<String> {
        format.source().map(str::to_owned)
    }
}

impl MediaProgramLawFixture for ReferenceLaws {
    fn located_law_source(&self) -> String {
        LAW_RESOURCE.to_owned()
    }

    fn law_extension(&self) -> String {
        "bin".to_owned()
    }

    fn expected_law_bytes(&self) -> Vec<u8> {
        LAW_BYTES.to_vec()
    }

    fn law_published_bytes(&self, path: &Path) -> Option<Vec<u8>> {
        self.downloads.file(path)
    }
}

impl DownloadLawFixture for ReferenceLaws {
    type Source = String;

    fn law_source(&self) -> Self::Source {
        LAW_RESOURCE.to_owned()
    }
    fn law_destination(&self) -> PathBuf {
        PathBuf::from("published.bin")
    }
    fn law_bytes(&self) -> Vec<u8> {
        LAW_BYTES.to_vec()
    }
    fn forget_law_resource(&mut self) {
        self.downloads = ReferenceDownloadEnv::default();
    }
    fn refuse_law_publication(&mut self) {
        self.downloads.refuse_publication();
    }
    fn law_progress(&self) -> Vec<u64> {
        self.downloads
            .progress()
            .iter()
            .map(|observed| observed.downloaded)
            .collect()
    }
    fn law_terminal_events(&self) -> usize {
        self.downloads.events().len()
    }
    fn law_published(&self) -> Option<Vec<u8>> {
        self.downloads.file(&self.law_destination())
    }
    fn law_abandoned(&self) -> usize {
        self.downloads.abandoned()
    }
}

// The Youtube fixtures reach this context's own seeded player, so they are stated wherever that
// interpreter is the Youtube component.
impl<Syntax, Programs, Catalog, Downloads, Manager> YoutubeChallengeLawFixture
    for LawEnv<Syntax, Programs, Catalog, Downloads, ReferenceYoutubeEnv, Manager>
{
    fn solve_law_challenge(&mut self, challenge: YoutubeChallenge, solution: &str) {
        self.youtube.solve(challenge, solution);
    }

    fn law_challenge_applications(&self) -> Vec<Vec<YoutubeChallenge>> {
        self.youtube.applications()
    }
}

impl<Syntax, Programs, Catalog, Manager> YoutubeLawFixture
    for LawEnv<Syntax, Programs, Catalog, ReferenceYoutubeDownloadEnv, ReferenceYoutubeEnv, Manager>
{
    fn law_watch_url(&self) -> String {
        REFERENCE_WATCH_URL.to_owned()
    }

    fn law_video_id(&self) -> String {
        REFERENCE_VIDEO_ID.to_owned()
    }

    fn law_watch_bytes(&self) -> Vec<u8> {
        ReferenceYoutubeEnv::watch_bytes()
    }

    fn law_player_bytes(&self) -> Vec<u8> {
        ReferenceYoutubeEnv::player_bytes()
    }

    fn law_executed_requests(&self) -> Vec<YoutubeRequest> {
        self.youtube.executed()
    }

    fn law_retrieved_requests(&self) -> Vec<YoutubeRequest> {
        self.downloads.opened()
    }
}

impl MediaProgramLawFixture for ReferenceYoutubeLaws {
    fn located_law_source(&self) -> String {
        REFERENCE_MEDIA_URL.to_owned()
    }

    fn law_extension(&self) -> String {
        REFERENCE_MEDIA_EXTENSION.to_owned()
    }

    fn expected_law_bytes(&self) -> Vec<u8> {
        LAW_BYTES.to_vec()
    }

    fn law_published_bytes(&self, path: &Path) -> Option<Vec<u8>> {
        self.downloads.file(path)
    }
}

// A Youtube format states where its bytes rest, and retrieving them is a media request.
impl<Syntax, Programs, Catalog, Downloads, Youtube, Manager> FormatSourceAlg<YoutubeRequest>
    for LawEnv<Syntax, Programs, Catalog, Downloads, Youtube, Manager>
where
    Syntax: MediaSorts<Format = Format>,
    Youtube: YoutubeSorts<Request = YoutubeRequest> + YoutubeRequestAlg,
{
    fn format_source(&self, format: &Format) -> Option<YoutubeRequest> {
        format.source().map(|url| self.youtube.media_request(url))
    }
}

impl<Syntax, Programs, Catalog, Downloads, Youtube, Manager, FetchError, Stream>
    FetchStreamAlg<YoutubeRequest>
    for LawEnv<Syntax, Programs, Catalog, Downloads, Youtube, Manager>
where
    Downloads: FetchStreamAlg<YoutubeRequest, Error = FetchError, Stream = Stream>,
{
    type Error = FetchError;
    type Stream = Stream;

    fn open_fetch(
        &self,
        request: &YoutubeRequest,
    ) -> Result<FetchStream<Self::Stream>, Self::Error> {
        self.downloads.open_fetch(request)
    }

    fn read_fetch(
        &self,
        stream: &mut Self::Stream,
        buffer: &mut [u8],
    ) -> Result<usize, Self::Error> {
        self.downloads.read_fetch(stream, buffer)
    }
}

// The downloads collection is the memory interpreter's own manager state, so it observes itself.
impl DownloadsLawFixture for MemoryManager {
    fn law_collected_sources(&self, downloads: &Self::Downloads) -> Vec<String> {
        downloads
            .queue()
            .iter()
            .map(|entry| QueueEntry::label(entry).to_owned())
            .collect()
    }
}
