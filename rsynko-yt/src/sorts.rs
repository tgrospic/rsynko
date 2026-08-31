/// The sorts of Youtube meaning — the type families the Youtube algebra classes share.
///
/// A pure carrier trait with no operations. Sharing it links the sorts across every class, so a
/// class never re-declares a sort and a composition never restates one as an equality bound.
pub trait YoutubeSorts {
    /// Represents one Youtube request.
    type Request;
    /// Represents the challenges resolved by one bulk application.
    type Solutions;
}
