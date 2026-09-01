#![doc = include_str!("../README.md")]

mod application;
mod challenge;
mod extraction;
mod laws;
mod request;
mod response;
mod sorts;

pub use application::{YoutubeApplicationError, YoutubeApplicationExt};
pub use challenge::{
    DEFAULT_SIGNATURE_PARAMETER, YoutubeChallenge, YoutubeChallengeAlg, YoutubeChallengeExt, YoutubeFormatSource,
    YoutubeGrant, YoutubeSolutionAlg, YoutubeUrlAlg,
};
pub use extraction::{
    WithheldReason, YOUTUBE_DESCRIBED, YOUTUBE_THROTTLED, YOUTUBE_UNREADABLE, YOUTUBE_WITHHELD, YoutubeError,
    YoutubeExtractionExt, YoutubeNotesExt, media_failure, youtube_id,
};
pub use laws::{
    YoutubeApplicationLaws, YoutubeChallengeLawFixture, YoutubeChallengeLaws, YoutubeExtractionLaws, YoutubeLawFixture,
    YoutubeRequestLaws, YoutubeResponseLaws, YoutubeSolutionLaws, YoutubeUrlLaws, withheld_challenge,
};
pub use request::{PlayerClaim, YoutubeClientAlg, YoutubeProgramAlg, YoutubeRequestAlg, YoutubeRequestBytesAlg};
pub use response::{YoutubeFormat, YoutubePlayer, YoutubeResponseAlg, YoutubeWatchPage};
pub use sorts::YoutubeSorts;

// Ambassador's generated delegation macros, re-exported so interpreters can compose.
pub use challenge::{
    ambassador_impl_YoutubeChallengeAlg, ambassador_impl_YoutubeSolutionAlg, ambassador_impl_YoutubeUrlAlg,
};
pub use request::{
    ambassador_impl_YoutubeClientAlg, ambassador_impl_YoutubeProgramAlg, ambassador_impl_YoutubeRequestAlg,
    ambassador_impl_YoutubeRequestBytesAlg,
};
pub use response::ambassador_impl_YoutubeResponseAlg;
