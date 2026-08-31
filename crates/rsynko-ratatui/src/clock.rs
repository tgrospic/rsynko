use rsynko_session::ClockAlg;
use std::time::{Duration, Instant};

/// Reads this machine's own clock, which only ever moves forward.
#[derive(Clone, Copy, Debug, Default)]
pub struct Monotonic;

impl ClockAlg for Monotonic {
    type Moment = Instant;

    fn now(&self) -> Instant {
        Instant::now()
    }

    fn since(&self, moment: &Instant) -> Duration {
        moment.elapsed()
    }
}
