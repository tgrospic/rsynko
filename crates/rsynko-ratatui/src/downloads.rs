use crate::clock::Monotonic;
use ambassador::Delegate;
use rsynko_manager::*;
use rsynko_media::{ApplicationExt, OutputTarget};
use rsynko_memory::{
    DownloadEvent, DownloadObservationInterpreter, DownloadProgress, ManagerState, MediaSyntax,
    QueueId,
};
use rsynko_reqwest::{
    RuntimeEnvironment, RuntimeObservation, RuntimeObservationReceiver, RuntimePause,
    runtime_observation_channel,
};
use rsynko_session::*;
use rsynko_yt::{YoutubeApplicationExt, media_failure, youtube_id};
use std::path::PathBuf;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Attends to resource downloads this program retrieves itself.
#[derive(Debug, Delegate)]
#[delegate(ClockAlg, target = "clock")]
pub struct Downloads<'a> {
    manager: &'a mut ManagerState,
    clock: Monotonic,
}

/// Carries one running download, what it has stated, and the handle that suspends it.
pub struct DownloadRun {
    worker: JoinHandle<Result<PathBuf, String>>,
    observations: RuntimeObservationReceiver,
    pause: RuntimePause,
}

/// Applies what one running download states to the request it retrieves.
struct DownloadObservation<'a> {
    manager: &'a mut ManagerState,
    id: QueueId,
}

impl<'a> Downloads<'a> {
    /// Attends to the downloads this collection asks for.
    pub const fn attending(manager: &'a mut ManagerState) -> Self {
        Self {
            manager,
            clock: Monotonic,
        }
    }
}

impl SessionSorts for Downloads<'_> {
    type Id = QueueId;
    type Run = DownloadRun;
    type Report = RuntimeObservation;
    type Ending = PathBuf;
    type Refusal = String;
}

impl UndertakingAlg for Downloads<'_> {
    fn unattended(&self) -> Vec<QueueId> {
        self.manager
            .wanting_work()
            .into_iter()
            .filter(|id| {
                self.manager
                    .queue_entry(*id)
                    .is_some_and(|entry| entry.performer() == Performer::Retrieval)
            })
            .collect()
    }

    fn begin(&self, id: &QueueId) -> Result<DownloadRun, String> {
        let entry = self
            .manager
            .queue_entry(*id)
            .ok_or_else(|| "the request is gone".to_owned())?;
        let destination = entry
            .output()
            .ok_or_else(|| "the download names no output path".to_owned())?
            .to_owned();
        if self.manager.destination_is_taken(*id) {
            return Err(format!(
                "another request is already writing {}",
                destination.display()
            ));
        }
        let source = entry.source().to_owned();
        let selection = MediaSyntax.request_selection(entry);
        let target = OutputTarget::Path(destination.clone());
        let (sender, observations) = runtime_observation_channel();
        let pause = RuntimePause::running();
        let environment = RuntimeEnvironment::build_pausable(sender, pause.clone())
            .map_err(|error| error.to_string())?;
        let worker = thread::spawn(move || {
            if youtube_id(&source).is_some() {
                environment
                    .download_youtube(&source, &selection, &target)
                    .map_err(|error| error.to_string())
            } else {
                environment
                    .download_url(&source, &selection, &target)
                    .map_err(|error| error.to_string())
            }
        });
        Ok(DownloadRun {
            worker,
            observations,
            pause,
        })
    }
}

impl RunReadAlg for Downloads<'_> {
    fn run_is_over(&self, run: &DownloadRun) -> bool {
        run.worker.is_finished()
    }

    fn read_run(&self, run: &mut DownloadRun) -> Vec<RuntimeObservation> {
        run.observations.try_iter().collect()
    }

    fn end_run(&self, run: DownloadRun) -> Result<PathBuf, String> {
        match run.worker.join() {
            Ok(outcome) => outcome.map_err(|detail| media_failure(&detail)),
            Err(_) => Err("the download stopped without saying why".to_owned()),
        }
    }
}

impl RunHoldAlg for Downloads<'_> {
    fn holding_is_possible(&self) -> bool {
        // Retrieval is this program's own work, and it stops between one piece and the next.
        true
    }

    fn hold_run(&self, run: &mut DownloadRun, held: bool) {
        run.pause.set_paused(held);
    }

    fn abandon_run(&self, run: &mut DownloadRun) {
        run.pause.cancel();
    }
}

impl AttentionAlg for Downloads<'_> {
    fn begun(&mut self, id: &QueueId, holdable: bool) {
        self.manager.begun(*id, holdable);
    }

    fn heard(&mut self, id: &QueueId, report: RuntimeObservation) {
        report.interpret(&mut DownloadObservation {
            manager: self.manager,
            id: *id,
        });
    }

    fn ran_for(&mut self, id: &QueueId, elapsed: Duration) {
        self.manager.ran_for(*id, elapsed);
    }

    fn ended(&mut self, id: &QueueId, ending: Result<PathBuf, String>) {
        // A download that arrived said so as it ran; only a refusal is left to state here.
        if let Err(detail) = ending {
            self.manager.refused(*id, &detail);
        }
    }

    fn wanted(&self, id: &QueueId) -> Wanted {
        self.manager.wanted(*id)
    }
}

impl DownloadObservationInterpreter for DownloadObservation<'_> {
    fn progress(&mut self, progress: DownloadProgress) {
        self.manager.apply_transfer_event(
            self.id,
            TransferObservationOp::Progress {
                destination: progress.destination,
                downloaded: progress.downloaded,
                total: progress.total,
            },
        );
    }

    fn terminal(&mut self, event: DownloadEvent) {
        let observed = match event {
            DownloadEvent::Succeeded { destination, bytes } => {
                TransferObservationOp::Completed { destination, bytes }
            }
            DownloadEvent::Failed {
                destination,
                message,
            } => TransferObservationOp::Failed {
                destination,
                message,
            },
        };
        self.manager.apply_transfer_event(self.id, observed);
    }
}
