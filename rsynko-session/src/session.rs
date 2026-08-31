use crate::{Attending, AttentionAlg, ClockAlg, RunHoldAlg, RunReadAlg, UndertakingAlg, Wanted};
use alux_ext::ext;
use std::time::Duration;

/// Derives one pass of attention from beginning, reading, holding, and ending runs.
#[ext(name = SessionExt)]
pub impl<This> This
where
    This: UndertakingAlg + RunReadAlg + RunHoldAlg + AttentionAlg + ClockAlg,
{
    /// Attends to every run once, then begins the work nobody has begun yet.
    ///
    /// The order is the meaning. Ending comes before beginning, so leaving never races starting;
    /// a run is seen to be over before it is read, so everything it wrote before ending is read;
    /// and what it said is stated before how it ended, so nothing it said is lost to its ending.
    fn attend(&mut self, running: &mut Vec<Attending<This::Id, This::Run, This::Moment>>) {
        let mut index = 0;
        while index < running.len() {
            if self.attend_one(&mut running[index]) {
                index += 1;
                continue;
            }
            let attending = running.swap_remove(index);
            let ending = self.end_run(attending.run);
            self.ended(&attending.id, ending);
        }
        let holdable = self.holding_is_possible();
        for id in self.unattended() {
            match self.begin(&id) {
                Ok(run) => {
                    self.begun(&id, holdable);
                    let began = self.now();
                    running.push(Attending::begun(id, run, began));
                }
                // A refusal to begin is how that work ended, and is stated once rather than
                // retried: what refused it is the request, which has not changed.
                Err(refusal) => self.ended(&id, Err(refusal)),
            }
        }
    }

    /// Attends to one run, and states whether it is still running afterward.
    fn attend_one(&mut self, attending: &mut Attending<This::Id, This::Run, This::Moment>) -> bool {
        let over = self.run_is_over(&attending.run);
        for report in self.read_run(&mut attending.run) {
            self.heard(&attending.id, report);
        }
        // A run that is over is past wanting: holding it still or telling it to stop would say
        // something about a run that is no longer happening.
        if !over {
            match self.wanted(&attending.id) {
                Wanted::Running => self.let_go(attending),
                Wanted::Held => self.hold_still(attending),
                // A run is another program: leaving without ending it leaves it running.
                Wanted::Unwanted => self.abandon_run(&mut attending.run),
            }
        }
        let elapsed = self.running_for(attending);
        self.ran_for(&attending.id, elapsed);
        !over
    }

    /// Holds one run still, and remembers when, so held time is not counted as running.
    fn hold_still(&self, attending: &mut Attending<This::Id, This::Run, This::Moment>) {
        if attending.held_since.is_none() {
            self.hold_run(&mut attending.run, true);
            attending.held_since = Some(self.now());
        }
    }

    /// Lets one held run go, keeping the time it was held still.
    fn let_go(&self, attending: &mut Attending<This::Id, This::Run, This::Moment>) {
        if let Some(held_since) = attending.held_since.take() {
            attending.held_for = attending.held_for.saturating_add(self.since(&held_since));
            self.hold_run(&mut attending.run, false);
        }
    }

    /// States how long one run has been running, not counting the time it was held still.
    fn running_for(&self, attending: &Attending<This::Id, This::Run, This::Moment>) -> Duration {
        let holding = attending
            .held_since
            .as_ref()
            .map_or(Duration::ZERO, |at| self.since(at));
        self.since(&attending.began)
            .saturating_sub(attending.held_for)
            .saturating_sub(holding)
    }
}
