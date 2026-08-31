use crate::SessionSorts;
use ambassador::delegatable_trait;
use std::time::Duration;

/// Specifies what beginning, saying, running, and ending mean to whoever asked for the work.
#[delegatable_trait]
pub trait AttentionAlg: SessionSorts {
    /// States that a run has begun, and whether a run of this kind can be held still.
    fn begun(&mut self, id: &Self::Id, holdable: bool);

    /// States one thing a run said.
    fn heard(&mut self, id: &Self::Id, report: Self::Report);

    /// States how long a run has been running, not counting the time it was held still.
    fn ran_for(&mut self, id: &Self::Id, elapsed: Duration);

    /// States how a run ended.
    fn ended(&mut self, id: &Self::Id, ending: Result<Self::Ending, Self::Refusal>);

    /// States what the request wants of its run now.
    fn wanted(&self, id: &Self::Id) -> Wanted;
}

/// Denotes what a request wants of the run working on its behalf.
///
/// Removing a request, pausing it, and leaving the application are three ways of stating one of
/// these three things, and a run is attended to by what is wanted rather than by why.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wanted {
    /// Denotes a request that wants its run to carry on.
    Running,
    /// Denotes a request that wants its run held still, and kept.
    Held,
    /// Denotes a request that wants no run at all.
    Unwanted,
}
