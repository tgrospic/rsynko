/// The sorts of manager meaning — the type families the manager algebra classes share.
///
/// A pure carrier trait with no operations. Sharing it links the sorts across every class, so a
/// class never re-declares a sort and a composition never restates one as an equality bound.
pub trait ManagerSorts {
    /// Represents one stable queue identity.
    type Id;
    /// Represents one submitted source.
    type Source;
    /// Represents the options fixed for one request.
    type Options;
    /// Represents one chosen output.
    type Output;
    /// Represents one selectable format description.
    type Format;
    /// Represents one change a rehearsal states.
    type Change;
    /// Represents one observable queue entry.
    type Entry;
    /// Represents the downloads collection.
    type Downloads;
}
