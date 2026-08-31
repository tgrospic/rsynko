/// The sorts of transfer meaning — the type families the transfer algebra classes share.
///
/// A pure carrier trait with no operations. Sharing it links the sorts across every class, so a
/// class never re-declares a sort and a composition never restates one as an equality bound.
pub trait RsyncSorts {
    /// Represents one end of a transfer.
    type Endpoint;
    /// Represents exactly the command one transfer runs.
    type Command;
    /// Represents one thing a running transfer states.
    type Observation;
    /// Represents one path a transfer would change.
    type Change;
}
