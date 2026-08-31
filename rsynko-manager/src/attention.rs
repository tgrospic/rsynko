use crate::*;
use alux_ext::ext;
use rsynko_session::Wanted;
use std::path::PathBuf;
use std::time::Duration;

/// States how much of a failure a reader is told about at a glance.
const SUMMARY_COLUMNS: usize = 160;

/// States how much of a failure is kept for whoever goes looking.
const DETAIL_COLUMNS: usize = 500;

/// Derives what beginning, running, refusing, and finishing one run states to the collection.
///
/// A run is attended to by what the request wants of it, and a request wanting nothing is one
/// that was removed, or one whose reader is leaving. Which kind of run a request asks for is
/// something the request already states, so nobody attending to it has to carry that around.
#[ext(name = ManagerAttentionExt)]
pub impl<This> This
where
    This: QueueCatalogAlg + SafeExitAlg + TransferStateAlg + RehearsalStateAlg,
    This::Entry: QueueEntryAlg,
    This::Change: PlannedChangeAlg,
    This::Id: Copy + Eq,
{
    /// States every request waiting for an interpreter to do what it asks, in collection order.
    fn wanting_work(&self) -> Vec<This::Id> {
        self.queue_ids()
            .filter(|id| self.entry_phase(*id) == Some(TransferPhase::Waiting))
            .collect()
    }

    /// States what one request wants of the run working on its behalf.
    fn wanted(&self, id: This::Id) -> Wanted {
        // Leaving is wanting nothing: every run ends the way a removed request's run ends.
        if self.exit_requested() {
            return Wanted::Unwanted;
        }
        match self.entry_phase(id) {
            None => Wanted::Unwanted,
            Some(TransferPhase::Paused) => Wanted::Held,
            Some(_) => Wanted::Running,
        }
    }

    /// States whether one request asks for a rehearsal rather than for the thing itself.
    fn rehearses(&self, id: This::Id) -> bool {
        self.queue_entry(id).and_then(QueueEntryAlg::dry_run) == Some(true)
    }

    /// States that a run has begun, as the kind of run the request asks for.
    ///
    /// A rehearsal moves nothing and is over when it has said what it would do, so it is never
    /// told that it could be held still.
    fn begun(&mut self, id: This::Id, holdable: bool) {
        if self.rehearses(id) {
            self.apply_rehearsal_event(id, RehearsalObservationOp::Started {});
            return;
        }
        self.apply_transfer_event(id, TransferObservationOp::Started {});
        self.apply_transfer_event(
            id,
            TransferObservationOp::PauseCapability {
                supported: holdable,
            },
        );
    }

    /// States how long one run has been running, when the request counts that.
    ///
    /// A rehearsal moves nothing, so how long it took states nothing about a transfer.
    fn ran_for(&mut self, id: This::Id, elapsed: Duration) {
        if !self.rehearses(id) {
            self.apply_transfer_event(id, TransferObservationOp::Elapsed { elapsed });
        }
    }

    /// States that one run refused, as the kind of run the request asks for.
    fn refused(&mut self, id: This::Id, detail: &str) {
        if self.rehearses(id) {
            self.apply_rehearsal_event(
                id,
                RehearsalObservationOp::Failed {
                    message: summary_line(detail),
                },
            );
            return;
        }
        self.apply_transfer_event(
            id,
            TransferObservationOp::ProgramFailed {
                summary: summary_line(detail),
                detail: detail_text(detail),
            },
        );
    }

    /// States what one finished run did, as the kind of run the request asks for.
    ///
    /// A rehearsal states what it would have changed. A transfer states that it is done, and how
    /// much it moved, which is what its changes add up to.
    fn performed(&mut self, id: This::Id, destination: PathBuf, changes: Vec<This::Change>) {
        if self.rehearses(id) {
            self.apply_rehearsal_event(id, RehearsalObservationOp::Reported { changes });
            return;
        }
        let bytes = changes
            .iter()
            .filter_map(PlannedChangeAlg::change_size)
            .fold(0_u64, u64::saturating_add);
        self.apply_transfer_event(id, TransferObservationOp::Completed { destination, bytes });
    }

    /// States whether another request is already writing where this one would write.
    ///
    /// Two runs writing one path would each publish over the other, and which of them a reader
    /// ends up with is decided by whichever finished last. The collection knows both, so it is
    /// the collection that states the clash rather than whoever happens to be running them.
    fn destination_is_taken(&self, id: This::Id) -> bool {
        let Some(destination) = self.queue_entry(id).and_then(QueueEntryAlg::output) else {
            return false;
        };
        self.queue_ids()
            .filter(|other| *other != id)
            .filter(|other| {
                self.entry_phase(*other)
                    .is_some_and(TransferPhase::is_running)
            })
            .any(|other| {
                self.queue_entry(other).and_then(QueueEntryAlg::output) == Some(destination)
            })
    }

    /// Observes the phase one request has reached, while the collection still holds it.
    fn entry_phase(&self, id: This::Id) -> Option<TransferPhase> {
        self.queue_entry(id).map(QueueEntryAlg::phase)
    }
}

/// States the one line a reader is shown about a failure.
pub fn summary_line(detail: &str) -> String {
    detail
        .lines()
        .next()
        .unwrap_or_default()
        .chars()
        .take(SUMMARY_COLUMNS)
        .collect()
}

/// States what is kept about a failure for whoever goes looking for it.
pub fn detail_text(detail: &str) -> String {
    detail.chars().take(DETAIL_COLUMNS).collect()
}
