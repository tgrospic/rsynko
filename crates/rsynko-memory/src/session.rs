use rsynko_session::*;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

/// Attends to runs that say what they were authored to say, one thing per pass.
///
/// Nothing here happens elsewhere: a run is a count of things left to say, and time passes only
/// when a scenario says it does, so what attendance means can be read without waiting for it.
#[derive(Debug, Default)]
pub struct ReferenceSession {
    requests: Vec<ReferenceRequest>,
    clock: Duration,
}

/// Holds one authored request, and everything it has been told about its run.
#[derive(Debug)]
struct ReferenceRequest {
    id: usize,
    wanted: Wanted,
    says: usize,
    beginnable: bool,
    attended: bool,
    told: Vec<Telling>,
    abandonments: Rc<Cell<usize>>,
}

/// Carries one authored run: the work it has left, what it has said, and whether it was stopped.
#[derive(Debug)]
pub struct ReferenceRun {
    work: usize,
    said: Vec<usize>,
    held: bool,
    abandoning: bool,
    abandonments: Rc<Cell<usize>>,
}

impl ReferenceSession {
    /// Observes one authored request.
    fn request(&self, id: usize) -> &ReferenceRequest {
        self.requests
            .iter()
            .find(|request| request.id == id)
            .expect("an authored request")
    }

    /// Observes one authored request to tell it something.
    fn request_mut(&mut self, id: usize) -> &mut ReferenceRequest {
        self.requests
            .iter_mut()
            .find(|request| request.id == id)
            .expect("an authored request")
    }

    /// Authors one request, and names it.
    fn author(&mut self, says: usize, beginnable: bool) -> usize {
        let id = self.requests.len();
        self.requests.push(ReferenceRequest {
            id,
            wanted: Wanted::Running,
            says,
            beginnable,
            attended: false,
            told: Vec::new(),
            abandonments: Rc::new(Cell::new(0)),
        });
        id
    }
}

impl SessionSorts for ReferenceSession {
    type Id = usize;
    type Run = ReferenceRun;
    type Report = usize;
    type Ending = ();
    type Refusal = String;
}

impl UndertakingAlg for ReferenceSession {
    fn unattended(&self) -> Vec<usize> {
        self.requests
            .iter()
            .filter(|request| !request.attended)
            .map(|request| request.id)
            .collect()
    }

    fn begin(&self, id: &usize) -> Result<ReferenceRun, String> {
        let request = self.request(*id);
        if !request.beginnable {
            return Err("the work refuses to begin".to_owned());
        }
        Ok(ReferenceRun {
            work: request.says,
            said: Vec::new(),
            held: false,
            abandoning: false,
            abandonments: Rc::clone(&request.abandonments),
        })
    }
}

impl RunReadAlg for ReferenceSession {
    fn run_is_over(&self, run: &ReferenceRun) -> bool {
        run.work == 0
    }

    fn read_run(&self, run: &mut ReferenceRun) -> Vec<usize> {
        if run.abandoning {
            // Being told to stop is honored where the run next looks, which is here.
            run.work = 0;
            run.said.clear();
            return Vec::new();
        }
        if run.held {
            return Vec::new();
        }
        // What a run says reaches a reader after it was said, so the last thing it says is still
        // waiting to be read once the run itself is over.
        let said = std::mem::take(&mut run.said);
        if run.work > 0 {
            run.work -= 1;
            run.said.push(run.work);
        }
        said
    }

    fn end_run(&self, _run: ReferenceRun) -> Result<(), String> {
        Ok(())
    }
}

impl RunHoldAlg for ReferenceSession {
    fn holding_is_possible(&self) -> bool {
        true
    }

    fn hold_run(&self, run: &mut ReferenceRun, held: bool) {
        run.held = held;
    }

    fn abandon_run(&self, run: &mut ReferenceRun) {
        run.abandoning = true;
        run.abandonments.set(run.abandonments.get() + 1);
    }
}

impl AttentionAlg for ReferenceSession {
    fn begun(&mut self, id: &usize, _holdable: bool) {
        let request = self.request_mut(*id);
        request.attended = true;
        request.told.push(Telling::Begun);
    }

    fn heard(&mut self, id: &usize, _report: usize) {
        self.request_mut(*id).told.push(Telling::Heard);
    }

    fn ran_for(&mut self, id: &usize, elapsed: Duration) {
        self.request_mut(*id).told.push(Telling::RanFor(elapsed));
    }

    fn ended(&mut self, id: &usize, ending: Result<(), String>) {
        let request = self.request_mut(*id);
        // Work that refused to begin is over the moment it refused, and is not begun again.
        request.attended = true;
        request.told.push(match ending {
            Ok(()) => Telling::Ended,
            Err(_) => Telling::Refused,
        });
    }

    fn wanted(&self, id: &usize) -> Wanted {
        self.request(*id).wanted
    }
}

impl ClockAlg for ReferenceSession {
    type Moment = Duration;

    fn now(&self) -> Duration {
        self.clock
    }

    fn since(&self, moment: &Duration) -> Duration {
        self.clock.saturating_sub(*moment)
    }
}

impl SessionLawFixture for ReferenceSession {
    fn law_wanting(&mut self, says: usize) -> usize {
        self.author(says, true)
    }

    fn law_unbeginnable(&mut self) -> usize {
        self.author(0, false)
    }

    fn law_wants(&mut self, id: &usize, wanted: Wanted) {
        self.request_mut(*id).wanted = wanted;
    }

    fn law_passes(&mut self, elapsed: Duration) {
        self.clock = self.clock.saturating_add(elapsed);
    }

    fn law_told(&self, id: &usize) -> Vec<Telling> {
        self.request(*id).told.clone()
    }

    fn law_abandonments(&self, id: &usize) -> usize {
        self.request(*id).abandonments.get()
    }
}
