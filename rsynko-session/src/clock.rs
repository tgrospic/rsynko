use ambassador::delegatable_trait;
use std::time::Duration;

/// Specifies reading the passage of time.
///
/// Time is read rather than taken, so how long a run has been running is a statement about a
/// clock the interpreter supplies and not about the one this machine happens to have.
#[delegatable_trait]
pub trait ClockAlg {
    /// Represents one reading of the clock.
    type Moment;

    /// Reads the clock now.
    fn now(&self) -> Self::Moment;

    /// States how long it has been since one reading.
    fn since(&self, moment: &Self::Moment) -> Duration;
}

/// Denotes one run, and the time it has spent running and held still.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attending<Id, Run, Moment> {
    /// Names the request the run works on behalf of.
    pub id: Id,
    /// Carries the run itself.
    pub run: Run,
    /// Denotes when the run began.
    pub began: Moment,
    /// Denotes when it was last held still, while it is still being held.
    pub held_since: Option<Moment>,
    /// Denotes how long it was held still before that.
    pub held_for: Duration,
}

impl<Id, Run, Moment> Attending<Id, Run, Moment> {
    /// Denotes one run that has just begun, and has not been held still yet.
    pub const fn begun(id: Id, run: Run, began: Moment) -> Self {
        Self {
            id,
            run,
            began,
            held_since: None,
            held_for: Duration::ZERO,
        }
    }
}
