//! Verifies the compiled fixture command gate.

use rsynko_reqwest::FIXTURE_BYTES;
use std::process::Command;

#[test]
fn command_downloads_the_fixture_to_its_semantic_default_path() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let output = Command::new(env!("CARGO_BIN_EXE_rsynko"))
        .arg("fixture://single-video")
        .current_dir(directory.path())
        .output()
        .expect("run rsynko");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::fs::read(directory.path().join("single-video.mp4")).expect("final file"),
        FIXTURE_BYTES
    );
    assert!(!directory.path().join("single-video.mp4.part").exists());
    assert_eq!(
        String::from_utf8(output.stdout)
            .expect("UTF-8 output")
            .lines()
            .filter(|line| line.starts_with("download succeeded:"))
            .count(),
        1
    );
}
