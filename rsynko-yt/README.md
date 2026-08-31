# rsynko-yt

This crate specifies how Youtube is a special case of media download. A Youtube watch URL denotes a video identity; retrieving its watch page reveals a player key; a fully described player request denotes the media catalog; each catalog entry states where its representation lives and what guards it; a granted entry denotes a fully described media request; and that request is consumed by the generic atomic download program.

The specification owns Youtube URL recognition, request descriptions, player-response interpretation, challenge meaning, format ranking, and the composition with ordinary selection, naming, progress, atomic publication, and terminal reporting. It chooses no HTTP client, no JSON decoder, no URL library, and no player-program evaluator. An interpreter executes a `YoutubeRequest`, decodes response bytes, performs the query mechanics, and resolves challenges; it never infers Youtube behavior from a host name or media URL.

URL recognition is a pure observation and belongs to the specification:

```rust
use rsynko_yt::youtube_id;

assert_eq!(
    youtube_id("https://www.youtube.com/watch?v=abc123"),
    Some("abc123".to_owned()),
);
assert_eq!(youtube_id("https://example.com/watch?v=abc123"), None);
```

[`YoutubeSorts`] carries the request and solution sorts every Youtube class shares, so a class never re-declares one. [`YoutubeRequestAlg`] defines the watch, player, player-program, and media requests in that carrier; [`YoutubeRequestBytesAlg`] executes one without changing what it means; [`YoutubeProgramAlg`] observes what a player program states about itself; and [`YoutubeResponseAlg`] decodes the bytes into [`YoutubeWatchPage`] and [`YoutubePlayer`] observations.

Youtube grants a catalog matching what a client claims about itself, so the program is retrieved before the catalog and the player request states the session the page issued and the signature timestamp of the program the client runs. A request claiming nothing is answered with the formats such a client may retrieve, or refused outright.

Each client is granted a *different* catalog, and neither is the whole truth: one states a single muxed representation carrying both streams, another states every adaptive representation on its own. [`YoutubeClientAlg`] names the clients an interpreter can present itself as, and extraction asks under each claim and takes the union in claim order. A client refused states nothing about the others.

A reference interpreter tying those sorts to an inspectable `YoutubeRequest` lives in `rsynko-memory`.

## Challenges

A described format is not the same as a retrievable one. Youtube states a format's location in one of two ways, denoted by `YoutubeFormatSource`: directly, or behind a signature that only the player program resolves. Independently, a location may carry a throttling parameter governing the rate at which the representation is served. `YoutubeChallenge` denotes both as one kind of thing — an obfuscated value whose solution is a value.

The two guard different things. A signature guards access: unanswered, the representation cannot be retrieved at all. A throttling parameter governs the rate at which a granted representation is served: unanswered, the representation is still retrievable, only slowly.

Granting a format means answering every challenge its source poses. `YoutubeGrant` denotes the outcome for exactly one format: granted at a fully answered URL, throttled at a URL still posing its throttling parameter, or withheld together with the signature that withheld it. Withholding is an observation, never a silent omission.

```rust
use rsynko_yt::{DEFAULT_SIGNATURE_PARAMETER, YoutubeChallenge, YoutubeFormatSource};

let source = YoutubeFormatSource::Signed {
    url: "https://media.example/video.mp4".to_owned(),
    signature: "obfuscated".to_owned(),
    parameter: DEFAULT_SIGNATURE_PARAMETER.to_owned(),
};
assert_eq!(source.url(), "https://media.example/video.mp4");
assert_eq!(
    source.signature_challenge(),
    Some(YoutubeChallenge::Signature("obfuscated".to_owned())),
);
```

Three primitive capabilities carry the parts a specification cannot state. `YoutubeUrlAlg` observes and answers the query surface. `YoutubeChallengeAlg` resolves the posed challenges in one application under the player program that poses them, so that resolution cost follows the number of distinct challenges rather than the number of formats, and a solution is never mistaken for one another program would give. `YoutubeSolutionAlg` observes one individual solution. `YoutubeChallengeExt` derives challenge collection, per-format granting, and whole-catalog granting over them.

An interpreter that resolves nothing is a complete interpreter. It withholds every signed format with its unresolved signature and serves the rest at whatever rate their unanswered throttling parameters govern, which keeps the catalog honest while a player-program evaluator is absent.

## Laws

- request construction is deterministic for a video identity and player key;
- only audio and/or video formats are admitted;
- a source posing no challenge is granted its stated URL unchanged;
- granting yields exactly one outcome per described format, in declaration order;
- one distinct challenge is posed to resolution once however many formats pose it;
- an unresolved signature withholds exactly the formats it guards and no others;
- an unresolved throttling parameter governs the rate of a still retrievable URL;
- a solution is stated relative to the player program that poses the challenge;
- attaching a solved signature leaves the throttling challenge of a URL unchanged;
- formats are ordered by height and bitrate before ordinary best/worst selection;
- the player catalog depends on the video identity, the API key, and what the client claims;
- the catalog is the union of what every client is granted, in claim order;
- extraction states how many formats were described, unreadable, withheld, and throttled;
- the selected media request is passed unchanged to generic download;
- generic download publication, progress, and exactly-one-terminal-event laws remain intact.
