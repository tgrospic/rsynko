//! Reusable law bundles for Youtube challenge resolution, stated over the capabilities.

use crate::*;
use alux_ext::ext;
use anyhow::{Result, bail};
use rsynko_download::*;
use rsynko_media::*;
use std::fmt::{Debug, Display};
use std::path::PathBuf;

/// Supplies the resolved challenges and the application trace a granting scenario cannot author.
pub trait YoutubeChallengeLawFixture {
    /// Resolves one challenge to the value the player program would produce for it.
    fn solve_law_challenge(&mut self, challenge: YoutubeChallenge, solution: &str);
    /// Observes the bulk applications the interpreter received, in application order.
    fn law_challenge_applications(&self) -> Vec<Vec<YoutubeChallenge>>;
}

/// Supplies the transport bytes and request trace a Youtube scenario cannot author for itself.
pub trait YoutubeLawFixture: YoutubeSorts {
    /// Names a watch URL the interpreter answers.
    fn law_watch_url(&self) -> String;
    /// States the video identity that URL denotes.
    fn law_video_id(&self) -> String;
    /// States watch-page bytes describing a playable video.
    fn law_watch_bytes(&self) -> Vec<u8>;
    /// States player bytes describing one retrievable format.
    fn law_player_bytes(&self) -> Vec<u8>;
    /// Observes the requests executed so far, in execution order.
    fn law_executed_requests(&self) -> Vec<Self::Request>;
    /// Observes the requests retrieval opened, in retrieval order.
    fn law_retrieved_requests(&self) -> Vec<Self::Request>;
}

/// Checks the query-surface laws against any interpreter of Youtube URL mechanics.
#[ext(name = YoutubeUrlLaws)]
pub impl<This> This
where
    This: YoutubeUrlAlg,
{
    /// Checks that the two substitutions are independent and that answering is observable.
    ///
    /// The laws checked are:
    ///
    /// 1. a solved throttling parameter is the one the answered URL then poses;
    /// 2. attaching a solved signature leaves the throttling challenge unchanged.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn youtube_url_laws(&self) -> Result<()> {
        let url = "https://media.example/video.mp4?n=posed";
        let parameter = DEFAULT_SIGNATURE_PARAMETER;
        let answered = self.with_throttle(url, "answered");
        if self.throttle_challenge(&answered).as_deref() != Some("answered") {
            bail!("a solved throttling parameter is not the one the answered URL poses");
        }
        let signed = self.with_signature(url, parameter, "solved");
        if self.throttle_challenge(&signed) != self.throttle_challenge(url) {
            bail!("attaching a signature disturbed the throttling challenge");
        }
        Ok(())
    }
}

/// Checks the granting laws against any interpreter that resolves Youtube challenges.
#[ext(name = YoutubeChallengeLaws)]
pub impl<This, ChallengeError> This
where
    This: YoutubeChallengeAlg<Error = ChallengeError> + YoutubeSolutionAlg + YoutubeUrlAlg,
{
    /// Checks that granting is total, ordered, and withholds only what stays unresolved.
    ///
    /// The laws checked are:
    ///
    /// 1. granting yields exactly one outcome per described format, in declaration order;
    /// 2. a source posing no challenge is granted its stated URL unchanged;
    /// 3. an unresolved signature withholds exactly the formats it guards, and no others;
    /// 4. an unresolved throttling parameter governs the rate of a still retrievable URL.
    ///
    /// # Errors
    ///
    /// Returns the first violated law, or the interpreter's resolution failure.
    fn youtube_granting_laws(&self) -> Result<()>
    where
        ChallengeError: Debug,
    {
        let formats = [
            law_format("18", YoutubeFormatSource::Direct("https://media.example/a.mp4".to_owned())),
            law_format(
                "137",
                YoutubeFormatSource::Signed {
                    url: "https://media.example/b.mp4".to_owned(),
                    signature: "obfuscated".to_owned(),
                    parameter: DEFAULT_SIGNATURE_PARAMETER.to_owned(),
                },
            ),
            law_format("22", YoutubeFormatSource::Direct("https://media.example/c.mp4?n=posed".to_owned())),
        ];
        let formats = &formats;
        let grants = match self.grant_formats(law_program(), formats) {
            Ok(grants) => grants.into_iter().collect::<Vec<_>>(),
            Err(error) => bail!("challenge resolution failed: {error:?}"),
        };
        if grants.len() != formats.len() {
            bail!("granting yielded {} outcomes for {} formats", grants.len(), formats.len());
        }
        for (format, grant) in formats.iter().zip(&grants) {
            if grant.id() != format.id {
                bail!("granting did not preserve declaration order at {}", grant.id());
            }
            let guarded = format.source.signature_challenge().is_some();
            let throttled = self.throttle_challenge(format.source.url()).is_some();
            match grant {
                YoutubeGrant::Granted { url, .. } => {
                    if guarded || throttled {
                        bail!("a guarded source was granted outright at {url}");
                    }
                    if url != format.source.url() {
                        bail!("an unguarded source was not granted its stated URL unchanged");
                    }
                }
                YoutubeGrant::Throttled { challenge, .. } => {
                    if !throttled {
                        bail!("a source posing no throttling parameter was throttled: {challenge:?}");
                    }
                    if guarded {
                        bail!("a signature guarding the representation was answered by throttling");
                    }
                }
                YoutubeGrant::Withheld { challenge, .. } => {
                    if !guarded {
                        bail!("an unguarded source was withheld behind {challenge:?}");
                    }
                }
            }
            if grant.retrievable().is_none() && !guarded {
                bail!("an unresolved throttling parameter denied retrieval of {}", grant.id());
            }
        }
        Ok(())
    }
}

