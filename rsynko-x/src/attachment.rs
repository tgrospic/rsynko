use crate::XSorts;
use alux_sdk::case_mapping;

/// Provides the carrier and constructor for one file a tweet carries.
pub trait XAttachmentAlg: XSorts {
    /// Defines one attachment from what the answer stated about it.
    fn attachment(
        &self,
        identity: impl Into<String>,
        kind: AttachmentKind,
        address: impl Into<String>,
    ) -> Self::Attachment;
}

/// Specifies what one attachment states about itself.
pub trait XAttachmentViewAlg: XSorts {
    /// Observes what the attachment is called among the others the tweet carries.
    fn attachment_identity<'a>(&self, attachment: &'a Self::Attachment) -> &'a str;
    /// Observes what kind of file it is.
    fn attachment_kind(&self, attachment: &Self::Attachment) -> AttachmentKind;
    /// Observes where its bytes are fetched from.
    fn attachment_address<'a>(&self, attachment: &'a Self::Attachment) -> &'a str;
}

/// Denotes what kind of file a tweet carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttachmentKind {
    /// Denotes a still picture.
    Photo,
    /// Denotes a video, which the answer states at several sizes.
    Video,
    /// Denotes a silent looping video, which was posted as an animation.
    Animation,
}

case_mapping! {
    AttachmentKind, &'static str as &str,
        Photo     <=> "photo",
        Video     <=> "video",
        Animation <=> "animated_gif",
}
