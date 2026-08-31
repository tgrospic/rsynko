/// The sorts of media meaning — the Rust encoding of the type families the media algebra classes
/// share.
///
/// A pure carrier trait with no operations. Sharing it links the sorts across every class, so a
/// class never re-declares a sort and a composition never restates one as an equality bound. An
/// interpreter ties the whole knot at once by choosing every sort.
pub trait MediaSorts {
    /// Represents one metadata value.
    type Value;
    /// Represents one metadata record.
    type Metadata;
    /// Represents one alternative representation of a media item.
    type Format;
    /// Represents one produced artifact.
    type Artifact;
    /// Represents one fully described media item.
    type Media;
    /// Represents one extraction result.
    type Extraction;
    /// Represents one extractor identity.
    type Extractor;
    /// Represents one predicate over a format.
    type Predicate;
    /// Represents one format-selection program.
    type Selection;
    /// Represents one portable output name.
    type Output;
}

/// The sorts of artifact-processing meaning.
///
/// Processing is its own calculus: it names transformations of artifacts without speaking about
/// extraction, formats, or selection, so it carries its own sorts.
pub trait ProcessingSorts {
    /// Represents one processor identity.
    type Processor;
    /// Represents one processing step.
    type Step;
    /// Represents one processing program.
    type Program;
}