/// Checks the granting laws that only a seeded, tracing interpreter can exercise.
#[ext(name = YoutubeSolutionLaws)]
pub impl<This, ChallengeError> This
where
    This: YoutubeChallengeAlg<Error = ChallengeError> + YoutubeSolutionAlg + YoutubeUrlAlg + YoutubeChallengeLawFixture,
{
    /// Checks that a resolved challenge answers exactly its own guard, once per granting.
    ///
    /// The laws checked are:
    ///
    /// 1. a solved signature is attached under the parameter the format names;
    /// 2. a solved throttling parameter replaces the one the URL poses;
    /// 3. one challenge shared by several formats is posed once for every format posing it;
    /// 4. granting resolves in exactly one bulk application;
    /// 5. a solution is stated relative to the program posing the challenge.
    ///
    /// # Errors
    ///
    /// Returns the first violated law, or the interpreter's resolution failure.
    fn youtube_solution_laws(&mut self) -> Result<()>
    where
        ChallengeError: Debug,
    {
        let signed_url = "https://media.example/signed.mp4";
        let throttled_url = "https://media.example/throttled.mp4?n=posed";
        self.solve_law_challenge(YoutubeChallenge::Signature("guarded".to_owned()), "solved");
        self.solve_law_challenge(YoutubeChallenge::Throttle("posed".to_owned()), "answered");

        let formats = [
            law_format(
                "137",
                YoutubeFormatSource::Signed {
                    url: signed_url.to_owned(),
                    signature: "guarded".to_owned(),
                    parameter: DEFAULT_SIGNATURE_PARAMETER.to_owned(),
                },
            ),
            law_format("18", YoutubeFormatSource::Direct(throttled_url.to_owned())),
            law_format("22", YoutubeFormatSource::Direct(throttled_url.to_owned())),
        ];
        let before = self.law_challenge_applications().len();
        let grants = match self.grant_formats(law_program(), &formats) {
            Ok(grants) => grants,
            Err(error) => bail!("challenge resolution failed: {error:?}"),
        };

        match &grants[0] {
            YoutubeGrant::Granted { url, .. }
                if *url == self.with_signature(signed_url, DEFAULT_SIGNATURE_PARAMETER, "solved") => {}
            other => bail!("a solved signature was not attached under its parameter: {other:?}"),
        }
        for grant in &grants[1..] {
            match grant {
                YoutubeGrant::Granted { url, .. } if self.throttle_challenge(url).as_deref() == Some("answered") => {}
                other => {
                    bail!("a solved throttling parameter did not replace the posed one: {other:?}")
                }
            }
        }

        let applications = self.law_challenge_applications();
        if applications.len() != before + 1 {
            bail!("granting resolved in {} applications rather than one", applications.len() - before);
        }
        let posed = &applications[before];
        if posed.iter().filter(|challenge| **challenge == YoutubeChallenge::Throttle("posed".to_owned())).count() != 1 {
            bail!("a challenge two formats share was not posed exactly once: {posed:?}");
        }
        Ok(())
    }
}

