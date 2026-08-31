//! Law scenarios for one-resource download, stated once over the capabilities.
//!
//! A scenario names the situation it checks and drives it itself. Retrieval state and the recorded
//! trace are things only an interpreter holds, so it supplies them through [`DownloadLawFixture`]
//! while the scenario still authors every law.

use crate::*;
use alux_ext::ext;
use anyhow::{Result, bail};
use std::fmt::Display;
use std::path::PathBuf;

/// Supplies the retrieval state and recorded trace a download scenario cannot author for itself.
pub trait DownloadLawFixture {
    /// Represents the source the scenarios retrieve.
    type Source;

    /// Names the source the scenarios retrieve.
    fn law_source(&self) -> Self::Source;
    /// Names the path the scenarios publish at.
    fn law_destination(&self) -> PathBuf;
    /// States the exact bytes the source denotes.
    fn law_bytes(&self) -> Vec<u8>;
    /// Forgets the resource, so retrieval fails before publication begins.
    fn forget_law_resource(&mut self);
    /// Refuses publication, so an begun publication fails.
    fn refuse_law_publication(&mut self);
    /// Observes recorded byte counts in emission order.
    fn law_progress(&self) -> Vec<u64>;
    /// Counts recorded terminal events.
    fn law_terminal_events(&self) -> usize;
    /// Observes bytes published at the scenario destination.
    fn law_published(&self) -> Option<Vec<u8>>;
    /// Counts publications abandoned without reaching their final path.
    fn law_abandoned(&self) -> usize;
}

/// Authors the one-resource download laws.
#[ext(name = DownloadLaws)]
pub impl<This, Source, Event, Progress> This
where
    This: DownloadLawFixture<Source = Source>
        + FetchStreamAlg<Source, Error: Display>
        + AtomicPublishAlg<Error: Display>
        + DownloadObservationAlg<Event = Event, Progress = Progress>
        + DownloadProgressAlg<Progress = Progress>
        + DownloadReportAlg<Event = Event>,
{
    /// Checks that a successful download publishes exactly the bytes it reported.
    ///
    /// The laws checked are: progress begins at zero and is monotonic; exactly one terminal event
    /// is emitted; publication preserves the fetched bytes exactly.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn download_laws(&mut self) -> Result<()> {
        let expected = self.law_bytes();
        let reported = match self.download_resource(&self.law_source(), &self.law_destination()) {
            Ok(bytes) => bytes,
            Err(error) => bail!("download failed: {error}"),
        };
        let stated = u64::try_from(expected.len()).unwrap_or(u64::MAX);
        if reported != stated {
            bail!("download reported {reported} bytes, expected {stated}");
        }
        self.check_progress_laws()?;
        if self.law_terminal_events() != 1 {
            bail!(
                "expected exactly one terminal event, observed {}",
                self.law_terminal_events()
            );
        }
        match self.law_published() {
            Some(bytes) if bytes == expected => Ok(()),
            Some(_) => bail!("publication did not preserve the fetched bytes exactly"),
            None => bail!("a successful download published nothing"),
        }
    }

    /// Checks that a retrieval failure publishes nothing and still reports exactly once.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn download_fetch_failure_laws(&mut self) -> Result<()> {
        self.forget_law_resource();
        if self
            .download_resource(&self.law_source(), &self.law_destination())
            .is_ok()
        {
            bail!("an absent resource downloaded successfully");
        }
        self.check_failure_laws(0)
    }

    /// Checks that a publication failure abandons its partial state and reports exactly once.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn download_publication_failure_laws(&mut self) -> Result<()> {
        self.refuse_law_publication();
        if self
            .download_resource(&self.law_source(), &self.law_destination())
            .is_ok()
        {
            bail!("a refused publication downloaded successfully");
        }
        self.check_failure_laws(1)
    }

    /// Checks that recorded progress begins at zero and never decreases.
    ///
    /// # Errors
    ///
    /// Returns the violated law.
    fn check_progress_laws(&self) -> Result<()> {
        let progress = self.law_progress();
        if let Some(first) = progress.first()
            && *first != 0
        {
            bail!("progress began at {first} rather than zero");
        }
        if progress.windows(2).any(|pair| pair[1] < pair[0]) {
            bail!("retrieved byte counts are not monotonic: {progress:?}");
        }
        Ok(())
    }

    /// Checks the laws every failed execution shares.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn check_failure_laws(&self, abandoned: usize) -> Result<()> {
        if self.law_terminal_events() != 1 {
            bail!(
                "expected exactly one terminal event, observed {}",
                self.law_terminal_events()
            );
        }
        if self.law_published().is_some() {
            bail!("a failed execution published bytes at its final path");
        }
        if self.law_abandoned() != abandoned {
            bail!(
                "a failed publication was abandoned {} times, expected {abandoned}",
                self.law_abandoned()
            );
        }
        self.check_progress_laws()
    }
}
