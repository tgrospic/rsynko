use alux_ext::ext;
use alux_sdk::trait_algebra;
use rsynko_media::{FormatPredicateAlg, FormatPredicateExt, FormatSelectionAlg};

/// Specifies what one request states about the media it selects.
pub trait RequestOptionsAlg {
    /// Observes what performs the request.
    fn performer(&self) -> Performer;

    /// Observes the stream roles the request prefers, and that it chooses no media when it
    /// transfers something a media role says nothing about.
    fn media_streams(&self) -> Option<MediaStreams>;

    /// Observes the one choice the request fixes, when it fixes one.
    ///
    /// What the choices are depends on what the request transfers: one representation of a media
    /// item, or one way of transferring a folder. Either way, exactly one may be fixed.
    fn chosen_choice(&self) -> Option<&str>;

    /// Observes every choice the request offers, in the order it offers them.
    ///
    /// A media item offers what extraction discovered, and its stream roles state which of them
    /// is *preferred* when none is fixed; they never hide one the source states. A folder offers every way it may be transferred.
    fn selectable_choices(&self) -> impl Iterator<Item = &str>;

    /// Observes what one offered choice states about itself, when it states anything.
    ///
    /// A choice a reader would not recognize by name says what it does; one described by what it
    /// carries is read through that description instead.
    fn choice_summary(&self, choice: &str) -> Option<&str>;
}

/// Defines the first-order observation stream of format discovery.
///
/// An observation is a method, so the reified stream is generated from this vocabulary rather than
/// written out. An interpreter states what each observation *means* and never restates the shape.
#[trait_algebra(derive(Clone, Debug, PartialEq, Eq))]
pub trait FormatDiscovery {
    /// Represents one described selectable format.
    type Format;

    /// Denotes that an interpreter started source inspection.
    fn started(&self);

    /// Supplies the described formats in extractor preference order.
    fn available(&self, formats: Vec<Self::Format>);

    /// Denotes source inspection failure.
    fn failed(&self, message: String);
}

/// Derives the media one request selects from the choices it states.
#[ext(name = RequestSelectionExt)]
pub impl<This> This
where
    This: FormatPredicateAlg + FormatSelectionAlg,
{
    /// Defines the selection one request's own choices state.
    fn request_selection(&self, options: &impl RequestOptionsAlg) -> This::Selection {
        let predicate = match options.chosen_choice() {
            // A fixed identity names exactly one format, so the stream roles add nothing to it.
            Some(choice) => self.format_id(choice),
            // A request choosing no media role is not one this program retrieves, so it asks for
            // the whole thing and lets selection refuse it.
            None => self.stream_role_format(options.media_streams().unwrap_or(MediaStreams::AudioVideo)),
        };
        self.best_format(predicate)
    }
}

/// Derives the format predicate one set of stream roles requires.
#[ext(name = MediaStreamsExt)]
pub impl<This> This
where
    This: FormatPredicateAlg,
{
    /// Defines the predicate accepting exactly the formats those roles require.
    fn stream_role_format(&self, streams: MediaStreams) -> This::Predicate {
        match streams {
            MediaStreams::AudioVideo => self.progressive_format(),
            MediaStreams::Audio => self.audio_only_format(),
            MediaStreams::Video => self.video_only_format(),
        }
    }
}

/// Denotes what performs one request.
///
/// A request is either run by naming a program and reading what it writes, or retrieved by this
/// program itself from wherever the source that claimed it keeps what it names. How many files
/// come to rest, and whether they land in a folder, follows from what the request chose — not
/// from which of these performs it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Performer {
    /// Denotes a request performed by naming a program and reading what it writes.
    Program,
    /// Denotes a request this program retrieves for itself.
    Retrieval,
}

/// Selects the stream roles a request prefers when it fixes no format identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaStreams {
    /// Selects a progressive format containing both audio and video.
    AudioVideo,
    /// Selects a format containing audio and no video.
    Audio,
    /// Selects a format containing video and no audio.
    Video,
}

impl MediaStreams {
    /// States the roles a request may prefer, in the order they are offered.
    pub const OFFERED: [Self; 3] = [Self::AudioVideo, Self::Video, Self::Audio];
}
