use crate::AttachmentKind;
use alux_sdk::case_mapping;

/// Selects which of the files a tweet carries are taken from it.
///
/// A tweet is a bundle, so what to take is a choice like any other: a reader picks one of these,
/// or picks one file out of the bundle by name. Each of these names what it *does* — not what it
/// is for — and states exactly which kinds it accepts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Take {
    /// Takes every file the tweet carries.
    Everything,
    /// Takes the videos and animations, and leaves the pictures.
    Videos,
    /// Takes the pictures, and leaves the videos.
    Images,
}

case_mapping! {
    Take, &'static str as &str,
        Everything <=> "everything",
        Videos     <=> "video",
        Images     <=> "images",
}

impl Take {
    /// States what taking this way does, and then what that is usually wanted for.
    ///
    /// A reader chooses by what a way of taking does, and confirms the choice by what it is good
    /// for, so both are stated and in that order.
    #[must_use]
    pub const fn summary(self) -> &'static str {
        match self {
            Self::Everything => "takes every file the tweet carries, good for keeping the whole post",
            Self::Videos => "takes the videos only, good for clips",
            Self::Images => "takes the pictures only, good for photographs",
        }
    }

    /// States whether taking this way accepts one kind of file.
    #[must_use]
    pub const fn accepts(self, kind: AttachmentKind) -> bool {
        match self {
            Self::Everything => true,
            // An animation is a video that was posted as a picture, and is fetched as a video.
            Self::Videos => matches!(kind, AttachmentKind::Video | AttachmentKind::Animation),
            Self::Images => matches!(kind, AttachmentKind::Photo),
        }
    }
}
