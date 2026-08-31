/// The sorts of tweet meaning — the type families the tweet algebra classes share.
///
/// A pure carrier trait with no operations. Sharing it links the sorts across every class, so a
/// class never re-declares a sort and a composition never restates one as an equality bound.
pub trait XSorts {
    /// Represents exactly the request that asks what one tweet carries.
    type Request;
    /// Represents one file a tweet carries.
    type Attachment;
}
