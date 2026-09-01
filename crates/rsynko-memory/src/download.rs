use crate::DownloadSyntax;
use crate::{DownloadEvent, DownloadProgress};
use rsynko_download::*;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Denotes retrieval failure in the deterministic download interpreter.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReferenceFetchError {
    /// Denotes a resource absent from the reference environment.
    #[error("unknown resource {0}")]
    UnknownResource(String),
}

/// Denotes publication failure in the deterministic download interpreter.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("publication refused")]
pub struct ReferencePublishError;

/// Interprets fetching, atomic publication, and reporting entirely in memory.
#[derive(Debug, Default)]
pub struct ReferenceDownloadEnv {
    resources: BTreeMap<String, Vec<u8>>,
    refuse_publication: bool,
    files: RefCell<BTreeMap<PathBuf, Vec<u8>>>,
    abandoned: Cell<usize>,
    events: RefCell<Vec<DownloadEvent>>,
    progress: RefCell<Vec<DownloadProgress>>,
}

impl ReferenceDownloadEnv {
    /// Associates exact bytes with one resource URL.
    pub fn register_resource(&mut self, url: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Option<Vec<u8>> {
        self.resources.insert(url.into(), bytes.into())
    }

    /// Refuses every subsequent publication, so failure laws can be exercised.
    pub fn refuse_publication(&mut self) {
        self.refuse_publication = true;
    }

    /// Counts publications abandoned without reaching their final path.
    #[must_use]
    pub fn abandoned(&self) -> usize {
        self.abandoned.get()
    }

    /// Observes bytes published at one final path.
    #[must_use]
    pub fn file(&self, path: &Path) -> Option<Vec<u8>> {
        self.files.borrow().get(path).cloned()
    }

    /// Observes terminal events in emission order.
    #[must_use]
    pub fn events(&self) -> Vec<DownloadEvent> {
        self.events.borrow().clone()
    }

    /// Observes byte-progress reports in emission order.
    #[must_use]
    pub fn progress(&self) -> Vec<DownloadProgress> {
        self.progress.borrow().clone()
    }
}

impl FetchStreamAlg for ReferenceDownloadEnv {
    type Error = ReferenceFetchError;
    type Stream = Cursor<Vec<u8>>;

    fn open_fetch(&self, url: &str) -> Result<FetchStream<Self::Stream>, Self::Error> {
        let bytes =
            self.resources.get(url).cloned().ok_or_else(|| ReferenceFetchError::UnknownResource(url.to_owned()))?;
        let total = u64::try_from(bytes.len()).ok();
        Ok(FetchStream::new(Cursor::new(bytes), total))
    }

    fn read_fetch(&self, stream: &mut Self::Stream, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(stream.read(buffer).unwrap_or(0))
    }
}

impl AtomicPublishAlg for ReferenceDownloadEnv {
    type Error = ReferencePublishError;
    type Publication = (PathBuf, Vec<u8>);

    fn begin_publication(&self, destination: &Path) -> Result<Self::Publication, Self::Error> {
        Ok((destination.to_owned(), Vec::default()))
    }

    fn write_publication(&self, publication: &mut Self::Publication, bytes: &[u8]) -> Result<(), Self::Error> {
        if self.refuse_publication {
            return Err(ReferencePublishError);
        }
        publication.1.extend_from_slice(bytes);
        Ok(())
    }

    fn commit_publication(&self, publication: Self::Publication) -> Result<(), Self::Error> {
        self.files.borrow_mut().insert(publication.0, publication.1);
        Ok(())
    }

    fn abort_publication(&self, _: Self::Publication) {
        self.abandoned.set(self.abandoned.get() + 1);
    }
}

impl DownloadReportAlg for ReferenceDownloadEnv {
    type Event = DownloadEvent;

    fn report_download(&self, event: DownloadEvent) {
        self.events.borrow_mut().push(event);
    }
}

impl DownloadProgressAlg for ReferenceDownloadEnv {
    type Progress = DownloadProgress;

    fn report_progress(&self, progress: DownloadProgress) {
        self.progress.borrow_mut().push(progress);
    }
}

impl DownloadObservationAlg for ReferenceDownloadEnv {
    type Event = DownloadEvent;
    type Progress = DownloadProgress;

    fn download_progress(&self, destination: &Path, downloaded: u64, total: Option<u64>) -> Self::Progress {
        DownloadSyntax.download_progress(destination, downloaded, total)
    }

    fn download_succeeded(&self, destination: &Path, bytes: u64) -> Self::Event {
        DownloadSyntax.download_succeeded(destination, bytes)
    }

    fn download_failed(&self, destination: &Path, message: String) -> Self::Event {
        DownloadSyntax.download_failed(destination, message)
    }
}
