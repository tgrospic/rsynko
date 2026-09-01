//! The process interpreter, run against a stated program rather than a transfer.
//!
//! What a transfer means is checked by the specification's own laws. What is checked here is only
//! what this crate decides: that a line ends at a newline or a carriage return, that every line
//! reaches whoever is watching, and that a program refusing its work is stated as a refusal.

use rsynko_manager::{ChangeKind, PlannedChangeAlg};
use rsynko_memory::{SyncCommand, SyncObservation};
use rsynko_process::{ProcessSyncEnv, ProcessSyncError};
use rsynko_rsync::SyncProgramExt;
use std::sync::mpsc::channel;

/// States one program the interpreter runs in place of a transfer.
fn stated(script: &str) -> SyncCommand {
    SyncCommand { program: "/bin/sh".to_owned(), arguments: vec!["-c".to_owned(), script.to_owned()] }
}

#[test]
fn progress_and_changes_arrive_however_the_program_ends_its_lines() {
    let (sender, watched) = channel();
    let environment = ProcessSyncEnv::new(sender);
    // Progress is written over itself with carriage returns; itemized paths end with newlines.
    let command = stated(r"printf '>f+++++++++|4|new.txt\n    19,084,083  28%%   2.02MB/s\r*deleting  |0|gone.txt\n'");

    let changes = environment.run_sync(&command).expect("stated program runs");

    let named = changes
        .iter()
        .map(|change| (change.change_path(), change.change_kind(), change.change_size()))
        .collect::<Vec<_>>();
    assert_eq!(
        named,
        vec![
            ("new.txt", ChangeKind::Create, Some(4)),
            // A removal states no size: the transfer never looked at what it holds.
            ("gone.txt", ChangeKind::Delete, None),
        ]
    );
    let observed = watched.try_iter().collect::<Vec<_>>();
    assert!(
        observed.iter().any(|observation| matches!(
            observation,
            SyncObservation::Progress { transferred: 19_084_083, percent: 28 }
        ))
    );
    assert_eq!(observed.len(), 3);
}

#[test]
fn a_program_that_refuses_states_why() {
    let (sender, _watched) = channel();
    let environment = ProcessSyncEnv::new(sender);
    let command = stated("echo 'rsync: connection unexpectedly closed' >&2; exit 12");

    let refusal = environment.run_sync(&command).expect_err("the stated program refuses");

    match refusal {
        ProcessSyncError::Refused(reason) => {
            assert_eq!(reason, "rsync: connection unexpectedly closed");
        }
        other => panic!("a refusing program states {other}"),
    }
}

#[test]
fn a_program_that_does_not_exist_is_not_a_refusal() {
    let (sender, _watched) = channel();
    let environment = ProcessSyncEnv::new(sender);
    let command = SyncCommand { program: "rsynko-no-such-program".to_owned(), arguments: Vec::new() };

    let refusal = environment.run_sync(&command).expect_err("an absent program cannot run");

    assert!(matches!(refusal, ProcessSyncError::Unstartable(_)));
}
