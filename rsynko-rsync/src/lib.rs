#![doc = include_str!("../README.md")]

mod command;
mod endpoint;
mod laws;
mod observation;
mod program;
mod sorts;

pub use command::sync_profile;
pub use command::{
    SYNC_MIRROR, SYNC_OUT_FORMAT, SYNC_PROGRAM, SYNC_REHEARSAL, SyncCommandAlg, SyncCommandExt, SyncCommandViewAlg,
    SyncMode, SyncProfile,
};
pub use endpoint::{RSYNC_SCHEME, RSYNC_WORD, RsyncEndpointAlg, RsyncEndpointExt, RsyncEndpointViewAlg};
pub use laws::{SyncLawFixture, SyncLaws};
pub use observation::{
    DELETION_MARK, FIELD_MARK, SyncChangeAlg, SyncObservationAlg, SyncObservationViewAlg, SyncReadExt,
};
pub use program::{SyncProgramExt, SyncRunAlg, SyncWatchAlg};
pub use sorts::RsyncSorts;

// Ambassador's generated delegation macros, re-exported so interpreters can compose.
pub use command::{ambassador_impl_SyncCommandAlg, ambassador_impl_SyncCommandViewAlg};
pub use endpoint::{ambassador_impl_RsyncEndpointAlg, ambassador_impl_RsyncEndpointViewAlg};
pub use observation::{
    ambassador_impl_SyncChangeAlg, ambassador_impl_SyncObservationAlg, ambassador_impl_SyncObservationViewAlg,
};
pub use program::{ambassador_impl_SyncRunAlg, ambassador_impl_SyncWatchAlg};
