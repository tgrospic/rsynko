use crate::SessionSorts;
use ambassador::delegatable_trait;

/// Specifies reading one run while it happens, and ending it once it is over.
#[delegatable_trait]
pub trait RunReadAlg: SessionSorts {
    /// States whether the run is over.
    fn run_is_over(&self, run: &Self::Run) -> bool;

    /// States everything the run has said since it was last read, without waiting for more.
    fn read_run(&self, run: &mut Self::Run) -> Vec<Self::Report>;

    /// Waits for a run that is over, and states how it ended.
    ///
    /// # Errors
    ///
    /// Returns the interpreter's own refusal, when the run ended by refusing.
    fn end_run(&self, run: Self::Run) -> Result<Self::Ending, Self::Refusal>;
}
