use crate::YoutubeSorts;
use ambassador::delegatable_trait;

/// Provides the carrier and constructors for Youtube requests.
#[delegatable_trait]
pub trait YoutubeRequestAlg: YoutubeSorts {
    /// Defines watch-page retrieval.
    fn watch_request(&self, url: impl Into<String>) -> Self::Request;
    /// Defines player-catalog retrieval under what the client claims about itself.
    ///
    /// Youtube grants a catalog matching that claim: a request claiming no session and no player
    /// program is answered with the formats such a client may retrieve, or refused outright.
    fn player_request(&self, id: impl Into<String>, api_key: impl Into<String>, claim: &PlayerClaim) -> Self::Request;
    /// Defines player-program retrieval.
    fn player_program_request(&self, url: impl Into<String>) -> Self::Request;
    /// Defines direct-media retrieval.
    fn media_request(&self, url: impl Into<String>) -> Self::Request;
}

/// Denotes what one client claims about itself when it asks for a catalog.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayerClaim {
    /// Names the client making the claim.
    pub client: String,
    /// Names the session the watch page issued to this client, when it issued one.
    pub visitor: Option<String>,
    /// States the signature timestamp of the player program the client runs.
    pub timestamp: Option<i64>,
}

/// Specifies which clients one interpreter can present itself as.
///
/// Youtube grants a different catalog to each client: one states a muxed representation carrying
/// both streams, another states every adaptive representation separately. Neither catalog is the
/// whole truth, so extraction asks under each claim and takes their union.
#[delegatable_trait]
pub trait YoutubeClientAlg {
    /// Names the clients this interpreter can claim to be, in preference order.
    fn player_clients(&self) -> impl Iterator<Item = &str>;
}

/// Specifies observation of what a player program states about itself.
#[delegatable_trait]
pub trait YoutubeProgramAlg {
    /// Observes the signature timestamp the program states, when it states one.
    fn program_timestamp(&self, program: &str) -> Option<i64>;
}

/// Specifies retrieval of bytes denoted by a Youtube request carrier.
#[delegatable_trait]
pub trait YoutubeRequestBytesAlg: YoutubeSorts {
    /// Denotes request execution failure.
    type Error;

    /// Retrieves response bytes without changing request meaning.
    ///
    /// # Errors
    ///
    /// Returns interpreter-specific transport failure.
    fn youtube_request_bytes(&self, request: &Self::Request) -> Result<Vec<u8>, Self::Error>;
}
