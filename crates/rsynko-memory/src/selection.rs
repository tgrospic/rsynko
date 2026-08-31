use crate::{
    Format, FormatPredicate, FormatSelection, InfoRecord, InfoValue, interpret_selection,
    predicate_accepts,
};
use rsynko_media::{FormatPredicateMatchAlg, FormatSelectionApplyAlg, FormatViewAlg, MediaSorts};

/// Interprets format predicates and selection programs over the reified media sorts.
#[derive(Clone, Copy, Debug, Default)]
pub struct ReferenceFormatSelector;

impl MediaSorts for ReferenceFormatSelector {
    type Value = InfoValue;
    type Metadata = InfoRecord;
    type Format = Format;
    type Artifact = ();
    type Media = ();
    type Extraction = ();
    type Extractor = ();
    type Predicate = FormatPredicate;
    type Selection = FormatSelection;
    type Output = ();
}

impl FormatViewAlg for ReferenceFormatSelector {
    fn format_text<'a>(&self, format: &'a Format, key: &str) -> Option<&'a str> {
        format.text(key)
    }
}

impl FormatPredicateMatchAlg for ReferenceFormatSelector {
    fn format_matches(&self, predicate: &FormatPredicate, format: &Format) -> bool {
        predicate_accepts(predicate, format)
    }
}

impl FormatSelectionApplyAlg for ReferenceFormatSelector {
    fn select_formats<'a>(
        &self,
        formats: &'a [Format],
        selection: &FormatSelection,
    ) -> Option<Vec<&'a Format>> {
        interpret_selection(self, formats, selection)
    }
}
