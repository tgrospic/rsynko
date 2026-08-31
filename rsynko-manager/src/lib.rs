#![doc = include_str!("../README.md")]

mod attention;
mod downloads;
mod draft;
mod intent;
mod laws;
mod log;
mod menu;
mod navigation;
mod options;
mod queue;
mod rehearsal;
mod sorts;
mod text;
mod transfer;
mod transitions;

pub use attention::{ManagerAttentionExt, detail_text, summary_line};
pub use downloads::{
    DownloadCollectionAlg, DownloadsAlg, DownloadsExt, MediaOptionsAlg, OutputChoiceAlg,
    ProgressiveDownloadsExt, SourceRecognitionAlg, SourceRequestAlg, SubmissionAlg,
};
pub use draft::{DraftStateAlg, InputDraftAlg, OutputDraftAlg};
pub use intent::{
    Breadcrumb, DetailControl, ManagerIntent, ManagerIntentInterpreter, ManagerIntentOp,
    ManagerIntentReply, ManagerPage, SpaceAction, TransferObservation,
    TransferObservationInterpreter, TransferObservationOp, TransferObservationReply,
};
pub use laws::{
    AttentionLaws, DownloadsLawFixture, DownloadsLaws, DraftLaws, IntentLaws, LogLaws,
    ManagerLawAuthoring, MenuLaws, NavigationLaws, OptionsLaws, QueueLaws, RehearsalLawFixture,
    RehearsalLaws, SubmissionLawFixture, SubmissionLaws, TextLaws, TransitionLaws,
};
pub use log::DownloadLogAlg;
pub use menu::{ActionAvailability, ManagerAction, ManagerMenuExt};
pub use navigation::{
    BreadcrumbExt, COLLECTION, DetailSelectionAlg, ManagerStatusAlg, NavigationStateAlg,
    SafeExitAlg,
};
pub use options::{
    FormatDiscovery, FormatDiscoveryInterpreter, FormatDiscoveryOp, FormatDiscoveryReply,
    MediaStreams, MediaStreamsExt, Performer, RequestOptionsAlg, RequestSelectionExt,
};
pub use rehearsal::{
    ChangeKind, PlannedChangeAlg, QueueDryRunAlg, RehearsalObservation,
    RehearsalObservationInterpreter, RehearsalObservationOp, RehearsalObservationReply,
    RehearsalState, RehearsalStateAlg, RehearsalViewAlg,
};

pub use queue::{
    FormatCatalogStateAlg, OutputNaming, QueueAppendAlg, QueueCatalogAlg, QueueDuplicateAlg,
    QueueEntryAlg, QueueFormatEditAlg, QueueOutputAlg, QueuePauseAlg, QueueRemoveAlg,
    QueueSourceAlg, SourceMetadataAlg, TransferStateAlg,
};
pub use sorts::ManagerSorts;
pub use text::{TextCursorExt, TextEditorStateAlg};
pub use transfer::{TransferPhase, TransferProgressExt, TransferViewAlg};
pub use transitions::{
    ChoiceEditingExt, CollectionNavigationExt, DetailNavigationExt, DraftSubmissionExt,
    ManagerIntentExt, OutputEditingExt, QueueLifecycleExt,
};

// Ambassador's generated delegation macros, re-exported so interpreters can compose.
pub use downloads::{
    ambassador_impl_MediaOptionsAlg, ambassador_impl_OutputChoiceAlg,
    ambassador_impl_SourceRecognitionAlg, ambassador_impl_SourceRequestAlg,
    ambassador_impl_SubmissionAlg,
};
pub use draft::{
    ambassador_impl_DraftStateAlg, ambassador_impl_InputDraftAlg, ambassador_impl_OutputDraftAlg,
};
pub use log::ambassador_impl_DownloadLogAlg;
pub use navigation::{
    ambassador_impl_DetailSelectionAlg, ambassador_impl_ManagerStatusAlg,
    ambassador_impl_NavigationStateAlg, ambassador_impl_SafeExitAlg,
};
pub use queue::{
    ambassador_impl_FormatCatalogStateAlg, ambassador_impl_QueueAppendAlg,
    ambassador_impl_QueueCatalogAlg, ambassador_impl_QueueDuplicateAlg,
    ambassador_impl_QueueFormatEditAlg, ambassador_impl_QueueOutputAlg,
    ambassador_impl_QueuePauseAlg, ambassador_impl_QueueRemoveAlg, ambassador_impl_QueueSourceAlg,
    ambassador_impl_SourceMetadataAlg, ambassador_impl_TransferStateAlg,
};
pub use rehearsal::{ambassador_impl_QueueDryRunAlg, ambassador_impl_RehearsalStateAlg};
pub use text::ambassador_impl_TextEditorStateAlg;
