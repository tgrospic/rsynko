use crate::MediaSorts;
use ambassador::delegatable_trait;

/// Provides the carrier and constructor for produced artifacts.
#[delegatable_trait]
pub trait ArtifactAlg: MediaSorts {
    /// Defines one artifact independently of collection storage.
    fn artifact(&self, id: impl Into<String>, kind: ArtifactKind, metadata: Self::Metadata) -> Self::Artifact;
}

/// Classifies a produced artifact by its semantic role.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ArtifactKind {
    /// Denotes a primary media artifact.
    Media,
    /// Denotes a subtitle artifact.
    Subtitle,
    /// Denotes a thumbnail artifact.
    Thumbnail,
    /// Denotes a description artifact.
    Description,
    /// Denotes an information-JSON artifact.
    InfoJson,
    /// Denotes an internet shortcut artifact.
    Link,
    /// Denotes another explicitly named artifact role.
    Other,
}

/// Names the observation stating where an artifact currently rests.
pub const ARTIFACT_LOCATION: &str = "filepath";
