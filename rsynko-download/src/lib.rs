#![doc = include_str!("../README.md")]

mod fetch;
mod laws;
mod observation;
mod program;
mod publication;

pub use fetch::{FetchStream, FetchStreamAlg};
pub use laws::{DownloadLawFixture, DownloadLaws};
pub use observation::{DownloadObservationAlg, DownloadProgressAlg, DownloadReportAlg};
pub use program::{DownloadError, DownloadExt};
pub use publication::AtomicPublishAlg;

// Ambassador's generated delegation macros, re-exported so interpreters can compose.
pub use fetch::ambassador_impl_FetchStreamAlg;
pub use observation::ambassador_impl_DownloadObservationAlg;
pub use observation::ambassador_impl_DownloadProgressAlg;
pub use observation::ambassador_impl_DownloadReportAlg;
pub use publication::ambassador_impl_AtomicPublishAlg;