/// Checks that the request vocabulary distinguishes the retrievals it names.
#[ext(name = YoutubeRequestLaws)]
pub impl<This, Request> This
where
    This: YoutubeSorts<Request = Request> + YoutubeRequestAlg,
    Request: PartialEq + Debug,
{
    /// Checks that each request is a function of its arguments and distinct from the others.
    ///
    /// The laws checked are:
    ///
    /// 1. a request is a function of the values naming it;
    /// 2. retrieving a watch page, a player catalog, and media are three distinct requests;
    /// 3. the player request depends on the identity, the key, and what the client claims.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn youtube_request_laws(&self) -> Result<()> {
        let url = "https://www.youtube.com/watch?v=abc123";
        if self.watch_request(url) != self.watch_request(url) {
            bail!("a watch request is not a function of its URL");
        }
        if self.watch_request(url) == self.media_request(url) {
            bail!("retrieving a watch page denotes the same request as retrieving media");
        }
        let claim =
            PlayerClaim { client: "CLIENT".to_owned(), visitor: Some("session".to_owned()), timestamp: Some(19_000) };
        let player = self.player_request("abc123", "key", &claim);
        if player != self.player_request("abc123", "key", &claim) {
            bail!("a player request is not a function of the values naming it");
        }
        if player == self.player_request("abc123", "other", &claim) {
            bail!("a player request does not depend on its API key");
        }
        if player == self.player_request("other", "key", &claim) {
            bail!("a player request does not depend on its video identity");
        }
        if player == self.player_request("abc123", "key", &PlayerClaim::default()) {
            bail!("a player request does not depend on what the client claims about itself");
        }
        let other = PlayerClaim { client: "OTHER".to_owned(), ..claim.clone() };
        if player == self.player_request("abc123", "key", &other) {
            bail!("a player request does not depend on which client makes the claim");
        }
        Ok(())
    }
}

/// Checks that decoding recovers the observations the transport bytes describe.
#[ext(name = YoutubeResponseLaws)]
pub impl<This> This
where
    This: YoutubeResponseAlg<Error: Debug> + YoutubeLawFixture,
{
    /// Checks that decoding is a function of the bytes and observes what they describe.
    ///
    /// The laws checked are:
    ///
    /// 1. decoding is a function of the bytes it reads;
    /// 2. a watch page describing a playable video observes a player response and its API key;
    /// 3. a player response describing one retrievable format observes it as locatable.
    ///
    /// # Errors
    ///
    /// Returns the first violated law, or the interpreter's decoding failure.
    fn youtube_response_laws(&self) -> Result<()> {
        let bytes = self.law_watch_bytes();
        let page = match self.decode_youtube_watch(&bytes) {
            Ok(page) => page,
            Err(error) => bail!("decoding the watch page failed: {error:?}"),
        };
        if !page.has_player_response {
            bail!("a watch page describing a playable video observes no player response");
        }
        if page.api_key.is_none() {
            bail!("a watch page describing a playable video observes no API key");
        }
        match self.decode_youtube_watch(&bytes) {
            Ok(again) if again == page => {}
            _ => bail!("decoding a watch page is not a function of its bytes"),
        }

        let bytes = self.law_player_bytes();
        let player = match self.decode_youtube_player(&bytes) {
            Ok(player) => player,
            Err(error) => bail!("decoding the player response failed: {error:?}"),
        };
        if player.status.as_deref() != Some("OK") {
            bail!("a player response describing a playable video is not observed as playable");
        }
        if player.formats.is_empty() {
            bail!("a player response describing one format observes none");
        }
        for format in &player.formats {
            if format.source.url().is_empty() {
                bail!("a described format states no location for its representation");
            }
        }
        match self.decode_youtube_player(&bytes) {
            Ok(again) if again == player => {}
            _ => bail!("decoding a player response is not a function of its bytes"),
        }
        Ok(())
    }
}

/// Checks that Youtube extraction denotes exactly the video its URL identifies.
#[ext(name = YoutubeExtractionLaws)]
pub impl<This, Request> This
where
    This: YoutubeSorts<Request = Request>
        + YoutubeRequestAlg
        + YoutubeClientAlg
        + YoutubeProgramAlg
        + YoutubeRequestBytesAlg<Error: Display>
        + YoutubeResponseAlg<Error: Display>
        + YoutubeChallengeAlg<Error: Display>
        + YoutubeSolutionAlg
        + YoutubeUrlAlg
        + MetadataAlg
        + FormatAlg
        + ExtractionAlg
        + ExtractionViewAlg
        + MediaViewAlg
        + YoutubeLawFixture,
    Request: PartialEq + Debug,
{
    /// Checks that identity, request order, and extracted meaning follow the watch URL.
    ///
    /// The laws checked are:
    ///
    /// 1. the supported spellings of one video denote the same identity, and no other host does;
    /// 2. an unsupported URL fails before any request is executed;
    /// 3. extraction executes the watch request first, then the requests that page enables;
    /// 4. extracting one video denotes one media item carrying that identity and its formats.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn youtube_extraction_laws(&self) -> Result<()> {
        let watch_url = self.law_watch_url();
        let id = self.law_video_id();
        if youtube_id(&watch_url).as_deref() != Some(id.as_str()) {
            bail!("the watch URL does not denote the identity the fixture states");
        }
        if youtube_id(&format!("https://youtu.be/{id}")).as_deref() != Some(id.as_str()) {
            bail!("the short spelling of one video denotes another identity");
        }
        if let Some(other) = youtube_id("https://example.com/watch?v=abc123") {
            bail!("an unsupported host denoted the video identity {other}");
        }

        let before = self.law_executed_requests();
        match self.extract_youtube("https://example.com/watch?v=abc123") {
            Err(YoutubeError::Url(_)) => {}
            Ok(_) => bail!("an unsupported URL extracted"),
            Err(error) => bail!("an unsupported URL failed as {error}"),
        }
        if self.law_executed_requests() != before {
            bail!("an unsupported URL executed a request");
        }

        let extraction = match self.extract_youtube(&watch_url) {
            Ok(extraction) => extraction,
            Err(error) => bail!("extracting the fixture URL failed: {error}"),
        };
        let executed = self.law_executed_requests();
        let performed = &executed[before.len()..];
        if performed.len() < 2 {
            bail!("extraction executed {} requests rather than several", performed.len());
        }
        if performed[0] != self.watch_request(&watch_url) {
            bail!("extraction did not begin with the watch request: {:?}", performed[0]);
        }
        if performed[1..].contains(&performed[0]) {
            bail!("extraction executed the watch request more than once");
        }

        let Some(media) = self.as_media(extraction) else {
            bail!("extracting one video did not denote one media item");
        };
        if self.media_id(&media) != id {
            bail!("the extracted media carries {} rather than {id}", self.media_id(&media));
        }
        if self.media_formats(&media).is_empty() {
            bail!("the extracted media carries no format");
        }
        Ok(())
    }
}

