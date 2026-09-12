use crate::*;
use alux_ext::ext;
use std::fmt::Display;
use std::path::Path;
use thiserror::Error;

/// Derives complete one-resource download meaning from three primitive capabilities.
#[ext(name = DownloadExt)]
pub impl<This, Source, FetchError, PublishError> This
where
    Source: ?Sized,
    This: FetchStreamAlg<Source, Error = FetchError>
        + AtomicPublishAlg<Error = PublishError>
        + DownloadObservationAlg
        + DownloadProgressAlg
        + DownloadReportAlg,
    FetchError: Display,
    PublishError: Display,
{
    /// Fetches, atomically publishes, and reports exactly one terminal event.
    ///
    /// # Errors
    ///
    /// Returns the fetch or publication failure after reporting it.
    fn download_resource(
        &self,
        source: &Source,
        destination: &Path,
    ) -> Result<u64, DownloadError<FetchError, PublishError>> {
        // One failure is stated wherever it arises exactly as its cause states it.
        let report_failure = |error: &dyn Display| {
            self.report_download(self.download_failed(destination, error.to_string()));
        };

        let mut opened =
            self.open_fetch(source).inspect_err(|error| report_failure(error)).map_err(DownloadError::Fetch)?;
        let mut publication = self
            .begin_publication(destination)
            .inspect_err(|error| report_failure(error))
            .map_err(DownloadError::Publish)?;

        let mut byte_count = 0_u64;
        self.report_progress(self.download_progress(destination, byte_count, opened.total));
        let mut buffer = vec![0_u8; 64 * 1024];
        // The stream ends by stating how it ended, so the publication is disposed of in one place.
        let streamed = loop {
            match self.read_fetch(&mut opened.stream, &mut buffer) {
                Ok(0) => break Ok(byte_count),
                Ok(read) => {
                    if let Err(error) = self.write_publication(&mut publication, &buffer[..read]) {
                        report_failure(&error);
                        break Err(DownloadError::Publish(error));
                    }
                    byte_count = byte_count.saturating_add(u64::try_from(read).unwrap_or(u64::MAX));
                    self.report_progress(self.download_progress(destination, byte_count, opened.total));
                }
                Err(error) => {
                    report_failure(&error);
                    break Err(DownloadError::Fetch(error));
                }
            }
        };
        let byte_count = match streamed {
            Ok(byte_count) => byte_count,
            Err(error) => {
                self.abort_publication(publication);
                return Err(error);
            }
        };

        self.commit_publication(publication)
            .inspect_err(|error| report_failure(error))
            .map_err(DownloadError::Publish)?;
        self.report_download(self.download_succeeded(destination, byte_count));
        Ok(byte_count)
    }
}

/// Denotes failure to fetch or atomically publish one resource.
#[derive(Debug, Error)]
pub enum DownloadError<FetchError, PublishError> {
    /// Denotes resource retrieval failure.
    #[error("resource fetch failed: {0}")]
    Fetch(FetchError),
    /// Denotes atomic publication failure.
    #[error("atomic publication failed: {0}")]
    Publish(PublishError),
}
