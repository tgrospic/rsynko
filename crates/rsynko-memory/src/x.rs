use rsynko_x::*;

/// Asks about tweets by writing down the address that would be asked.
///
/// Nothing here reaches anywhere: a request is the address itself, which is what makes what a
/// tweet source decides before it asks anything readable without asking anything.
#[derive(Clone, Copy, Debug, Default)]
pub struct ReferenceXEnv;

/// Holds one file a tweet carries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XAttachment {
    identity: String,
    kind: AttachmentKind,
    address: String,
}

impl XSorts for ReferenceXEnv {
    type Request = String;
    type Attachment = XAttachment;
}

impl XRequestAlg for ReferenceXEnv {
    fn tweet_request(&self, address: impl Into<String>) -> String {
        address.into()
    }
}

impl XRequestViewAlg for ReferenceXEnv {
    fn request_address<'a>(&self, request: &'a String) -> &'a str {
        request
    }
}

impl XAttachmentAlg for ReferenceXEnv {
    fn attachment(&self, identity: impl Into<String>, kind: AttachmentKind, address: impl Into<String>) -> XAttachment {
        XAttachment { identity: identity.into(), kind, address: address.into() }
    }
}

impl XAttachmentViewAlg for ReferenceXEnv {
    fn attachment_identity<'a>(&self, attachment: &'a XAttachment) -> &'a str {
        &attachment.identity
    }

    fn attachment_kind(&self, attachment: &XAttachment) -> AttachmentKind {
        attachment.kind
    }

    fn attachment_address<'a>(&self, attachment: &'a XAttachment) -> &'a str {
        &attachment.address
    }
}
