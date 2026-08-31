//! Verifies the deterministic runtime download gate.

use rsynko_media::{ApplicationExt, FormatSelectionProgramExt, OutputTarget};
use rsynko_memory::{DownloadEvent, MediaSyntax};
use rsynko_reqwest::{FIXTURE_BYTES, RuntimeEnvironment};

#[test]
fn fixture_download_has_exact_bytes_one_success_and_no_partial_file() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let destination = directory.path().join("single-video.mp4");
    let partial = directory.path().join("single-video.mp4.part");
    let environment = RuntimeEnvironment::build().expect("runtime environment");
    let progressive = MediaSyntax.best_progressive_format();

    let published = environment
        .download_url(
            "fixture://single-video",
            &progressive,
            &OutputTarget::Path(destination.clone()),
        )
        .expect("fixture download");

    assert_eq!(published, destination);
    assert_eq!(
        std::fs::read(&published).expect("published bytes"),
        FIXTURE_BYTES
    );
    assert!(!partial.exists());
    assert_eq!(
        environment
            .progress()
            .iter()
            .map(|item| item.downloaded)
            .collect::<Vec<_>>(),
        vec![
            0,
            u64::try_from(FIXTURE_BYTES.len()).expect("fixture byte count")
        ]
    );
    assert_eq!(
        environment.events(),
        vec![DownloadEvent::Succeeded {
            destination: published,
            bytes: u64::try_from(FIXTURE_BYTES.len()).expect("fixture byte count"),
        }]
    );
}
