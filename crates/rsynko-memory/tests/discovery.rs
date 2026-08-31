//! What discovery does to a request that asked for something the source turned out not to have.

use rsynko_manager::*;
use rsynko_media::*;
use rsynko_memory::{ManagerState, MediaSyntax, MemoryManager};
use rsynko_ui::*;

#[test]
fn a_tweet_carrying_only_pictures_asks_for_one_by_name() {
    let mut manager = ManagerState::downloads();
    manager.apply_manager_event(ManagerIntentOp::AddSources {
        requests: vec![MemoryManager.submitted("https://x.com/somebody/status/123")],
    });
    let id = manager.selected_id().expect("the request");
    let photo = MediaSyntax.format(
        "photo-1",
        MediaSyntax.metadata([
            (
                FORMAT_SOURCE.to_owned(),
                MediaSyntax.string_metadata("https://pbs.twimg.com/media/A.jpg?name=orig"),
            ),
            (
                FORMAT_EXTENSION.to_owned(),
                MediaSyntax.string_metadata("jpg"),
            ),
            (
                FORMAT_HAS_AUDIO.to_owned(),
                MediaSyntax.boolean_metadata(false),
            ),
            (
                FORMAT_HAS_VIDEO.to_owned(),
                MediaSyntax.boolean_metadata(false),
            ),
        ]),
    );
    manager.apply_manager_event(ManagerIntentOp::FormatCatalog {
        id,
        event: FormatDiscoveryOp::Available {
            formats: vec![photo],
        },
    });
    let entry = manager.entry(id).expect("the request");
    // A picture carries no stream role, so a tweet of photographs offers none: asking it for
    // sound, or for a picture that moves, would select nothing at all.
    assert_eq!(entry.offered_streams(), Vec::new());
    assert_eq!(entry.chosen_choice(), Some("photo-1"));
}
