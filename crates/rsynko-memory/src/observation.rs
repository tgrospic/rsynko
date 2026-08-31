//! The reified download observations and the observation stream carrying them.

use alux_sdk::trait_algebra;
use derive_new::new;
use std::path::PathBuf;

/// Defines the first-order observation stream emitted by one download.
#[trait_algebra(derive(Clone, Debug, PartialEq, Eq))]
pub trait DownloadObservation {
    /// Observes non-terminal byte progress.
    fn progress(&self, progress: DownloadProgress);

    /// Observes terminal download completion or failure.
    fn terminal(&self, event: DownloadEvent);
}

/// Denotes one monotonic observation of bytes retrieved for a destination.
#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct DownloadProgress {
    /// Names the intended final path.
    pub destination: PathBuf,
    /// Counts bytes retrieved so far.
    pub downloaded: u64,
    /// Denotes the expected complete byte count when known.
    pub total: Option<u64>,
}

/// Denotes one terminal download observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DownloadEvent {
    /// Denotes successful atomic publication.
    Succeeded {
        /// Names the published path.
        destination: PathBuf,
        /// Counts published bytes.
        bytes: u64,
    },
    /// Denotes failure before publication completed.
    Failed {
        /// Names the intended final path.
        destination: PathBuf,
        /// Preserves a stable human-readable cause.
        message: String,
    },
}
