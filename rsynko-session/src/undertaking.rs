use crate::SessionSorts;
use ambassador::delegatable_trait;

/// Specifies which requests want work done, and what beginning it is.
///
/// Attendance is not recorded twice. A request that has a run is no longer one that wants a run,
/// so what the requests themselves state is the whole record of what is running.
#[delegatable_trait]
pub trait UndertakingAlg: SessionSorts {
    /// States every request that wants a run and does not have one.
    fn unattended(&self) -> Vec<Self::Id>;

    /// Begins the work one request asks for.
    ///
    /// # Errors
    ///
    /// Returns the interpreter's own refusal to begin it.
    fn begin(&self, id: &Self::Id) -> Result<Self::Run, Self::Refusal>;
}
