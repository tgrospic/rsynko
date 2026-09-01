use crate::ProcessHold;
use ambassador::Delegate;
use rsynko_manager::ChangeKind;
use rsynko_memory::{PlannedChange, RsyncEndpoint, RsyncSyntax, SyncCommand, SyncObservation};
use rsynko_rsync::*;
use std::io::{BufReader, Read};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::mpsc::Sender;

/// Runs folder transfers as operating-system processes.
// Several capabilities delegate to the same component, which Clippy reads as a repeated attribute.
#[allow(clippy::duplicated_attributes, reason = "one delegation per capability, not per target")]
#[derive(Debug, Delegate)]
#[delegate(RsyncEndpointAlg, target = "syntax")]
#[delegate(RsyncEndpointViewAlg, target = "syntax")]
#[delegate(SyncCommandAlg, target = "syntax")]
#[delegate(SyncCommandViewAlg, target = "syntax")]
#[delegate(SyncChangeAlg, target = "syntax")]
#[delegate(SyncObservationAlg, target = "syntax")]
#[delegate(SyncObservationViewAlg, target = "syntax")]
pub struct ProcessSyncEnv {
    syntax: RsyncSyntax,
    observations: Sender<SyncObservation>,
    hold: ProcessHold,
}

impl RsyncSorts for ProcessSyncEnv {
    type Endpoint = RsyncEndpoint;
    type Command = SyncCommand;
    type Observation = SyncObservation;
    type Change = PlannedChange;
}

/// Carries one running transfer process and the output it has not been read out of yet.
#[derive(Debug)]
pub struct SyncRun {
    child: Child,
    output: BufReader<ChildStdout>,
}

/// Denotes failure to run one folder transfer.
#[derive(Debug, thiserror::Error)]
pub enum ProcessSyncError {
    /// Denotes a transfer program that could not be started at all.
    #[error("the transfer program could not be started: {0}")]
    Unstartable(std::io::Error),
    /// Denotes failure to read what a running transfer wrote.
    #[error("the transfer could not be read: {0}")]
    Unreadable(std::io::Error),
    /// Denotes a transfer that ran and refused what it was asked to do.
    #[error("the transfer refused: {0}")]
    Refused(String),
}

impl ProcessSyncEnv {
    /// Runs transfers, stating every observation it reads to the given receiver.
    #[must_use]
    pub fn new(observations: Sender<SyncObservation>) -> Self {
        Self::held(observations, ProcessHold::default())
    }

    /// Runs transfers that whoever holds this handle can hold still and let go again.
    #[must_use]
    pub const fn held(observations: Sender<SyncObservation>, hold: ProcessHold) -> Self {
        Self { syntax: RsyncSyntax, observations, hold }
    }
}

impl SyncRunAlg for ProcessSyncEnv {
    type Error = ProcessSyncError;
    type Run = SyncRun;

    fn start_sync(&self, command: &Self::Command) -> Result<Self::Run, Self::Error> {
        let mut started = Command::new(self.command_program(command));
        started
            .args(self.command_arguments(command).collect::<Vec<_>>())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // The transfer is given a family of its own, so holding it still holds all of it and
        // nothing else.
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut started, 0);
        let mut child = started.spawn().map_err(ProcessSyncError::Unstartable)?;
        self.hold.running(Some(child.id()));
        let stdout =
            child.stdout.take().ok_or_else(|| ProcessSyncError::Refused("the transfer wrote nothing".to_owned()))?;
        Ok(SyncRun { child, output: BufReader::new(stdout) })
    }

    fn next_sync_line(&self, run: &mut Self::Run) -> Result<Option<String>, Self::Error> {
        let mut line = Vec::new();
        let mut byte = [0_u8; 1];
        loop {
            let read = run.output.read(&mut byte).map_err(ProcessSyncError::Unreadable)?;
            if read == 0 {
                break;
            }
            // Progress is written over itself with carriage returns, so either ends a line.
            if byte[0] == b'\n' || byte[0] == b'\r' {
                if line.is_empty() {
                    continue;
                }
                break;
            }
            line.push(byte[0]);
        }
        if line.is_empty() {
            return Ok(None);
        }
        Ok(Some(String::from_utf8_lossy(&line).into_owned()))
    }

    fn finish_sync(&self, mut run: Self::Run) -> Result<(), Self::Error> {
        let status = run.child.wait().map_err(ProcessSyncError::Unreadable)?;
        self.hold.running(None);
        if status.success() {
            return Ok(());
        }
        let mut refusal = String::new();
        if let Some(errors) = run.child.stderr.as_mut() {
            let _read = errors.read_to_string(&mut refusal);
        }
        let refusal = refusal.trim();
        Err(ProcessSyncError::Refused(if refusal.is_empty() {
            format!("the transfer ended with {status}")
        } else {
            refusal.lines().take(4).collect::<Vec<_>>().join("; ")
        }))
    }
}

impl SyncWatchAlg for ProcessSyncEnv {
    fn watch_sync(&self, observation: &Self::Observation) {
        // Nobody watching is not a failure: a transfer states what it does whether or not it is
        // being read.
        let _watched = self.observations.send(observation.clone());
    }
}
