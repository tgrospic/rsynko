#![doc = include_str!("../README.md")]

mod download;
mod extraction;
mod laws;
mod manager;
mod observation;
mod options;
mod processing;
mod screen;
mod selection;
mod session;
mod sync;
mod syntax;
mod term;
mod x;
mod youtube;

pub use download::{ReferenceDownloadEnv, ReferenceFetchError, ReferencePublishError};
pub use extraction::{
    ReferenceExtractionError, ReferenceExtractionOutcome, ReferenceExtractor,
    ReferenceExtractorRegistry,
};
pub use laws::{ReferenceLaws, ReferenceProcessingRefusal, ReferenceYoutubeLaws};
pub use manager::{
    ManagerState, MemoryManager, PlannedChange, QueueEntry, QueueId, Rehearsal, SourceRequest,
    TransferState,
};
pub use observation::{
    DownloadEvent, DownloadObservation, DownloadObservationInterpreter, DownloadObservationOp,
    DownloadObservationReply, DownloadProgress,
};
pub use options::{DownloadOptions, FormatCatalog, FormatChoice, role_accepts};
pub use processing::{
    ArtifactSet, ReferenceArtifactTransform, ReferenceProcessingError, ReferenceProcessor,
    ReferenceProcessorEnv,
};
pub use rsynko_manager::ChangeKind;
pub use screen::TextScreen;
pub use selection::ReferenceFormatSelector;
pub use session::{ReferenceRun, ReferenceSession};
pub use sync::{
    ReferenceSyncEnv, ReferenceSyncError, RsyncEndpoint, RsyncSyntax, SyncCommand, SyncObservation,
};
pub use syntax::{DownloadSyntax, MediaSyntax, ProcessingSyntax};
pub use term::{
    Artifact, Collection, Extraction, ExtractorKey, Format, FormatPredicate, FormatSelection,
    InfoRecord, InfoValue, Media, ProcessingProgram, ProcessingStep, ProcessorId, UrlReference,
    interpret_selection, predicate_accepts,
};
pub use x::{ReferenceXEnv, XAttachment};
pub use youtube::{
    REFERENCE_MEDIA_EXTENSION, REFERENCE_MEDIA_URL, REFERENCE_PLAYER_PROGRAM,
    REFERENCE_PLAYER_PROGRAM_URL, REFERENCE_VIDEO_ID, REFERENCE_WATCH_URL,
    ReferenceYoutubeDownloadEnv, ReferenceYoutubeEnv, YoutubeRequest, YoutubeRequestSyntax,
};
