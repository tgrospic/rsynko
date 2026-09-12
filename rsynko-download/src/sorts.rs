use ambassador::delegatable_trait;

/// The sorts of download observation, the type families the observation classes share.
///
/// A pure carrier trait with no operations. Sharing it links the sorts across every class, so a
/// class never re-declares a sort and a composition never restates one as an equality bound.
#[delegatable_trait]
pub trait DownloadSorts {
    /// Represents one terminal download event.
    type Event;
    /// Represents one progress observation.
    type Progress;
}
