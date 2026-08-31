#![doc = include_str!("../README.md")]

mod attachment;
mod bundle;
mod laws;
mod request;
mod sorts;
mod status;

pub use attachment::{AttachmentKind, XAttachmentAlg, XAttachmentViewAlg, attachment_kind};
pub use bundle::{Take, take};
pub use laws::XLaws;
pub use request::{READING_ADDRESS, XRequestAlg, XRequestExt, XRequestViewAlg, reading_token};
pub use sorts::XSorts;
pub use status::{X_HOSTS, status_id};

// Ambassador's generated delegation macros, re-exported so interpreters can compose.
pub use request::ambassador_impl_XRequestAlg;
