//! Every way of transferring, checked against the transfer program itself.
//!
//! A way of transferring is a claim about what a transfer does. These check the claim by making
//! the situation it is about and reading what happened, rather than by reading the arguments back.
//! Where this machine cannot make that situation, the scenario says so instead of pretending.

use rsynko_manager::PlannedChangeAlg;
use rsynko_process::ProcessSyncEnv;
use rsynko_rsync::{
    RsyncEndpointExt, SYNC_PROGRAM, SyncCommandExt, SyncMode, SyncProfile, SyncProgramExt,
};
use std::fs::{File, create_dir_all, read, write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::channel;
use std::time::{Duration, SystemTime};

/// Observes whether the machine running the test has the transfer program.
fn transfers_available() -> bool {
    Command::new(SYNC_PROGRAM)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Denotes one authored situation: a source, a destination, and what happened between them.
struct Situation {
    root: tempfile::TempDir,
}

impl Situation {
    /// Authors an empty source and destination to fill in.
    fn new() -> Self {
        let root = tempfile::tempdir().expect("a place to author folders");
        create_dir_all(root.path().join("source")).expect("a source");
        create_dir_all(root.path().join("destination")).expect("a destination");
        Self { root }
    }

    /// Names one path inside the source.
    fn source(&self, path: &str) -> PathBuf {
        self.root.path().join("source").join(path)
    }

    /// Names one path inside the destination.
    fn destination(&self, path: &str) -> PathBuf {
        self.root.path().join("destination").join(path)
    }

    /// Transfers the source into the destination the stated way, and states what it named.
    fn transferred(&self, profile: SyncProfile) -> Vec<String> {
        let (sender, _watched) = channel();
        let environment = ProcessSyncEnv::new(sender);
        let command = environment.transfer_command(
            &environment.read_endpoint(&format!("{}/", self.source("").display())),
            &environment.read_endpoint(&self.destination("").display().to_string()),
            SyncMode::transfer().profiled(profile),
        );
        environment
            .run_sync(&command)
            .expect("the transfer runs")
            .iter()
            .map(|change| {
                format!(
                    "{:?} {}",
                    change.change_kind(),
                    change.change_path().trim_end_matches('/')
                )
            })
            .collect()
    }
}

/// Writes one file, making the folders that lead to it.
fn wrote(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        create_dir_all(parent).expect("the folders leading to a file");
    }
    write(path, bytes).expect("a file");
}

/// Stamps one file with a time, so what a transfer compares is decided rather than raced.
fn stamped(path: &Path, seconds_ago: u64) {
    let file = File::options()
        .write(true)
        .open(path)
        .expect("a stamped file");
    let stamp = SystemTime::now() - Duration::from_secs(seconds_ago);
    file.set_modified(stamp).expect("a stamped time");
}

/// Authors a source holding one file and a destination holding one nobody asked for.
fn with_an_extra_file() -> Situation {
    let situation = Situation::new();
    wrote(&situation.source("kept.txt"), b"from the source");
    wrote(
        &situation.destination("extra.txt"),
        b"nobody asked for this",
    );
    situation
}

#[test]
fn copy_adds_and_replaces_and_removes_nothing() {
    if !transfers_available() {
        return;
    }
    let situation = with_an_extra_file();

    situation.transferred(SyncProfile::Copy);

    assert_eq!(
        read(situation.destination("kept.txt")).unwrap(),
        b"from the source"
    );
    assert!(situation.destination("extra.txt").exists());
}

#[test]
fn mirror_makes_the_destination_match_exactly() {
    if !transfers_available() {
        return;
    }
    let situation = with_an_extra_file();

    let named = situation.transferred(SyncProfile::Mirror);

    assert!(situation.destination("kept.txt").exists());
    assert!(!situation.destination("extra.txt").exists());
    // What it removed is named, and not only done.
    assert!(named.contains(&"Delete extra.txt".to_owned()), "{named:?}");
}

#[test]
fn mirror_keeping_keeps_what_it_replaces() {
    if !transfers_available() {
        return;
    }
    let situation = with_an_extra_file();
    wrote(&situation.source("replaced.txt"), b"the new text");
    wrote(&situation.destination("replaced.txt"), b"the old text");
    stamped(&situation.destination("replaced.txt"), 3600);

    situation.transferred(SyncProfile::MirrorKeeping);

    assert_eq!(
        read(situation.destination("replaced.txt")).unwrap(),
        b"the new text"
    );
    // The replaced text is kept beside what replaced it.
    assert_eq!(
        read(situation.destination("replaced.txt~")).unwrap(),
        b"the old text"
    );
    assert!(!situation.destination("extra.txt").exists());
}

#[test]
fn mirror_whole_mirrors_by_sending_whole_files() {
    if !transfers_available() {
        return;
    }
    let situation = with_an_extra_file();

    situation.transferred(SyncProfile::MirrorWhole);

    // Sending whole files rather than comparing pieces is a choice about the wire, and leaves no
    // trace at either end: what can be read here is that it mirrored.
    assert_eq!(
        read(situation.destination("kept.txt")).unwrap(),
        b"from the source"
    );
    assert!(!situation.destination("extra.txt").exists());
}

#[cfg(unix)]
#[test]
fn mirror_readable_makes_everything_readable() {
    use std::os::unix::fs::PermissionsExt;

    if !transfers_available() {
        return;
    }
    let situation = with_an_extra_file();
    wrote(&situation.source("private/secret.txt"), b"only mine");
    std::fs::set_permissions(
        situation.source("private/secret.txt"),
        PermissionsExt::from_mode(0o600),
    )
    .expect("a private file");

    situation.transferred(SyncProfile::MirrorReadable);

    let file = std::fs::metadata(situation.destination("private/secret.txt")).expect("the file");
    let folder = std::fs::metadata(situation.destination("private")).expect("the folder");
    assert_eq!(file.permissions().mode() & 0o777, 0o644);
    assert_eq!(folder.permissions().mode() & 0o777, 0o755);
}

#[test]
fn skip_newer_replaces_nothing_newer_at_the_destination() {
    if !transfers_available() {
        return;
    }
    let situation = Situation::new();
    wrote(&situation.source("notes.txt"), b"from the source");
    wrote(&situation.destination("notes.txt"), b"written here, later");
    stamped(&situation.source("notes.txt"), 3600);

    situation.transferred(SyncProfile::SkipNewer);

    assert_eq!(
        read(situation.destination("notes.txt")).unwrap(),
        b"written here, later"
    );
}

#[test]
fn compare_content_replaces_what_only_its_content_distinguishes() {
    if !transfers_available() {
        return;
    }
    // The two differ in content alone: same length, same time, so only reading them tells.
    let author = || {
        let situation = Situation::new();
        wrote(&situation.source("report.txt"), b"the true text");
        // The same length, so a transfer comparing sizes cannot tell them apart either.
        wrote(&situation.destination("report.txt"), b"the fake text");
        stamped(&situation.source("report.txt"), 3600);
        stamped(&situation.destination("report.txt"), 3600);
        situation
    };

    let unread = author();
    unread.transferred(SyncProfile::Copy);
    let read_through = author();
    read_through.transferred(SyncProfile::CompareContent);

    // Comparing when they changed cannot tell them apart; comparing what they hold can.
    assert_eq!(
        read(unread.destination("report.txt")).unwrap(),
        b"the fake text"
    );
    assert_eq!(
        read(read_through.destination("report.txt")).unwrap(),
        b"the true text"
    );
}

#[test]
fn resume_continues_a_file_where_it_left_off() {
    if !transfers_available() {
        return;
    }
    let situation = Situation::new();
    let whole = (0..64_u8).cycle().take(256 * 1024).collect::<Vec<_>>();
    wrote(&situation.source("large.bin"), &whole);
    // What arrived before the transfer was interrupted is the beginning of the file.
    wrote(&situation.destination("large.bin"), &whole[..64 * 1024]);
    stamped(&situation.destination("large.bin"), 3600);

    situation.transferred(SyncProfile::Resume);

    assert_eq!(read(situation.destination("large.bin")).unwrap(), whole);
}

#[test]
fn move_removes_from_the_source_what_arrived_safely() {
    if !transfers_available() {
        return;
    }
    let situation = Situation::new();
    wrote(&situation.source("staged/one.txt"), b"one");
    wrote(&situation.source("staged/two.txt"), b"two");

    situation.transferred(SyncProfile::Move);

    assert_eq!(
        read(situation.destination("staged/one.txt")).unwrap(),
        b"one"
    );
    assert_eq!(
        read(situation.destination("staged/two.txt")).unwrap(),
        b"two"
    );
    assert!(!situation.source("staged/one.txt").exists());
    assert!(!situation.source("staged/two.txt").exists());
    // The folders stay: only what arrived is taken away.
    assert!(situation.source("staged").exists());
}

#[test]
fn limit_rate_transfers_what_it_was_given() {
    if !transfers_available() {
        return;
    }
    let situation = with_an_extra_file();

    situation.transferred(SyncProfile::LimitRate);

    // How fast it went is not something a finished transfer states, so what is read here is that
    // limiting the rate did not change what arrived.
    assert_eq!(
        read(situation.destination("kept.txt")).unwrap(),
        b"from the source"
    );
    assert!(situation.destination("extra.txt").exists());
}

#[cfg(unix)]
#[test]
fn keep_marks_keeps_the_links_between_files() {
    use std::os::unix::fs::MetadataExt;

    if !transfers_available() {
        return;
    }
    let situation = Situation::new();
    wrote(&situation.source("original.txt"), b"one file, two names");
    std::fs::hard_link(
        situation.source("original.txt"),
        situation.source("same.txt"),
    )
    .expect("two names for one file");

    situation.transferred(SyncProfile::KeepMarks);

    let original = std::fs::metadata(situation.destination("original.txt")).expect("the original");
    let same = std::fs::metadata(situation.destination("same.txt")).expect("the other name");
    // One file arrived, under both of its names, rather than two files that look alike.
    assert_eq!(original.ino(), same.ino());
    assert_eq!(original.nlink(), 2);
}

#[test]
fn one_disk_transfers_what_it_was_given() {
    if !transfers_available() {
        return;
    }
    let situation = with_an_extra_file();

    situation.transferred(SyncProfile::OneDisk);

    // Staying on one disk is only observable where another is mounted underneath, which a
    // temporary folder cannot arrange: what is read here is that it transferred.
    assert_eq!(
        read(situation.destination("kept.txt")).unwrap(),
        b"from the source"
    );
}

#[test]
fn skip_junk_leaves_behind_what_no_one_meant_to_keep() {
    if !transfers_available() {
        return;
    }
    let situation = Situation::new();
    wrote(&situation.source("keep.txt"), b"wanted");
    wrote(&situation.source(".DS_Store"), b"junk");
    wrote(&situation.source("Thumbs.db"), b"junk");
    wrote(&situation.source("draft.tmp"), b"junk");
    wrote(&situation.source("deep/other.tmp"), b"junk");

    situation.transferred(SyncProfile::SkipJunk);

    assert!(situation.destination("keep.txt").exists());
    for junk in [".DS_Store", "Thumbs.db", "draft.tmp", "deep/other.tmp"] {
        assert!(!situation.destination(junk).exists(), "{junk} arrived");
    }
}
