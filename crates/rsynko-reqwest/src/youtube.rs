use crate::{ANDROID_CLIENT, RuntimeEnvironment, SignatureProgram, VISIONOS_CLIENT};
use rsynko_media::*;
use rsynko_memory::{Extraction, Format, Media, MediaSyntax, YoutubeRequest, YoutubeRequestSyntax};
use rsynko_yt::*;
use serde_json::Value;
use std::convert::Infallible;
use url::{Url, form_urlencoded};

/// Denotes the challenges one player program resolved, and the values it resolved them to.
///
/// The throttling parameter is governed by an arbitrary player function rather than the fixed
/// transformations a signature poses, so this runtime answers signatures and leaves a throttling
/// parameter posed. A representation is then served slowly rather than withheld.
#[derive(Clone, Debug, Default)]
pub struct YoutubeSolutions(Vec<(YoutubeChallenge, String)>);

impl YoutubeSorts for RuntimeEnvironment {
    type Request = YoutubeRequest;
    type Solutions = YoutubeSolutions;
}

impl YoutubeRequestAlg for RuntimeEnvironment {
    fn watch_request(&self, url: impl Into<String>) -> Self::Request {
        YoutubeRequestSyntax.watch_request(url)
    }

    fn player_request(
        &self,
        id: impl Into<String>,
        api_key: impl Into<String>,
        claim: &PlayerClaim,
    ) -> Self::Request {
        YoutubeRequestSyntax.player_request(id, api_key, claim)
    }

    fn player_program_request(&self, url: impl Into<String>) -> Self::Request {
        YoutubeRequestSyntax.player_program_request(url)
    }

    fn media_request(&self, url: impl Into<String>) -> Self::Request {
        YoutubeRequestSyntax.media_request(url)
    }
}

impl YoutubeClientAlg for RuntimeEnvironment {
    fn player_clients(&self) -> impl Iterator<Item = &str> {
        // The handheld client states one muxed representation, served without throttling; the
        // headset client states every adaptive representation. Neither alone is the catalog.
        [ANDROID_CLIENT, VISIONOS_CLIENT].into_iter()
    }
}

impl YoutubeProgramAlg for RuntimeEnvironment {
    fn program_timestamp(&self, program: &str) -> Option<i64> {
        // The player states the timestamp its signatures were issued under.
        let marker = "signatureTimestamp:";
        let tail = program.get(program.find(marker)? + marker.len()..)?;
        let digits: String = tail.chars().take_while(char::is_ascii_digit).collect();
        digits.parse().ok()
    }
}

impl FormatSourceAlg<YoutubeRequest> for RuntimeEnvironment {
    fn format_source(&self, format: &Format) -> Option<YoutubeRequest> {
        format.source().map(|url| self.media_request(url))
    }
}

impl MetadataAlg for RuntimeEnvironment {
    fn empty_metadata(&self) -> Self::Metadata {
        MediaSyntax.empty_metadata()
    }
    fn null_metadata(&self) -> Self::Value {
        MediaSyntax.null_metadata()
    }
    fn boolean_metadata(&self, value: bool) -> Self::Value {
        MediaSyntax.boolean_metadata(value)
    }
    fn integer_metadata(&self, value: i64) -> Self::Value {
        MediaSyntax.integer_metadata(value)
    }
    fn float_metadata(&self, value: f64) -> Self::Value {
        MediaSyntax.float_metadata(value)
    }
    fn string_metadata(&self, value: impl Into<String>) -> Self::Value {
        MediaSyntax.string_metadata(value)
    }
    fn list_metadata(&self, values: impl IntoIterator<Item = Self::Value>) -> Self::Value {
        MediaSyntax.list_metadata(values)
    }
    fn record_metadata(&self, record: Self::Metadata) -> Self::Value {
        MediaSyntax.record_metadata(record)
    }
    fn metadata(&self, fields: impl IntoIterator<Item = (String, Self::Value)>) -> Self::Metadata {
        MediaSyntax.metadata(fields)
    }
}

impl FormatAlg for RuntimeEnvironment {
    fn format(&self, id: impl Into<String>, metadata: Self::Metadata) -> Self::Format {
        MediaSyntax.format(id, metadata)
    }
}

impl ExtractionAlg for RuntimeEnvironment {
    fn media(
        &self,
        id: impl Into<String>,
        metadata: Self::Metadata,
        formats: impl IntoIterator<Item = Self::Format>,
    ) -> Self::Extraction {
        MediaSyntax.media(id, metadata, formats)
    }