/// Checks that Youtube specialization hands its media request to generic download unchanged.
#[ext(name = YoutubeApplicationLaws)]
pub impl<This, Request, Event, Progress> This
where
    This: YoutubeSorts<Request = Request>
        + YoutubeRequestAlg
        + YoutubeClientAlg
        + YoutubeProgramAlg
        + YoutubeRequestBytesAlg<Error: Display>
        + YoutubeResponseAlg<Error: Display>
        + YoutubeChallengeAlg<Error: Display>
        + YoutubeSolutionAlg
        + YoutubeUrlAlg
        + MetadataAlg
        + FormatAlg
        + FormatViewAlg
        + FormatPredicateAlg
        + FormatSelectionAlg
        + ExtractionAlg
        + ExtractionViewAlg
        + MediaViewAlg
        + FormatSelectionApplyAlg
        + FormatSourceAlg<Request>
        + FetchStreamAlg<Request, Error: Display>
        + AtomicPublishAlg<Error: Display>
        + DownloadObservationAlg<Event = Event, Progress = Progress>
        + DownloadProgressAlg<Progress = Progress>
        + DownloadReportAlg<Event = Event>
        + YoutubeLawFixture
        + MediaProgramLawFixture,
    Request: PartialEq + Debug,
{
    /// Checks that one Youtube URL publishes exactly the bytes its granted representation denotes.
    ///
    /// The laws checked are:
    ///
    /// 1. the published path is named from the extracted identity and the selected format;
    /// 2. retrieval opens exactly one request, and it is neither the watch nor the player request;
    /// 3. the granted representation reaches generic download unchanged.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn youtube_application_laws(&mut self) -> Result<()> {
        let watch_url = self.law_watch_url();
        let selection = self.best_format(self.any_format());
        let published = match self.download_youtube(&watch_url, &selection, &OutputTarget::MediaId) {
            Ok(published) => published,
            Err(error) => bail!("the Youtube download failed: {error}"),
        };
        let expected = PathBuf::from(format!("{}.{}", self.law_video_id(), self.law_extension()));
        if published != expected {
            bail!("naming derived {} rather than {}", published.display(), expected.display());
        }
        let retrieved = self.law_retrieved_requests();
        if retrieved.len() != 1 {
            bail!("retrieval opened {} requests rather than one", retrieved.len());
        }
        if retrieved[0] == self.watch_request(&watch_url) {
            bail!("retrieval reused the watch request rather than the media request");
        }
        if self.law_published_bytes(&published).as_deref() != Some(&self.expected_law_bytes()[..]) {
            bail!("the granted representation did not reach generic download unchanged");
        }
        Ok(())
    }
}

/// Names the challenge a grant withheld, when it withheld one.
#[must_use]
pub fn withheld_challenge(grant: &YoutubeGrant) -> Option<&YoutubeChallenge> {
    match grant {
        YoutubeGrant::Withheld { challenge, .. } => Some(challenge),
        YoutubeGrant::Granted { .. } | YoutubeGrant::Throttled { .. } => None,
    }
}

/// States the player program a granting scenario poses its challenges under.
fn law_program() -> &'static str {
    "law://player-program"
}

/// Describes one format a granting scenario authors.
fn law_format(id: &str, source: YoutubeFormatSource) -> YoutubeFormat {
    YoutubeFormat {
        id: id.to_owned(),
        source,
        extension: Some("mp4".to_owned()),
        has_audio: true,
        has_video: true,
        quality: None,
        width: None,
        height: None,
        bitrate: None,
        content_length: None,
        codecs: None,
    }
}
