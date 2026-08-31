/// The sorts of running work — the type families the attendance classes share.
///
/// A pure carrier trait with no operations. Sharing it links the sorts across every class, so a
/// class never re-declares a sort and a composition never restates one as an equality bound.
pub trait SessionSorts {
    /// Names the request one run works on behalf of.
    type Id;
    /// Represents one thing happening where this program cannot reach into it.
    type Run;
    /// Represents one thing a run states while it is happening.
    type Report;
    /// Represents how a run ended.
    type Ending;
    /// Denotes a run refusing to begin, to be read, or to end.
    type Refusal;
}
