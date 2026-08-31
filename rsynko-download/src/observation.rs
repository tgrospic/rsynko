use ambassador::delegatable_trait;
use std::path::Path;

/// Specifies observation of terminal download events.
#[delegatable_trait]
pub trait DownloadReportAlg {
    /// Represents one terminal download event.
    type Event;

    /// Reports one terminal event.
    fn report_download(&self, event: Self::Event);
}

/// Specifies observation of non-terminal download progress.
#[delegatable_trait]
pub trait DownloadProgressAlg {
    /// Represents one progress observation.
    type Progress;

    /// Reports one monotonic byte-progress observation.
    fn report_progress(&self, progress: Self::Progress);
}

/// Provides the carriers and constructors for download observations.
#[delegatable_trait]
pub trait DownloadObservationAlg {
    /// Represents one terminal event.
    type Event;
    /// Represents one progress observation.
    type Progress;

    /// Defines a byte-progress observation.
    fn download_progress(
        &self,
        destination: &Path,
        downloaded: u64,
        total: Option<u64>,
    ) -> Self::Progress;
    /// Defines terminal success.
    fn download_succeeded(&self, destination: &Path, bytes: u64) -> Self::Event;
    /// Defines terminal failure.
    fn download_failed(&self, destination: &Path, message: String) -> Self::Event;
}