    fn url_reference(
        &self,
        url: impl Into<String>,
        extractor: Option<Self::Extractor>,
        transparent: bool,
    ) -> Self::Extraction {
        MediaSyntax.url_reference(url, extractor, transparent)
    }

    fn extraction_collection(
        &self,
        id: Option<String>,
        kind: CollectionKind,
        metadata: Self::Metadata,
        entries: impl IntoIterator<Item = Self::Extraction>,
    ) -> Self::Extraction {
        MediaSyntax.extraction_collection(id, kind, metadata, entries)
    }
}

impl ExtractionViewAlg for RuntimeEnvironment {
    fn as_media(&self, extraction: Extraction) -> Option<Media> {
        MediaSyntax.as_media(extraction)
    }
}

impl MediaViewAlg for RuntimeEnvironment {
    fn media_id<'a>(&self, media: &'a Media) -> &'a str {
        MediaSyntax.media_id(media)
    }

    fn media_title<'a>(&self, media: &'a Media) -> Option<&'a str> {
        MediaSyntax.media_title(media)
    }

    fn media_formats<'a>(&self, media: &'a Media) -> &'a [Format] {
        MediaSyntax.media_formats(media)
    }
}

impl YoutubeResponseAlg for RuntimeEnvironment {
    type Error = serde_json::Error;

    fn decode_youtube_watch(&self, bytes: &[u8]) -> Result<YoutubeWatchPage, Self::Error> {
        let page = String::from_utf8_lossy(bytes);
        let player = player_response(&page);
        Ok(YoutubeWatchPage {
            has_player_response: player.is_some(),
            api_key: quoted_config(&page, "INNERTUBE_API_KEY").map(str::to_owned),
            player_url: player_program_url(&page),
            visitor_data: quoted_config(&page, "VISITOR_DATA").map(str::to_owned),
            title: player
                .as_ref()
                .and_then(|value| value.pointer("/videoDetails/title"))
                .and_then(Value::as_str)
                .map(str::to_owned),
        })
    }

    fn decode_youtube_player(&self, bytes: &[u8]) -> Result<YoutubePlayer, Self::Error> {
        let player: Value = serde_json::from_slice(bytes)?;
        Ok(YoutubePlayer {
            status: player
                .pointer("/playabilityStatus/status")
                .and_then(Value::as_str)
                .map(str::to_owned),
            reason: player
                .pointer("/playabilityStatus/reason")
                .and_then(Value::as_str)
                .map(str::to_owned),
            title: player
                .pointer("/videoDetails/title")
                .and_then(Value::as_str)
                .map(str::to_owned),
            formats: direct_formats(&player),
            unreadable: unreadable_formats(&player),
        })
    }
}

fn player_response(page: &str) -> Option<Value> {
    ["ytInitialPlayerResponse =", "ytInitialPlayerResponse="]
        .into_iter()
        .find_map(|marker| {
            let tail = page.get(page.find(marker)? + marker.len()..)?;
            serde_json::from_str(balanced_object(tail)?).ok()
        })
}

fn quoted_config<'a>(page: &'a str, key: &str) -> Option<&'a str> {
    let marker = format!("\"{key}\":\"");
    let tail = page.get(page.find(&marker)? + marker.len()..)?;
    tail.get(..tail.find('"')?)
}

fn balanced_object(input: &str) -> Option<&str> {
    let start = input.find('{')?;
    let mut depth = 0_u32;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, character) in input[start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }
        match character {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return input.get(start..=start + offset);
                }
            }
            _ => {}
        }
    }
    None
}

impl YoutubeUrlAlg for RuntimeEnvironment {
    fn throttle_challenge(&self, url: &str) -> Option<String> {
        Url::parse(url)
            .ok()?
            .query_pairs()
            .find(|(key, _)| key == "n")
            .map(|(_, value)| value.into_owned())
    }

    fn with_throttle(&self, url: &str, solution: &str) -> String {
        replaced_query(url, "n", solution)
    }

    fn with_signature(&self, url: &str, parameter: &str, signature: &str) -> String {
        replaced_query(url, parameter, signature)
    }
}

impl YoutubeChallengeAlg for RuntimeEnvironment {
    type Error = Infallible;

    fn solve_challenges(
        &self,
        program: &str,
        challenges: impl IntoIterator<Item = YoutubeChallenge>,
    ) -> Result<Self::Solutions, Self::Error> {
        let Ok(decipher) = SignatureProgram::recover(program) else {
            return Ok(YoutubeSolutions::default());
        };
        Ok(YoutubeSolutions(
            challenges
                .into_iter()
                .filter_map(|challenge| match &challenge {
                    YoutubeChallenge::Signature(value) => {
                        let solved = decipher.decipher(value);
                        Some((challenge, solved))
                    }
                    YoutubeChallenge::Throttle(_) => None,
                })
                .collect(),
        ))
    }
}

