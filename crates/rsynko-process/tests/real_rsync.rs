//! Real folder transfers, run against a tree the test builds for itself.
//!
//! These need the transfer program itself, which not every machine has. Where it is absent the
//! scenario states so and stops, rather than claiming to have checked something it could not.

use rsynko_manager::{ChangeKind, PlannedChangeAlg};
use rsynko_process::ProcessSyncEnv;
use rsynko_rsync::{
    RsyncEndpointExt, SYNC_PROGRAM, SyncCommandExt, SyncMode, SyncObservationViewAlg, SyncProfile,
    SyncProgramExt,
};
use std::fs::{create_dir_all, read, write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc::channel;

/// Observes whether the machine running the test has the transfer program.
fn transfers_available() -> bool {
    Command::new(SYNC_PROGRAM)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Authors a folder of documents, and a stale copy of it to transfer into.
fn authored(root: &Path) {
    let source = root.join("Documents");
    create_dir_all(source.join("Invoices/2026")).expect("source folders");
    create_dir_all(source.join("Notes")).expect("source folders");
    write(
        source.join("Invoices/2026/invoice-0104.pdf"),
        vec![7_u8; 2048],
    )
    .expect("a kept file");
    write(
        source.join("Invoices/2026/invoice-0105.pdf"),
        vec![9_u8; 4096],
    )
    .expect("a new file");
    write(source.join("Notes/todo.txt"), b"the longer new text").expect("a changed file");

    let backup = root.join("Documents-backup");
    create_dir_all(backup.join("Invoices/2026")).expect("backup folders");
    create_dir_all(backup.join("Notes")).expect("backup folders");
    write(
        backup.join("Invoices/2026/invoice-0104.pdf"),
        vec![7_u8; 2048],
    )
    .expect("a kept file");
    write(backup.join("Notes/todo.txt"), b"the old text").expect("a changed file");
    write(backup.join("Notes/gone.md"), b"only here").expect("a removed file");
}

/// Counts what a report states of one kind.
fn counted(changes: &[impl PlannedChangeAlg], kind: ChangeKind) -> usize {
    changes
        .iter()
        .filter(|change| change.change_kind() == kind)
        .count()
}

#[test]
fn a_rehearsal_states_what_it_would_do_and_changes_nothing() {
    if !transfers_available() {
        println!("{SYNC_PROGRAM} is absent: the transfer was not run");
        return;
    }
    let root = tempfile::tempdir().expect("a place to author folders");
    authored(root.path());
    let (sender, watched) = channel();
    let environment = ProcessSyncEnv::new(sender);
    let command = environment.transfer_command(
        &environment.read_endpoint(&format!("{}/", root.path().join("Documents").display())),
        &environment.read_endpoint(&root.path().join("Documents-backup").display().to_string()),
        SyncMode::rehearsal().profiled(SyncProfile::Mirror),
    );

    let changes = environment.run_sync(&command).expect("the rehearsal runs");

    assert_eq!(counted(&changes, ChangeKind::Create), 1);
    assert_eq!(counted(&changes, ChangeKind::Update), 1);
    assert_eq!(counted(&changes, ChangeKind::Delete), 1);
    // A report states what it would leave exactly as it is, and not only what it would alter.
    assert_eq!(counted(&changes, ChangeKind::Unchanged), 1);
    assert!(changes.iter().any(
        |change| change.change_path() == "Invoices/2026/invoice-0105.pdf"
            && change.change_size() == Some(4096)
    ));
    // Nothing was moved, so the stale copy is exactly as stale as it was.
    let backup = root.path().join("Documents-backup");
    assert!(!backup.join("Invoices/2026/invoice-0105.pdf").exists());
    assert!(backup.join("Notes/gone.md").exists());
    assert_eq!(
        read(backup.join("Notes/todo.txt")).expect("the old text"),
        b"the old text"
    );
    assert!(watched.try_iter().count() >= changes.len());
}

#[test]
fn a_transfer_does_what_its_rehearsal_said_it_would() {
    if !transfers_available() {
        println!("{SYNC_PROGRAM} is absent: the transfer was not run");
        return;
    }
    let root = tempfile::tempdir().expect("a place to author folders");
    authored(root.path());
    let (sender, _watched) = channel();
    let environment = ProcessSyncEnv::new(sender);
    let source =
        environment.read_endpoint(&format!("{}/", root.path().join("Documents").display()));
    let destination =
        environment.read_endpoint(&root.path().join("Documents-backup").display().to_string());
    let rehearsed = environment
        .run_sync(&environment.transfer_command(
            &source,
            &destination,
            SyncMode::rehearsal().profiled(SyncProfile::Mirror),
        ))
        .expect("the rehearsal runs");

    let performed = environment
        .run_sync(&environment.transfer_command(
            &source,
            &destination,
            SyncMode::transfer().profiled(SyncProfile::Mirror),
        ))
        .expect("the transfer runs");

    let named = |changes: &[_]| {
        let mut paths = changes
            .iter()
            .map(|change: &_| {
                format!(
                    "{:?} {}",
                    PlannedChangeAlg::change_kind(change),
                    PlannedChangeAlg::change_path(change)
                )
            })
            .collect::<Vec<_>>();
        paths.sort();
        paths
    };
    assert_eq!(named(&performed), named(&rehearsed));
    let backup = root.path().join("Documents-backup");
    assert_eq!(
        read(backup.join("Invoices/2026/invoice-0105.pdf")).expect("the new file arrived"),
        vec![9_u8; 4096]
    );
    assert_eq!(
        read(backup.join("Notes/todo.txt")).expect("the changed file arrived"),
        b"the longer new text"
    );
    assert!(!backup.join("Notes/gone.md").exists());
}

#[test]
fn a_running_transfer_states_how_far_it_has_come() {
    if !transfers_available() {
        println!("{SYNC_PROGRAM} is absent: the transfer was not run");
        return;
    }
    let root = tempfile::tempdir().expect("a place to author folders");
    let source = root.path().join("Large");
    create_dir_all(&source).expect("a source folder");
    for index in 0..4 {
        write(
            source.join(format!("part-{index}.bin")),
            vec![index; 512 * 1024],
        )
        .expect("something worth watching");
    }
    let (sender, watched) = channel();
    let environment = ProcessSyncEnv::new(sender);
    let command = environment.transfer_command(
        &environment.read_endpoint(&format!("{}/", source.display())),
        &environment.read_endpoint(&root.path().join("Copy").display().to_string()),
        SyncMode::transfer(),
    );

    environment.run_sync(&command).expect("the transfer runs");

    let advanced = watched
        .try_iter()
        .filter_map(|observation| environment.observation_progress(&observation))
        .collect::<Vec<_>>();
    assert!(!advanced.is_empty(), "a transfer stated no progress at all");
    assert_eq!(advanced.last().map(|(_, percent)| *percent), Some(100));
}
