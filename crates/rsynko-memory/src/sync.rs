//! The in-memory layout of folder transfers, and a run that states a transcript.

use crate::PlannedChange;
use ambassador::Delegate;
use rsynko_manager::ChangeKind;
use rsynko_rsync::*;
use std::cell::RefCell;
use std::collections::VecDeque;

/// Interprets the folder-transfer sorts as ordinary in-memory records.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RsyncSyntax;

/// Denotes one end of a folder transfer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RsyncEndpoint {
    /// Names the account the far end is reached as.
    pub user: Option<String>,
    /// Names the machine the path rests on.
    pub host: Option<String>,
    /// Names the path itself.
    pub path: String,
}

/// Denotes exactly the command one transfer runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncCommand {
    /// Names the program the transfer is performed by.
    pub program: String,
    /// States every argument, in order.
    pub arguments: Vec<String>,
}

/// Denotes one thing a running transfer states.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyncObservation {
    /// States one path the transfer would change, or has changed.
    Change(PlannedChange),
    /// States how far the whole transfer has come.
    Progress {
        /// Counts the bytes moved so far.
        transferred: u64,
        /// States the completed share as whole percent.
        percent: u16,
    },
    /// States a line naming nothing the specification reads.
    Nothing,
}

impl RsyncSorts for RsyncSyntax {
    type Endpoint = RsyncEndpoint;
    type Command = SyncCommand;
    type Observation = SyncObservation;
    type Change = PlannedChange;
}

impl RsyncEndpointAlg for RsyncSyntax {
    fn endpoint(
        &self,
        user: Option<String>,
        host: Option<String>,
        path: impl Into<String>,
    ) -> Self::Endpoint {
        RsyncEndpoint {
            user,
            host,
            path: path.into(),
        }
    }
}

impl RsyncEndpointViewAlg for RsyncSyntax {
    fn endpoint_user<'a>(&self, endpoint: &'a Self::Endpoint) -> Option<&'a str> {
        endpoint.user.as_deref()
    }

    fn endpoint_host<'a>(&self, endpoint: &'a Self::Endpoint) -> Option<&'a str> {
        endpoint.host.as_deref()
    }

    fn endpoint_path<'a>(&self, endpoint: &'a Self::Endpoint) -> &'a str {
        &endpoint.path
    }
}

impl SyncCommandAlg for RsyncSyntax {
    fn sync_command(
        &self,
        program: impl Into<String>,
        arguments: impl IntoIterator<Item = String>,
    ) -> Self::Command {
        SyncCommand {
            program: program.into(),
            arguments: arguments.into_iter().collect(),
        }
    }
}

impl SyncCommandViewAlg for RsyncSyntax {
    fn command_program<'a>(&self, command: &'a Self::Command) -> &'a str {
        &command.program
    }

    fn command_arguments<'a>(&self, command: &'a Self::Command) -> impl Iterator<Item = &'a str> {
        command.arguments.iter().map(String::as_str)
    }
}

impl SyncChangeAlg for RsyncSyntax {
    fn sync_change(
        &self,
        path: impl Into<String>,
        kind: ChangeKind,
        size: Option<u64>,
    ) -> Self::Change {
        PlannedChange::new(path.into(), kind, size)
    }
}

impl SyncObservationAlg for RsyncSyntax {
    fn observed_change(&self, change: Self::Change) -> Self::Observation {
        SyncObservation::Change(change)
    }

    fn observed_progress(&self, transferred: u64, percent: u16) -> Self::Observation {
        SyncObservation::Progress {
            transferred,
            percent,
        }
    }

    fn observed_nothing(&self) -> Self::Observation {
        SyncObservation::Nothing
    }
}

impl SyncObservationViewAlg for RsyncSyntax {
    fn observation_change<'a>(
        &self,
        observation: &'a Self::Observation,
    ) -> Option<&'a Self::Change> {
        match observation {
            SyncObservation::Change(change) => Some(change),
            SyncObservation::Progress { .. } | SyncObservation::Nothing => None,
        }
    }

    fn observation_progress(&self, observation: &Self::Observation) -> Option<(u64, u16)> {
        match observation {
            SyncObservation::Progress {
                transferred,
                percent,
            } => Some((*transferred, *percent)),
            SyncObservation::Change(_) | SyncObservation::Nothing => None,
        }
    }
}

/// Runs folder transfers by stating a transcript instead of moving anything.
// Several capabilities delegate to the same component, which Clippy reads as a repeated attribute.
#[allow(
    clippy::duplicated_attributes,
    reason = "one delegation per capability, not per target"
)]
#[derive(Clone, Debug, Default, Delegate)]
#[delegate(RsyncEndpointAlg, target = "syntax")]
#[delegate(RsyncEndpointViewAlg, target = "syntax")]
#[delegate(SyncCommandAlg, target = "syntax")]
#[delegate(SyncCommandViewAlg, target = "syntax")]
#[delegate(SyncChangeAlg, target = "syntax")]
#[delegate(SyncObservationAlg, target = "syntax")]
#[delegate(SyncObservationViewAlg, target = "syntax")]
pub struct ReferenceSyncEnv {
    syntax: RsyncSyntax,
    transcript: Vec<String>,
    refusal: Option<String>,
    commands: RefCell<Vec<SyncCommand>>,
    watched: RefCell<Vec<SyncObservation>>,
}

/// Denotes a transfer the reference interpreter refused to run.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("the reference transfer refused: {0}")]
pub struct ReferenceSyncError(pub String);

impl RsyncSorts for ReferenceSyncEnv {
    type Endpoint = RsyncEndpoint;
    type Command = SyncCommand;
    type Observation = SyncObservation;
    type Change = PlannedChange;
}

impl ReferenceSyncEnv {
    /// States exactly what the next transfer writes, whatever command it is given.
    pub fn register_transcript(&mut self, lines: impl IntoIterator<Item = String>) {
        self.transcript = lines.into_iter().collect();
    }

    /// Refuses every subsequent transfer, so failure laws can be exercised.
    pub fn refuse_transfers(&mut self, reason: impl Into<String>) {
        self.refusal = Some(reason.into());
    }

    /// Observes every command the interpreter was asked to run, in order.
    #[must_use]
    pub fn commands(&self) -> Vec<SyncCommand> {
        self.commands.borrow().clone()
    }

    /// Observes everything the interpreter read out of its transfers, in order.
    #[must_use]
    pub fn watched(&self) -> Vec<SyncObservation> {
        self.watched.borrow().clone()
    }
}

impl SyncRunAlg for ReferenceSyncEnv {
    type Error = ReferenceSyncError;
    type Run = VecDeque<String>;

    fn start_sync(&self, command: &Self::Command) -> Result<Self::Run, Self::Error> {
        self.commands.borrow_mut().push(command.clone());
        match &self.refusal {
            Some(reason) => Err(ReferenceSyncError(reason.clone())),
            None => Ok(self.transcript.iter().cloned().collect()),
        }
    }

    fn next_sync_line(&self, run: &mut Self::Run) -> Result<Option<String>, Self::Error> {
        Ok(run.pop_front())
    }

    fn finish_sync(&self, _run: Self::Run) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl SyncWatchAlg for ReferenceSyncEnv {
    fn watch_sync(&self, observation: &Self::Observation) {
        self.watched.borrow_mut().push(observation.clone());
    }
}