impl YoutubeSolutionAlg for RuntimeEnvironment {
    fn solution_of(
        &self,
        solutions: &Self::Solutions,
        challenge: &YoutubeChallenge,
    ) -> Option<String> {
        solutions
            .0
            .iter()
            .find(|(posed, _)| posed == challenge)
            .map(|(_, solution)| solution.clone())
    }
}

fn replaced_query(url: &str, key: &str, value: &str) -> String {
    let Ok(mut parsed) = Url::parse(url) else {
        return url.to_owned();
    };
    let kept: Vec<(String, String)> = parsed
        .query_pairs()
        .filter(|(name, _)| name != key)
        .map(|(name, value)| (name.into_owned(), value.into_owned()))
        .collect();
    parsed
        .query_pairs_mut()
        .clear()
        .extend_pairs(kept)
        .append_pair(key, value);
    parsed.into()
}

fn direct_formats(player: &Value) -> Vec<YoutubeFormat> {
    described_formats(player)
        .filter_map(direct_format)
        .collect()
}

/// Counts the described formats this interpreter could not read.
fn unreadable_formats(player: &Value) -> usize {
    described_formats(player)
        .filter(|value| direct_format(value).is_none())
        .count()
}

/// Observes every format the response described, muxed and adaptive alike.
fn described_formats(player: &Value) -> impl Iterator<Item = &Value> {
    ["/streamingData/formats", "/streamingData/adaptiveFormats"]
        .into_iter()
        .filter_map(|path| player.pointer(path).and_then(Value::as_array))
        .flatten()
}

/// Recovers the location a format states, whether stated directly or behind a signature.
fn format_source(value: &Value) -> Option<YoutubeFormatSource> {
    if let Some(url) = value.get("url").and_then(Value::as_str) {
        return Some(YoutubeFormatSource::Direct(url.to_owned()));
    }
    let cipher = value.get("signatureCipher").and_then(Value::as_str)?;
    let (mut url, mut signature, mut parameter) = (None, None, None);
    for (key, field) in form_urlencoded::parse(cipher.as_bytes()) {
        match key.as_ref() {
            "url" => url = Some(field.into_owned()),
            "s" => signature = Some(field.into_owned()),
            "sp" => parameter = Some(field.into_owned()),
            _ => {}
        }
    }
    Some(YoutubeFormatSource::Signed {
        url: url?,
        signature: signature?,
        parameter: parameter.unwrap_or_else(|| DEFAULT_SIGNATURE_PARAMETER.to_owned()),
    })
}

fn direct_format(value: &Value) -> Option<YoutubeFormat> {
    let source = format_source(value)?;
    let mime = value.get("mimeType")?.as_str()?;
    let has_video = mime.starts_with("video/");
    let has_audio = mime.starts_with("audio/")
        || value.get("audioQuality").is_some()
        || value.get("audioSampleRate").is_some();
    if !has_audio && !has_video {
        return None;
    }
    Some(YoutubeFormat {
        id: value.get("itag")?.as_u64()?.to_string(),
        source,
        extension: mime.split('/').nth(1)?.split(';').next().map(str::to_owned),
        has_audio,
        has_video,
        quality: value
            .get("qualityLabel")
            .or_else(|| value.get("quality"))
            .and_then(Value::as_str)
            .map(str::to_owned),
        width: value.get("width").and_then(Value::as_i64),
        height: value.get("height").and_then(Value::as_i64),
        bitrate: value.get("bitrate").and_then(Value::as_i64),
        content_length: value
            .get("contentLength")
            .and_then(Value::as_str)
            .and_then(|length| length.parse().ok()),
        codecs: mime
            .split("codecs=\"")
            .nth(1)
            .and_then(|tail| tail.split('"').next())
            .map(str::to_owned),
    })
}

/// Locates the player program the watch page states, as an absolute URL.
fn player_program_url(page: &str) -> Option<String> {
    let stated = ["\"jsUrl\":\"", "\"PLAYER_JS_URL\":\""]
        .into_iter()
        .find_map(|marker| {
            let tail = page.get(page.find(marker)? + marker.len()..)?;
            tail.get(..tail.find('"')?)
        })?
        .replace("\\/", "/");
    Some(if stated.starts_with('/') {
        format!("https://www.youtube.com{stated}")
    } else {
        stated
    })
}
