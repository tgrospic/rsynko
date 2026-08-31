use crate::YoutubeFormatSource;
use ambassador::delegatable_trait;

/// Specifies decoding transport bytes into Youtube response observations.
#[delegatable_trait]
pub trait YoutubeResponseAlg {
    /// Denotes decoding failure.
    type Error;

    /// Decodes one watch page.
    ///
    /// # Errors
    ///
    /// Returns interpreter-specific decoding failure.
    fn decode_youtube_watch(&self, bytes: &[u8]) -> Result<YoutubeWatchPage, Self::Error>;

    /// Decodes one player response.
    ///
    /// # Errors
    ///
    /// Returns interpreter-specific decoding failure.
    fn decode_youtube_player(&self, bytes: &[u8]) -> Result<YoutubePlayer, Self::Error>;
}

/// Denotes the Youtube observations recovered from a watch page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct YoutubeWatchPage {
    /// Preserves whether an initial player response was present and parseable.
    pub has_player_response: bool,
    /// Names the player API key when present.
    pub api_key: Option<String>,
    /// Locates the player program the page states, when it states one.
    pub player_url: Option<String>,
    /// Names the session the page issued to this client, when it issued one.
    pub visitor_data: Option<String>,
    /// Preserves the initial player title when present.
    pub title: Option<String>,
}

/// Denotes one format described by the Youtube player response.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct YoutubeFormat {
    /// Identifies the format by `itag`.
    pub id: String,
    /// States how its media representation is located and what it is guarded by.
    pub source: YoutubeFormatSource,
    /// Names its container extension when known.
    pub extension: Option<String>,
    /// Denotes whether the representation contains audio.
    pub has_audio: bool,
    /// Denotes whether the representation contains video.
    pub has_video: bool,
    /// Preserves the displayed quality when known.
    pub quality: Option<String>,
    /// Preserves coded width when known.
    pub width: Option<i64>,
    /// Preserves coded height when known.
    pub height: Option<i64>,
    /// Preserves average bitrate when known.
    pub bitrate: Option<i64>,
    /// Preserves expected bytes when known.
    pub content_length: Option<i64>,
    /// Preserves codec names when known.
    pub codecs: Option<String>,
}

/// Denotes the Youtube observations recovered from a player response.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct YoutubePlayer {
    /// Names the playability status.
    pub status: Option<String>,
    /// Explains an unavailable status when supplied.
    pub reason: Option<String>,
    /// Names the media title when supplied.
    pub title: Option<String>,
    /// Preserves described formats before challenge resolution and preference ordering.
    pub formats: Vec<YoutubeFormat>,
    /// Counts the described formats this interpreter could not read.
    ///
    /// A response may describe a format this interpreter cannot use — one stating no location, or
    /// carrying neither audio nor video. Counting them keeps the difference between what the
    /// response described and what the catalog offers an observation rather than a silence.
    pub unreadable: usize,
}
