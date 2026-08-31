# rsynko-memory

`rsynko-memory` provides deterministic, inspectable in-memory interpreters for the Rsynko specifications. It is a semantic witness and test model, not an application framework.

Reference extractors use explicit URL prefixes and fixed outcomes. Reference processors use a closed transformation language. Neither hides policy inside callbacks.

The in-memory manager interprets the associated manager carriers as `ManagerState`, `SourceRequest`, and stable numeric `QueueId` values. This representation belongs here rather than in the manager specification:

```rust
use rsynko_manager::ProgressiveDownloadsExt;
use rsynko_memory::MemoryManager;

let manager = MemoryManager.progressive_downloads([
    "fixture://single-video",
    "https://www.youtube.com/watch?v=VIDEO_ID",
]);
assert_eq!(manager.queue().len(), 2);
```

Interactive event interpretation uses the same concrete carrier:

```rust
use rsynko_manager::{ManagerIntentExt, ManagerIntentOp, ManagerPage};
use rsynko_memory::{DownloadOptions, ManagerState, SourceRequest};

let mut manager = ManagerState::downloads();
manager.apply_manager_event(ManagerIntentOp::AddSources {
    requests: vec![SourceRequest::new(
        "fixture://single-video",
        None,
        DownloadOptions::progressive(),
    )],
});
manager.apply_manager_event(ManagerIntentOp::OpenSelected {});

assert!(matches!(manager.page(), ManagerPage::Details(_)));
assert_eq!(manager.queue().len(), 1);
```

```rust
use rsynko_media::ExtractionExt;
use rsynko_memory::{Extraction, InfoRecord, Media};
use rsynko_memory::{ReferenceExtractor, ReferenceExtractorRegistry};

let result = Extraction::Media(Media::new("42".to_owned(), InfoRecord::default(), Vec::default()));
let mut registry = ReferenceExtractorRegistry::default();
registry.push(ReferenceExtractor::succeeds(
    "example",
    "https://example.test/",
    result.clone(),
));
assert_eq!(registry.extract_url("https://example.test/42"), Ok(result));
```
