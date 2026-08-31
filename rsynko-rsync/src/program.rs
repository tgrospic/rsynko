use crate::*;
use alux_ext::ext;
use ambassador::delegatable_trait;

/// Specifies running one transfer command and reading what it states.
///
/// The command is already decided; an interpreter only starts it, hands back what it wrote line
/// by line, and states how it ended.
#[delegatable_trait]
pub trait SyncRunAlg: RsyncSorts {
    /// Denotes failure to run the transfer.
    type Error;
    /// Carries one running transfer.
    type Run;

    /// Starts one transfer command.
    ///
    /// # Errors
    ///
    /// Returns the interpreter-specific failure to start it.
    fn start_sync(&self, command: &Self::Command) -> Result<Self::Run, Self::Error>;

    /// Reads the next line the running transfer wrote, and states when it has written its last.
    ///
    /// # Errors
    ///
    /// Returns the interpreter-specific failure to read from it.
    fn next_sync_line(&self, run: &mut Self::Run) -> Result<Option<String>, Self::Error>;

    /// Waits for the transfer to end and states whether it ended well.
    ///
    /// # Errors
    ///
    /// Returns the interpreter-specific failure the transfer ended with.
    fn finish_sync(&self, run: Self::Run) -> Result<(), Self::Error>;
}

/// Specifies what an interpreter does with each thing a running transfer states.
#[delegatable_trait]
pub trait SyncWatchAlg: RsyncSorts {
    /// Applies one observation of the running transfer.
    fn watch_sync(&self, observation: &Self::Observation);
}

/// Derives one whole transfer from starting it, reading it, and finishing it.
#[ext(name = SyncProgramExt)]
pub impl<This> This
where
    This: SyncRunAlg + SyncWatchAlg + SyncChangeAlg + SyncObservationAlg + SyncObservationViewAlg,
    This::Change: Clone,
{
    /// Runs one transfer and states every change it named, in the order it named them.
    ///
    /// Every line is watched as it arrives, so an interpreter following a transfer sees progress
    /// while it runs rather than after it ends.
    ///
    /// # Errors
    ///
    /// Returns the failure that started, read, or ended the transfer.
    fn run_sync(&self, command: &This::Command) -> Result<Vec<This::Change>, This::Error> {
        let mut run = self.start_sync(command)?;
        let mut changes = Vec::new();
        while let Some(line) = self.next_sync_line(&mut run)? {
            let observation = self.read_sync_line(&line);
            self.watch_sync(&observation);
            if let Some(change) = self.observation_change(&observation) {
                changes.push(change.clone());
            }
        }
        self.finish_sync(run)?;
        Ok(changes)
    }
}
