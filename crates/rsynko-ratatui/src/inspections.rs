use crate::clock::Monotonic;
use ambassador::Delegate;
use rsynko_manager::*;
use rsynko_media::*;
use rsynko_memory::{Extraction, Format, ManagerState, MediaSyntax, QueueEntry, QueueId};
use rsynko_reqwest::RuntimeEnvironment;
use rsynko_session::*;
use rsynko_yt::{YoutubeNotesExt, media_failure};
use std::convert::Infallible;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Attends to source inspections, which state what one request may choose between.
#[derive(Debug, Delegate)]
#[delegate(ClockAlg, target = "clock")]
pub struct Inspections<'a> {
    manager: &'a mut ManagerState,
    clock: Monotonic,
}

/// Carries one running inspection.
#[derive(Debug)]
pub struct InspectionRun {
    worker: JoinHandle<Result<Inspected, String>>,
}

/// States everything one finished inspection found out about a source.
#[derive(Debug)]
pub struct Inspected {
    media_id: String,
    title: Option<String>,
    formats: Vec<Format>,
    notes: Vec<String>,
}

impl<'a> Inspections<'a> {
    /// Attends to the source inspections this collection asks for.
    pub const fn attending(manager: &'a mut ManagerState) -> Self {
        Self { manager, clock: Monotonic }
    }
}

impl SessionSorts for Inspections<'_> {
    type Id = QueueId;
    type Run = InspectionRun;
    /// An inspection says nothing until it knows everything, so there is nothing to hear.
    type Report = Infallible;
    type Ending = Inspected;
    type Refusal = String;
}

impl UndertakingAlg for Inspections<'_> {
    fn unattended(&self) -> Vec<QueueId> {
        // One source is inspected at a time: naming only the first is what makes that so.
        self.manager.first_waiting_format_catalog().map(QueueEntry::id).into_iter().collect()
    }

    fn begin(&self, id: &QueueId) -> Result<InspectionRun, String> {
        let source = self.manager.queue_entry(*id).ok_or_else(|| "the request is gone".to_owned())?.source().to_owned();
        let worker = thread::spawn(move || {
            let environment = RuntimeEnvironment::build().map_err(|error| error.to_string())?;
            let Extraction::Media(media) =
                environment.extract_url(&source).map_err(|error| media_failure(&error.to_string()))?
            else {
                return Err("the extracted result is not a single media item".to_owned());
            };
            Ok(Inspected {
                media_id: media.id,
                title: MediaSyntax.metadata_text(&media.metadata, "title").map(str::to_owned),
                notes: MediaSyntax.granting_notes(&media.metadata),
                formats: media.formats,
            })
        });
        Ok(InspectionRun { worker })
    }
}

impl RunReadAlg for Inspections<'_> {
    fn run_is_over(&self, run: &InspectionRun) -> bool {
        run.worker.is_finished()
    }

    fn read_run(&self, _run: &mut InspectionRun) -> Vec<Infallible> {
        Vec::new()
    }

    fn end_run(&self, run: InspectionRun) -> Result<Inspected, String> {
        match run.worker.join() {
            Ok(outcome) => outcome,
            Err(_) => Err("the inspection stopped without saying why".to_owned()),
        }
    }
}

impl RunHoldAlg for Inspections<'_> {
    fn holding_is_possible(&self) -> bool {
        // An inspection is one question and one answer: there is no middle to stop in.
        false
    }

    fn hold_run(&self, _run: &mut InspectionRun, _held: bool) {}

    fn abandon_run(&self, _run: &mut InspectionRun) {}
}

impl AttentionAlg for Inspections<'_> {
    fn begun(&mut self, id: &QueueId, _holdable: bool) {
        self.manager.apply_format_catalog_event(*id, FormatDiscoveryOp::Started {});
    }

    fn heard(&mut self, _id: &QueueId, report: Infallible) {
        match report {}
    }

    fn ran_for(&mut self, _id: &QueueId, _elapsed: Duration) {}

    fn ended(&mut self, id: &QueueId, ending: Result<Inspected, String>) {
        let event = match ending {
            Ok(found) => {
                self.manager.apply_source_metadata(*id, found.media_id, found.title);
                for note in found.notes {
                    self.manager.note_download(*id, note);
                }
                FormatDiscoveryOp::Available { formats: found.formats }
            }
            Err(message) => FormatDiscoveryOp::Failed { message },
        };
        self.manager.apply_format_catalog_event(*id, event);
    }

    fn wanted(&self, id: &QueueId) -> Wanted {
        // An inspection cannot be held still, so a request either still wants one or does not.
        if self.manager.exit_requested() || self.manager.queue_entry(*id).is_none() {
            return Wanted::Unwanted;
        }
        Wanted::Running
    }
}
