use crate::{YoutubeFormat, YoutubeSorts};
use alux_ext::ext;
use ambassador::delegatable_trait;

/// Specifies the query surface through which Youtube poses and answers URL challenges.
#[delegatable_trait]
pub trait YoutubeUrlAlg {
    /// Observes the throttling challenge a representation URL carries.
    fn throttle_challenge(&self, url: &str) -> Option<String>;

    /// Replaces the throttling parameter with its solution.
    fn with_throttle(&self, url: &str, solution: &str) -> String;

    /// Attaches a solved signature under the parameter naming it.
    fn with_signature(&self, url: &str, parameter: &str, signature: &str) -> String;
}

/// Specifies resolution of every challenge one catalog poses in a single application.
#[delegatable_trait]
pub trait YoutubeChallengeAlg: YoutubeSorts {
    /// Denotes resolution failure.
    type Error;

    /// Resolves the given challenges together under the program that poses them.
    ///
    /// A challenge is posed by one player program, so a solution is stated relative to that
    /// program and says nothing about the challenges another program poses.
    ///
    /// # Errors
    ///
    /// Returns the interpreter-specific resolution failure.
    fn solve_challenges(
        &self,
        program: &str,
        challenges: impl IntoIterator<Item = YoutubeChallenge>,
    ) -> Result<Self::Solutions, Self::Error>;
}

/// Specifies observation of one individually resolved challenge.
#[delegatable_trait]
pub trait YoutubeSolutionAlg: YoutubeSorts {
    /// Observes the solution of one challenge exactly when that challenge was resolved.
    fn solution_of(&self, solutions: &Self::Solutions, challenge: &YoutubeChallenge) -> Option<String>;
}

/// Names the query parameter carrying a solved signature when a format states no other.
pub const DEFAULT_SIGNATURE_PARAMETER: &str = "signature";

/// Denotes one obfuscated value that only the Youtube player program resolves.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum YoutubeChallenge {
    /// Guards access to one format representation.
    Signature(String),
    /// Governs the rate at which a granted representation is served.
    Throttle(String),
}

impl YoutubeChallenge {
    /// Observes the obfuscated value this challenge poses.
    #[must_use]
    pub fn value(&self) -> &str {
        match self {
            Self::Signature(value) | Self::Throttle(value) => value,
        }
    }
}

/// Denotes how one format states the location of its representation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum YoutubeFormatSource {
    /// States a retrievable URL directly.
    Direct(String),
    /// States a URL granting retrieval only under a solved signature.
    Signed {
        /// Locates the representation before the signature is attached.
        url: String,
        /// Poses the signature challenge guarding the representation.
        signature: String,
        /// Names the query parameter carrying the solved signature.
        parameter: String,
    },
}

impl YoutubeFormatSource {
    /// Locates the representation before any challenge is answered.
    #[must_use]
    pub fn url(&self) -> &str {
        match self {
            Self::Direct(url) | Self::Signed { url, .. } => url,
        }
    }

    /// Observes the signature challenge guarding this source.
    #[must_use]
    pub fn signature_challenge(&self) -> Option<YoutubeChallenge> {
        match self {
            Self::Direct(_) => None,
            Self::Signed { signature, .. } => Some(YoutubeChallenge::Signature(signature.clone())),
        }
    }
}

/// Denotes the outcome of granting one format its retrievable representation.
///
/// The two challenges guard different things. A signature guards access: unanswered, the
/// representation cannot be retrieved at all. A throttling parameter governs the rate at which a
/// granted representation is served: unanswered, the representation is still retrievable, only
/// slowly. Granting therefore withholds one and throttles the other.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum YoutubeGrant {
    /// Grants retrieval at a URL whose every challenge is answered.
    Granted {
        /// Identifies the granted format by `itag`.
        id: String,
        /// Locates the fully answered representation.
        url: String,
    },
    /// Grants retrieval at a rate the unanswered throttling parameter governs.
    Throttled {
        /// Identifies the throttled format by `itag`.
        id: String,
        /// Locates the representation, still posing its throttling parameter.
        url: String,
        /// Names the throttling challenge that stayed unresolved.
        challenge: YoutubeChallenge,
    },
    /// Withholds retrieval because the signature guarding it remains unresolved.
    Withheld {
        /// Identifies the withheld format by `itag`.
        id: String,
        /// Names the challenge that remains unresolved.
        challenge: YoutubeChallenge,
    },
}

impl YoutubeGrant {
    /// Identifies the format this outcome speaks about.
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::Granted { id, .. } | Self::Throttled { id, .. } | Self::Withheld { id, .. } => id,
        }
    }

    /// Locates the representation exactly when it is retrievable.
    #[must_use]
    pub fn retrievable(&self) -> Option<&str> {
        match self {
            Self::Granted { url, .. } | Self::Throttled { url, .. } => Some(url),
            Self::Withheld { .. } => None,
        }
    }
}

/// Derives bulk challenge resolution and per-format granting from primitive capabilities.
#[ext(name = YoutubeChallengeExt)]
pub impl<This, ChallengeError> This
where
    This: YoutubeChallengeAlg<Error = ChallengeError> + YoutubeSolutionAlg + YoutubeUrlAlg,
{
    /// Observes every distinct challenge the formats pose, in first-appearance order.
    fn format_challenges<'a>(
        &'a self,
        formats: impl IntoIterator<Item = &'a YoutubeFormat> + 'a,
    ) -> impl Iterator<Item = YoutubeChallenge> + 'a {
        let mut posed = Vec::new();
        formats
            .into_iter()
            .flat_map(|format| {
                [
                    self.throttle_challenge(format.source.url()).map(YoutubeChallenge::Throttle),
                    format.source.signature_challenge(),
                ]
            })
            .flatten()
            .filter(move |challenge| {
                let fresh = !posed.contains(challenge);
                if fresh {
                    posed.push(challenge.clone());
                }
                fresh
            })
    }

    /// Grants one format its retrievable URL under already resolved challenges.
    fn grant_format(&self, format: &YoutubeFormat, solutions: &This::Solutions) -> YoutubeGrant {
        let id = format.id.clone();
        let mut url = format.source.url().to_owned();
        let mut throttled = None;
        if let Some(value) = self.throttle_challenge(&url) {
            let challenge = YoutubeChallenge::Throttle(value);
            match self.solution_of(solutions, &challenge) {
                Some(solution) => url = self.with_throttle(&url, &solution),
                None => throttled = Some(challenge),
            }
        }
        if let YoutubeFormatSource::Signed { signature, parameter, .. } = &format.source {
            let challenge = YoutubeChallenge::Signature(signature.clone());
            match self.solution_of(solutions, &challenge) {
                Some(solution) => url = self.with_signature(&url, parameter, &solution),
                None => return YoutubeGrant::Withheld { id, challenge },
            }
        }
        throttled.map_or_else(
            || YoutubeGrant::Granted { id: id.clone(), url: url.clone() },
            |challenge| YoutubeGrant::Throttled { id: id.clone(), url: url.clone(), challenge },
        )
    }

    /// Resolves the challenges one program poses once, and grants every format in order.
    ///
    /// # Errors
    ///
    /// Returns the interpreter-specific resolution failure.
    fn grant_formats(&self, program: &str, formats: &[YoutubeFormat]) -> Result<Vec<YoutubeGrant>, ChallengeError> {
        let solutions = self.solve_challenges(program, self.format_challenges(formats))?;
        Ok(formats.iter().map(|format| self.grant_format(format, &solutions)).collect())
    }
}
