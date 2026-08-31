//! Holding a running transfer still, letting it go, and ending it.
//!
//! A transfer is another program. What can be checked here is what happens to that program: that
//! it stops, that it starts again, and that it does not outlive being abandoned.

use rsynko_memory::SyncCommand;
use rsynko_process::{HOLDING_IS_POSSIBLE, ProcessHold, ProcessSyncEnv};
use rsynko_rsync::SyncProgramExt;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, Instant};

/// States one program the interpreter runs in place of a transfer.
fn stated(script: &str) -> SyncCommand {
    SyncCommand {
        program: "/bin/sh".to_owned(),
        arguments: vec!["-c".to_owned(), script.to_owned()],
    }
}

/// Waits until something is true, and states whether it ever became true.
fn became(condition: impl Fn() -> bool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if condition() {
            return true;
        }
        thread::sleep(Duration::from_millis(20));
    }
    false
}

#[test]
fn an_abandoned_transfer_does_not_outlive_being_ended() {
    if !HOLDING_IS_POSSIBLE {
        return;
    }
    let (sender, watched) = channel();
    let hold = ProcessHold::default();
    let environment = ProcessSyncEnv::held(sender, hold.clone());
    // A transfer that would run far longer than anybody would wait for it.
    let command = stated("echo started; sleep 300");
    let worker = thread::spawn(move || environment.run_sync(&command));
    assert!(
        became(|| watched.try_recv().is_ok()),
        "the program never ran"
    );

    // Ending is asked for again until it is over, because a program that has not started yet
    // cannot be told to stop.
    let ended = became(|| {
        let _asked = hold.cancel();
        worker.is_finished()
    });

    assert!(ended, "an abandoned transfer outlived being ended");
}

#[cfg(target_os = "linux")]
#[test]
fn a_running_transfer_is_held_still_and_let_go_again() {
    let (sender, watched) = channel();
    let hold = ProcessHold::default();
    let environment = ProcessSyncEnv::held(sender, hold.clone());
    let command = stated("echo started; sleep 300");
    let worker = thread::spawn(move || environment.run_sync(&command));
    assert!(
        became(|| watched.try_recv().is_ok()),
        "the program never ran"
    );
    let process = hold.process().expect("a running program");

    assert!(hold.hold(), "a running program was not held");
    assert!(became(|| held_still(process)), "the program kept running");
    assert!(hold.release(), "a held program was not let go");
    assert!(became(|| !held_still(process)), "the program stayed held");

    let ended = became(|| {
        let _asked = hold.cancel();
        worker.is_finished()
    });
    assert!(ended, "the program outlived being ended");
}

/// Observes whether the machine says one of its own processes is stopped.
///
/// A process states its condition as one letter, after the name it was started under. The name
/// may itself hold spaces and brackets, so what follows the last bracket is where reading begins.
#[cfg(target_os = "linux")]
fn held_still(process: u32) -> bool {
    std::fs::read_to_string(format!("/proc/{process}/stat"))
        .ok()
        .and_then(|stated| {
            stated
                .rsplit_once(')')
                .and_then(|(_, rest)| rest.split_whitespace().next().map(str::to_owned))
        })
        .is_some_and(|condition| condition == "T")
}
