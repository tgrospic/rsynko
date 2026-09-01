use crate::*;
use alux_ext::ext;
use rsynko_manager::MediaStreams;

/// Specifies what one selectable format states where a chooser can read it.
///
/// A chooser reads a format to tell it from its alternatives. These are the observations that
/// distinguish them; everything else an extractor stated stays in the extraction.
pub trait FormatDescriptionAlg {
    /// Observes the identity the extractor gave the format.
    fn format_identity(&self) -> &str;
    /// Observes the container or file-name extension.
    fn format_extension(&self) -> Option<&str>;
    /// Observes the stream roles the format carries, when it carries any.
    fn format_streams(&self) -> Option<MediaStreams>;
    /// Observes the quality the extractor named.
    fn format_quality(&self) -> Option<&str>;
    /// Observes the picture height in pixels.
    fn format_height(&self) -> Option<u64>;
    /// Observes the picture width in pixels.
    fn format_width(&self) -> Option<u64>;
    /// Observes the stated bits per second.
    fn format_bitrate(&self) -> Option<u64>;
    /// Observes the stated byte count of the whole format.
    fn format_size(&self) -> Option<u64>;
    /// Observes the codecs the format is encoded with.
    fn format_codecs(&self) -> Option<&str>;
}

/// Denotes what discovery has stated about one request's selectable formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscoveryState<'a> {
    /// Denotes a source whose formats no one has asked for.
    Unrequested,
    /// Denotes a request waiting for an interpreter to inspect the source.
    Waiting,
    /// Denotes source inspection currently in progress.
    Inspecting,
    /// Denotes formats described in extractor preference order.
    Described,
    /// Denotes inspection that failed, and why.
    Failed(&'a str),
}

/// Specifies what one queue entry states about the formats a chooser may offer.
pub trait FormatChoiceViewAlg {
    /// Represents one described selectable format.
    type Format;

    /// Observes what discovery has stated about this request's formats.
    fn discovery(&self) -> DiscoveryState<'_>;
    /// Observes the described formats in extractor preference order.
    fn described_formats(&self) -> impl Iterator<Item = &Self::Format>;
}

/// Derives what a chooser reads about one format.
#[ext(name = FormatLabelExt)]
pub impl<This> This
where
    This: FormatDescriptionAlg,
{
    /// States the format in aligned columns, so alternatives are compared by reading down.
    fn format_label(&self) -> String {
        let described = self.format_observations();
        let columns = format!(
            "{:<4} {:<5} {:<13}",
            self.format_identity(),
            self.format_extension().unwrap_or(UNSTATED),
            self.format_streams().map_or("no media streams", MediaStreams::streams_label)
        );
        if described.is_empty() { columns } else { format!("{columns} {described}") }
    }

    /// States what distinguishes the format from an alternative carrying the same streams.
    fn format_observations(&self) -> String {
        let quality = self.format_stated_quality();
        [
            quality.as_deref().map(str::to_owned),
            self.format_bitrate().map(|bitrate| format!("{}/s", bytes_label(bitrate / 8))),
            self.format_size().map(bytes_label),
            self.format_codecs().map(str::to_owned),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("  ")
    }

    /// States the quality the extractor named, or the picture size standing in for it.
    fn format_stated_quality(&self) -> Option<String> {
        if let Some(quality) = self.format_quality() {
            return Some(quality.to_owned());
        }
        match (self.format_width(), self.format_height()) {
            (Some(width), Some(height)) => Some(format!("{width}x{height}")),
            (None, Some(height)) => Some(format!("{height}p")),
            (Some(_) | None, None) => None,
        }
    }
}

/// Derives which stream roles a request may actually be asked for.
#[ext(name = FormatRolesExt)]
pub impl<This> This
where
    This: FormatChoiceViewAlg,
    This::Format: FormatDescriptionAlg,
{
    /// States the roles some described format carries, and every role while none are described.
    ///
    /// A chooser offers what can be chosen. Asking for sound alone from a source that only ever
    /// states one file carrying both would select nothing at all, so a role no format carries is
    /// not offered at all. Before anything is described there is nothing to rule out, and every
    /// role stands.
    fn offered_streams(&self) -> Vec<MediaStreams> {
        let described = self.described_formats().collect::<Vec<_>>();
        // Nothing described is not the same as nothing carrying a role: before a source has said
        // what it holds there is nothing to rule out, and a source that has said, and named only
        // pictures, offers no role at all.
        if described.is_empty() {
            return MediaStreams::OFFERED.to_vec();
        }
        let carried = described.into_iter().filter_map(FormatDescriptionAlg::format_streams).collect::<Vec<_>>();
        MediaStreams::OFFERED.into_iter().filter(|role| carried.contains(role)).collect()
    }
}
