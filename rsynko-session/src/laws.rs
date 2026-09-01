//! Law scenarios for attendance, stated once over the capabilities.
//!
//! A scenario authors its own runs through a fixture and reads back what each request was told,
//! so it constrains any interpreter of the sorts rather than the one this workspace ships.

use crate::*;
use alux_ext::ext;
use anyhow::{Result, bail};
use std::fmt::Debug;
use std::time::Duration;

/// Supplies the runs an attendance scenario cannot author for itself.
pub trait SessionLawFixture: SessionSorts {
    /// Authors one request wanting work whose run states the given number of things, one per
    /// pass, and is over once it has stated them.
    fn law_wanting(&mut self, says: usize) -> Self::Id;

    /// Authors one request whose work refuses to begin.
    fn law_unbeginnable(&mut self) -> Self::Id;

    /// States what one request now wants of its run.
    fn law_wants(&mut self, id: &Self::Id, wanted: Wanted);

    /// Lets the stated time pass.
    fn law_passes(&mut self, elapsed: Duration);

    /// Observes everything one request was told, in the order it was told.
    fn law_told(&self, id: &Self::Id) -> Vec<Telling>;

    /// Observes how many times one run was told to end whether or not it had finished.
    fn law_abandonments(&self, id: &Self::Id) -> usize;
}

/// Authors the attendance laws.
#[ext(name = SessionLaws)]
pub impl<This> This
where
    This: UndertakingAlg + RunReadAlg + RunHoldAlg + AttentionAlg + ClockAlg + SessionLawFixture,
    This::Id: Clone + Eq + Debug,
{
    /// Checks that one pass reconciles what is wanted with what is happening.
    ///
    /// The laws checked are:
    ///
    /// 1. one pass begins every request wanting work, and no other;
    /// 2. a request whose work has begun is not begun again, however many passes run;
    /// 3. everything a run said is told before how it ended, and nothing is told after;
    /// 4. work refusing to begin ends once and is not begun again;
    /// 5. a run nobody wants is told to end on every pass until it is over, and ends once;
    /// 6. time a run is held still is not counted as time it was running;
    /// 7. attending to no runs tells nothing.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn session_laws(&mut self) -> Result<()> {
        self.check_beginning_laws()?;
        self.check_saying_laws()?;
        self.check_unwanted_laws()?;
        self.check_holding_laws()
    }

    /// Checks what one pass begins, and what it declines to begin twice.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn check_beginning_laws(&mut self) -> Result<()> {
        let mut running = Vec::new();
        self.attend(&mut running);
        if !running.is_empty() {
            bail!("attending to nothing began something");
        }

        let wanting = [self.law_wanting(1), self.law_wanting(1)];
        let refusing = self.law_unbeginnable();
        self.attend(&mut running);
        if running.len() != wanting.len() {
            bail!("one pass began {} runs where {} requests wanted work", running.len(), wanting.len());
        }
        for id in &wanting {
            if begun_count(&self.law_told(id)) != 1 {
                bail!("{id:?} was not told once that its work began");
            }
        }
        if self.law_told(&refusing) != vec![Telling::Refused] {
            bail!("work refusing to begin was not stated once as a refusal");
        }

        self.attend(&mut running);
        for id in &wanting {
            if begun_count(&self.law_told(id)) != 1 {
                bail!("{id:?} was begun again while its run was still happening");
            }
        }
        if self.law_told(&refusing) != vec![Telling::Refused] {
            bail!("work refusing to begin was begun again");
        }
        Ok(())
    }

    /// Checks that what a run said reaches the request before how it ended.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn check_saying_laws(&mut self) -> Result<()> {
        let mut running = Vec::new();
        let said = 3;
        let id = self.law_wanting(said);
        self.attend(&mut running);
        while !running.is_empty() {
            self.attend(&mut running);
        }

        let told = self.law_told(&id);
        let heard = told.iter().filter(|telling| **telling == Telling::Heard).count();
        if heard != said {
            bail!("a run stating {said} things was heard {heard} times");
        }
        let Some(ending) = told.iter().position(|telling| *telling == Telling::Ended) else {
            bail!("a run that finished never stated how it ended");
        };
        if ending + 1 != told.len() {
            bail!("something was stated after the run had ended");
        }
        if told.first() != Some(&Telling::Begun) {
            bail!("a run stated something before it began");
        }
        Ok(())
    }

    /// Checks how a run nobody wants any more is ended.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn check_unwanted_laws(&mut self) -> Result<()> {
        let mut running = Vec::new();
        let id = self.law_wanting(4);
        self.attend(&mut running);
        self.law_wants(&id, Wanted::Unwanted);

        let mut passes = 0;
        while !running.is_empty() {
            self.attend(&mut running);
            passes += 1;
            if passes > 8 {
                bail!("a run nobody wanted never ended");
            }
        }
        // Told once for every pass it was still happening: a run that has not begun yet cannot
        // be told to stop, so telling it once would let it escape.
        if self.law_abandonments(&id) != passes - 1 {
            bail!("a run nobody wanted was told to end {} times over {passes} passes", self.law_abandonments(&id));
        }
        let ended =
            self.law_told(&id).iter().filter(|telling| matches!(telling, Telling::Ended | Telling::Refused)).count();
        if ended != 1 {
            bail!("a run nobody wanted stated how it ended {ended} times");
        }
        Ok(())
    }

    /// Checks that time a run is held still is not time it was running.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn check_holding_laws(&mut self) -> Result<()> {
        let mut running = Vec::new();
        let id = self.law_wanting(5);
        let tick = Duration::from_secs(1);
        self.attend(&mut running);

        self.law_passes(tick);
        self.law_wants(&id, Wanted::Held);
        self.attend(&mut running);
        let held_at = last_elapsed(&self.law_told(&id));

        self.law_passes(tick * 10);
        self.attend(&mut running);
        if last_elapsed(&self.law_told(&id)) != held_at {
            bail!("a run held still went on running");
        }

        self.law_wants(&id, Wanted::Running);
        self.attend(&mut running);
        self.law_passes(tick);
        self.attend(&mut running);
        if last_elapsed(&self.law_told(&id)) != held_at.map(|elapsed| elapsed + tick) {
            bail!("a run let go again did not carry on from where it was held");
        }
        Ok(())
    }
}

/// Denotes one thing a request was told about the run working on its behalf.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Telling {
    /// Denotes being told that a run began.
    Begun,
    /// Denotes being told one thing the run said.
    Heard,
    /// Denotes being told how long the run has been running.
    RanFor(Duration),
    /// Denotes being told that the run ended well.
    Ended,
    /// Denotes being told that the run refused.
    Refused,
}

/// Counts how many times one request was told its work began.
fn begun_count(told: &[Telling]) -> usize {
    told.iter().filter(|telling| **telling == Telling::Begun).count()
}

/// Observes the last running time one request was told, when it was told one.
fn last_elapsed(told: &[Telling]) -> Option<Duration> {
    told.iter().rev().find_map(|telling| match telling {
        Telling::RanFor(elapsed) => Some(*elapsed),
        Telling::Begun | Telling::Heard | Telling::Ended | Telling::Refused => None,
    })
}
