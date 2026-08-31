use crate::SessionSorts;
use ambassador::delegatable_trait;

/// Specifies holding one run still, letting it go again, and ending one nobody wants.
#[delegatable_trait]
pub trait RunHoldAlg: SessionSorts {
    /// States whether a run of this kind can be held still at all.
    fn holding_is_possible(&self) -> bool;

    /// Holds the run still, or lets it go.
    fn hold_run(&self, run: &mut Self::Run, held: bool);

    /// Tells the run to end whether or not it has finished.
    ///
    /// A run that has not begun yet cannot be told to stop, so this is stated again on every pass
    /// until the run is over rather than once.
    fn abandon_run(&self, run: &mut Self::Run);
}
