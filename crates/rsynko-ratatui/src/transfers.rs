use crate::clock::Monotonic;
use ambassador::Delegate;
use rsynko_manager::*;
use rsynko_memory::{ManagerState, PlannedChange, QueueId, SyncObservation};
use rsynko_process::{HOLDING_IS_POSSIBLE, ProcessHold, ProcessSyncEnv};
use rsynko_rsync::SyncProgramExt;
use rsynko_session::*;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Attends to folder transfers performed by another program.
#[derive(Debug, Delegate)]
#[delegate(ClockAlg, target = "clock")]
pub struct Transfers<'a> {
    manager: &'a mut ManagerState,
    clock: Monotonic,
}

/// Carries one running transfer, what it has written, and the handle that holds it still.
#[derive(Debug)]
pub struct TransferRun {
    worker: JoinHandle<Result<Vec<PlannedChange>, String>>,
    observations: Receiver<SyncObservation>,
    hold: ProcessHold,
}

impl<'a> Transfers<'a> {
    /// Attends to the folder transfers this collection asks for.
    pub const fn attending(manager: &'a mut ManagerState) -> Self {
        Self {
            manager,
            clock: Monotonic,
        }
    }

    /// Observes where one request states its transfer comes to rest.
    fn destination(&self, id: QueueId) -> Option<PathBuf> {
        self.manager
            .queue_entry(id)
            .and_then(QueueEntryAlg::output)
            .map(Path::to_owned)
    }
}

impl SessionSorts for Transfers<'_> {
    type Id = QueueId;
    type Run = TransferRun;
    type Report = SyncObservation;
    type Ending = Vec<PlannedChange>;
    type Refusal = String;
}

impl UndertakingAlg for Transfers<'_> {
    fn unattended(&self) -> Vec<QueueId> {
        self.manager
            .wanting_work()
            .into_iter()
            .filter(|id| {
                self.manager
                    .queue_entry(*id)
                    .is_some_and(|entry| entry.performer() == Performer::Program)
            })
            .collect()
    }

    fn begin(&self, id: &QueueId) -> Result<TransferRun, String> {
        let entry = self
            .manager
            .queue_entry(*id)
            .ok_or_else(|| "the request is gone".to_owned())?;
        if entry.output().is_none() {
            return Err("the transfer names no destination".to_owned());
        }
        // The command is the one the request states, and not one built again here: a reader is
        // shown that command, and running a different one would make what is shown a lie.
        let command = entry
            .transfer_command()
            .ok_or_else(|| "the transfer states no command to run".to_owned())?;
        let (sender, observations) = channel();
        let hold = ProcessHold::default();
        let environment = ProcessSyncEnv::held(sender, hold.clone());
        let worker = thread::spawn(move || {
            environment
                .run_sync(&command)
                .map_err(|error| error.to_string())
        });
        Ok(TransferRun {
            worker,
            observations,
            hold,
        })
    }
}

impl RunReadAlg for Transfers<'_> {
    fn run_is_over(&self, run: &TransferRun) -> bool {
        run.worker.is_finished()
    }

    fn read_run(&self, run: &mut TransferRun) -> Vec<SyncObservation> {
        run.observations.try_iter().collect()
    }

    fn end_run(&self, run: TransferRun) -> Result<Vec<PlannedChange>, String> {
        match run.worker.join() {
            Ok(outcome) => outcome,
            Err(_) => Err("the folder transfer stopped without saying why".to_owned()),
        }
    }
}

impl RunHoldAlg for Transfers<'_> {
    fn holding_is_possible(&self) -> bool {
        // A transfer performed by another program is held still by signalling it, which not
        // every machine can do.
        HOLDING_IS_POSSIBLE
    }

    fn hold_run(&self, run: &mut TransferRun, held: bool) {
        let _signalled = if held {
            run.hold.hold()
        } else {
            run.hold.release()
        };
    }

    fn abandon_run(&self, run: &mut TransferRun) {
        let _ended = run.hold.cancel();
    }
}

impl AttentionAlg for Transfers<'_> {
    fn begun(&mut self, id: &QueueId, holdable: bool) {
        self.manager.begun(*id, holdable);
    }

    fn heard(&mut self, id: &QueueId, report: SyncObservation) {
        let SyncObservation::Progress {
            transferred,
            percent,
        } = report
        else {
            return;
        };
        // A rehearsal moves nothing, so its progress states nothing about a transfer.
        if self.manager.rehearses(*id) {
            return;
        }
        let Some(destination) = self.destination(*id) else {
            return;
        };
        self.manager.apply_transfer_event(
            *id,
            TransferObservationOp::Progress {
                destination,
                downloaded: transferred,
                total: (percent > 0).then(|| transferred.saturating_mul(100) / u64::from(percent)),
            },
        );
    }

    fn ran_for(&mut self, id: &QueueId, elapsed: Duration) {
        self.manager.ran_for(*id, elapsed);
    }

    fn ended(&mut self, id: &QueueId, ending: Result<Vec<PlannedChange>, String>) {
        match ending {
            Ok(changes) => {
                let destination = self.destination(*id).unwrap_or_default();
                self.manager.performed(*id, destination, changes);
            }
            Err(detail) => self.manager.refused(*id, &detail),
        }
    }

    fn wanted(&self, id: &QueueId) -> Wanted {
        self.manager.wanted(*id)
    }
}
