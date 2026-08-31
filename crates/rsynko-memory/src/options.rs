//! The layout of one request's fixed choices and of the formats discovered for it.

use crate::{Format, InfoValue, MediaSyntax, predicate_accepts};
use rsynko_manager::{MediaStreams, MediaStreamsExt, Performer};
use rsynko_rsync::SyncProfile;
use rsynko_ui::FormatDescriptionAlg;

/// Denotes editable choices for one not-yet-started download.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DownloadOptions {
    pub(crate) media_streams: Option<MediaStreams>,
    pub(crate) format: FormatChoice,
    pub(crate) profile: Option<SyncProfile>,
    pub(crate) dry_run: Option<bool>,
}

impl DownloadOptions {
    /// Selects the best progressive format containing audio and video.
    #[must_use]
    pub const fn progressive() -> Self {
        Self {
            media_streams: Some(MediaStreams::AudioVideo),
            format: FormatChoice::Best,
            profile: None,
            dry_run: None,
        }
    }

    /// Selects the best audio-only format.
    #[must_use]
    pub const fn audio() -> Self {
        Self {
            media_streams: Some(MediaStreams::Audio),
            format: FormatChoice::Best,
            profile: None,
            dry_run: None,
        }
    }

    /// Selects the best video-only format.
    #[must_use]
    pub const fn video() -> Self {
        Self {
            media_streams: Some(MediaStreams::Video),
            format: FormatChoice::Best,
            profile: None,
            dry_run: None,
        }
    }

    /// Selects one transferred path, rehearsed before it is transferred.
    ///
    /// A transfer states what it would do before it does it, so it begins in rehearsal mode and
    /// only leaves it when someone turns that mode off. What is transferred may be one file or a
    /// whole folder; the request is the same either way.
    #[must_use]
    pub const fn transfer() -> Self {
        Self {
            media_streams: None,
            format: FormatChoice::Best,
            profile: Some(SyncProfile::Copy),
            dry_run: Some(true),
        }
    }

    /// Observes the way a path is transferred, and that a media item is transferred no way.
    #[must_use]
    pub const fn profile(&self) -> Option<SyncProfile> {
        self.profile
    }

    /// Observes what performs the request these options belong to.
    ///
    /// A way of transferring names a program to run; everything else is retrieved from the source
    /// that claimed the line.
    #[must_use]
    pub const fn performer(&self) -> Performer {
        if self.profile.is_some() {
            Performer::Program
        } else {
            Performer::Retrieval
        }
    }

    /// Selects the way a path is transferred, for a request that transfers one.
    #[must_use]
    pub const fn with_profile(mut self, profile: SyncProfile) -> Self {
        if self.profile.is_some() {
            self.profile = Some(profile);
        }
        self
    }

    /// Turns the rehearsal mode back on, for a request that has one.
    ///
    /// A request nobody has performed yet states what it would do before it does it, whether it
    /// was submitted or duplicated from one somebody had already armed.
    #[must_use]
    pub const fn rehearsing(mut self) -> Self {
        if self.dry_run.is_some() {
            self.dry_run = Some(true);
        }
        self
    }

    /// Observes the rehearsal mode, and that the request has none when it has none.
    #[must_use]
    pub const fn dry_run(&self) -> Option<bool> {
        self.dry_run
    }

    /// Selects the rehearsal mode of a request that has one.
    pub const fn set_dry_run(&mut self, dry_run: bool) {
        if self.dry_run.is_some() {
            self.dry_run = Some(dry_run);
        }
    }

    /// Observes the selected stream roles.
    #[must_use]
    pub const fn media_streams(&self) -> Option<MediaStreams> {
        self.media_streams
    }

    /// Observes the selected format policy.
    #[must_use]
    pub const fn format(&self) -> &FormatChoice {
        &self.format
    }

    /// Selects new stream roles and releases the concrete format identity.
    #[must_use]
    pub fn with_media_streams(mut self, media_streams: MediaStreams) -> Self {
        self.media_streams = Some(media_streams);
        self.format = FormatChoice::Best;
        self
    }

    /// Selects one format policy without changing stream roles.
    #[must_use]
    pub fn with_format(mut self, format: FormatChoice) -> Self {
        self.format = format;
        self
    }
}

/// Selects either the best matching format or one concrete format identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormatChoice {
    /// Selects the most preferred format with the requested stream roles.
    Best,
    /// Selects one extractor-defined format identity.
    Id(String),
}

/// Denotes discovery state for selectable formats.
#[derive(Clone, Debug, PartialEq)]
pub enum FormatCatalog {
    /// Denotes a source whose formats have not been requested.
    Unknown,
    /// Denotes a request waiting for an interpreter to inspect the source.
    Waiting,
    /// Denotes source inspection currently in progress.
    Inspecting,
    /// Denotes the discovered formats in extractor preference order.
    Available(Vec<Format>),
    /// Denotes an inspection failure suitable for display.
    Failed(String),
}

impl FormatCatalog {
    /// Observes discovered formats when inspection succeeded.
    #[must_use]
    pub fn available(&self) -> Option<&[Format]> {
        match self {
            Self::Available(formats) => Some(formats),
            Self::Unknown | Self::Waiting | Self::Inspecting | Self::Failed(_) => None,
        }
    }
}

/// Observes whether one described format carries exactly the streams a role requires.
#[must_use]
pub fn role_accepts(streams: MediaStreams, format: &Format) -> bool {
    predicate_accepts(&MediaSyntax.stream_role_format(streams), format)
}

impl FormatDescriptionAlg for Format {
    fn format_identity(&self) -> &str {
        &self.id
    }

    fn format_extension(&self) -> Option<&str> {
        self.extension()
    }

    fn format_streams(&self) -> Option<MediaStreams> {
        match (self.has_audio(), self.has_video()) {
            (true, true) => Some(MediaStreams::AudioVideo),
            (true, false) => Some(MediaStreams::Audio),
            (false, true) => Some(MediaStreams::Video),
            (false, false) => None,
        }
    }

    fn format_quality(&self) -> Option<&str> {
        self.text("quality")
    }

    fn format_height(&self) -> Option<u64> {
        format_count(self, "height")
    }

    fn format_width(&self) -> Option<u64> {
        format_count(self, "width")
    }

    fn format_bitrate(&self) -> Option<u64> {
        format_count(self, "bitrate")
    }

    fn format_size(&self) -> Option<u64> {
        format_count(self, "content_length")
    }

    fn format_codecs(&self) -> Option<&str> {
        self.text("codecs")
    }
}

/// Observes one counted observation of a format, which only a whole number states.
fn format_count(format: &Format, key: &str) -> Option<u64> {
    match format.observe(key) {
        Some(InfoValue::Integer(value)) => u64::try_from(*value).ok(),
        Some(
            InfoValue::Null
            | InfoValue::Bool(_)
            | InfoValue::Float(_)
            | InfoValue::String(_)
            | InfoValue::List(_)
            | InfoValue::Record(_),
        )
        | None => None,
    }
}
