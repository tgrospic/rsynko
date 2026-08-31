use crate::*;
use alux_ext::ext;
use std::fmt::Display;
use std::path::Path;
use thiserror::Error;

/// Derives complete one-resource download meaning from three primitive capabilities.
#[ext(name = DownloadExt)]
pub impl<This, Source, FetchError, PublishError, Event, Progress> This
where
    Source: ?Sized,
    This: FetchStreamAlg<Source, Error = FetchError>
        + AtomicPublishAlg<Error = PublishError>
        + DownloadObservationAlg<Event = Event, Progress = Progress>
        + DownloadProgressAlg<Progress = Progress>
        + DownloadReportAlg<Event = Event>,
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
        let mut opened = match self.open_fetch(source) {
            Ok(opened) => opened,
            Err(error) => {
                self.report_download(self.download_failed(destination, error.to_string()));
                return Err(DownloadError::Fetch(error));
            }
        };
        let mut publication = match self.begin_publication(destination) {
            Ok(publication) => publication,
            Err(error) => {
                self.report_download(self.download_failed(destination, error.to_string()));
                return Err(DownloadError::Publish(error));
            }
        };
        let mut byte_count = 0_u64;
        self.report_progress(self.download_progress(destination, byte_count, opened.total));
        let mut buffer = vec![0_u8; 64 * 1024];
        loop {
            let read = match self.read_fetch(&mut opened.stream, &mut buffer) {
                Ok(read) => read,
                Err(error) => {
                    self.abort_publication(publication);
                    self.report_download(self.download_failed(destination, error.to_string()));
                    return Err(DownloadError::Fetch(error));
                }
            };
            if read == 0 {
                break;
            }
            if let Err(error) = self.write_publication(&mut publication, &buffer[..read]) {
                self.abort_publication(publication);
                self.report_download(self.download_failed(destination, error.to_string()));
                return Err(DownloadError::Publish(error));
            }
            byte_count = byte_count.saturating_add(u64::try_from(read).unwrap_or(u64::MAX));
            self.report_progress(self.download_progress(destination, byte_count, opened.total));
        }
        if let Err(error) = self.commit_publication(publication) {
            self.report_download(self.download_failed(destination, error.to_string()));
            return Err(DownloadError::Publish(error));
        }
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
