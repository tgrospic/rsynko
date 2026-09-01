//! Checks that construction stays parametric in the carrier, using a counting interpreter.

use rsynko_media::{CollectionKind, ExtractionAlg, MediaSorts};
use rsynko_memory::{Extraction, Format, InfoRecord, MediaSyntax};

struct CountSyntax;

impl MediaSorts for CountSyntax {
    type Value = ();
    type Metadata = ();
    type Format = ();
    type Artifact = ();
    type Media = ();
    type Extraction = usize;
    type Extractor = ();
    type Predicate = ();
    type Selection = ();
    type Output = ();
}

impl ExtractionAlg for CountSyntax {
    fn media(
        &self,
        _: impl Into<String>,
        (): Self::Metadata,
        formats: impl IntoIterator<Item = Self::Format>,
    ) -> Self::Extraction {
        formats.into_iter().count() + 1
    }
    fn url_reference(&self, _: impl Into<String>, _: Option<Self::Extractor>, _: bool) -> Self::Extraction {
        1
    }
    fn extraction_collection(
        &self,
        _: Option<String>,
        _: CollectionKind,
        (): Self::Metadata,
        entries: impl IntoIterator<Item = Self::Extraction>,
    ) -> Self::Extraction {
        entries.into_iter().sum::<usize>() + 1
    }
}

#[test]
fn recursive_extraction_construction_does_not_fix_tree_representation() {
    let count = CountSyntax.extraction_collection(
        None,
        CollectionKind::Playlist,
        (),
        [CountSyntax.media("one", (), [()]), CountSyntax.url_reference("two", None, false)],
    );
    assert_eq!(count, 4);

    let syntax = MediaSyntax.media("one", InfoRecord::default(), Vec::<Format>::default());
    assert!(matches!(syntax, Extraction::Media(_)));
}
