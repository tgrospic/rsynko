use alux_ext::ext;
use std::path::Path;
use std::time::Duration;

/// Specifies what one transfer states about its own progress.
pub trait TransferViewAlg {
    /// Observes retrieved bytes.
    fn transferred(&self) -> u64;
    /// Observes expected bytes when the source states them.
    fn transfer_total(&self) -> Option<u64>;
    /// Observes elapsed transfer time.
    fn transfer_elapsed(&self) -> Duration;
    /// Observes whether the transfer reached terminal success.
    fn transfer_complete(&self) -> bool;
    /// Observes the path publication will produce once an interpreter states one.
    fn transfer_destination(&self) -> Option<&Path>;
    /// Observes the concise reason a transfer failed.
    fn transfer_summary(&self) -> Option<&str>;
    /// Observes the diagnostic accompanying a failure.
    fn transfer_detail(&self) -> Option<&str>;
}

/// Derives rate, remaining time, and completed share from observed bytes and elapsed time.
#[ext(name = TransferProgressExt)]
pub impl<This> This
where
    This: TransferViewAlg,
{
    /// Derives average retrieved bytes per second once progress has duration.
    fn bytes_per_second(&self) -> Option<u64> {
        let elapsed = self.transfer_elapsed().as_nanos();
        let transferred = self.transferred();
        if transferred == 0 || elapsed == 0 {
            return None;
        }
        let rate = u128::from(transferred).saturating_mul(1_000_000_000) / elapsed;
        Some(u64::try_from(rate.max(1)).unwrap_or(u64::MAX))
    }

    /// Derives remaining time from average progress once the expected byte count is known.
    fn estimated_remaining(&self) -> Option<Duration> {
        let total = self.transfer_total()?;
        let elapsed = self.transfer_elapsed().as_nanos();
        let transferred = self.transferred();
        if transferred == 0 || elapsed == 0 {
            return None;
        }
        let remaining = total.saturating_sub(transferred);
        let nanos = u128::from(remaining).saturating_mul(elapsed) / u128::from(transferred);
        Some(Duration::from_nanos(
            u64::try_from(nanos).unwrap_or(u64::MAX),
        ))
    }

    /// Derives the completed share as whole percent once it is known.
    fn percent(&self) -> Option<u16> {
        if self.transfer_complete() {
            return Some(100);
        }
        let total = self.transfer_total()?;
        if total == 0 {
            return Some(0);
        }
        let percent = u128::from(self.transferred()).saturating_mul(100) / u128::from(total);
        Some(u16::try_from(percent.min(100)).unwrap_or(100))
    }
}

/// Denotes the lifecycle phase of one queued transfer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferPhase {
    /// Denotes an entry ready to be scheduled.
    Ready,
    /// Denotes an enabled entry waiting for an interpreter to schedule it.
    Waiting,
    /// Denotes an interpreter stating what the request would do instead of doing it.
    Rehearsing,
    /// Denotes extraction and format selection.
    Extracting,
    /// Denotes incremental resource retrieval.
    Downloading,
    /// Denotes a cooperatively suspended active transfer.
    Paused,
    /// Denotes final atomic publication.
    Publishing,
    /// Denotes successful terminal completion.
    Complete,
    /// Denotes terminal failure.
    Failed,
}

impl TransferPhase {
    /// States every phase one transfer passes through, in lifecycle order.
    pub const LIFECYCLE: [Self; 9] = [
        Self::Ready,
        Self::Waiting,
        Self::Rehearsing,
        Self::Extracting,
        Self::Downloading,
        Self::Paused,
        Self::Publishing,
        Self::Complete,
        Self::Failed,
    ];

    /// States whether a run is happening on the request's behalf while it is in this phase.
    ///
    /// A request that has not been asked for, and one that is over, are both unattended: nothing
    /// is working on their behalf, and nothing has to be told they were removed.
    pub const fn is_running(self) -> bool {
        matches!(
            self,
            Self::Rehearsing
                | Self::Extracting
                | Self::Downloading
                | Self::Paused
                | Self::Publishing
        )
    }
}
