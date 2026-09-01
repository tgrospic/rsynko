//! Checks that construction stays parametric in the carrier, using a counting interpreter.

use rsynko_media::*;
use rsynko_memory::{FormatPredicate, FormatSelection, MediaSyntax};

struct CountSyntax;

impl MediaSorts for CountSyntax {
    type Value = ();
    type Metadata = ();
    type Format = ();
    type Artifact = ();
    type Media = ();
    type Extraction = ();
    type Extractor = ();
    type Predicate = usize;
    type Selection = usize;
    type Output = ();
}

impl FormatPredicateAlg for CountSyntax {
    fn any_format(&self) -> Self::Predicate {
        1
    }
    fn format_id(&self, _: impl Into<String>) -> Self::Predicate {
        1
    }
    fn observed_format(&self, _: impl Into<String>) -> Self::Predicate {
        1
    }
    fn text_format(&self, _: impl Into<String>, _: impl Into<String>) -> Self::Predicate {
        1
    }
    fn flag_format(&self, _: impl Into<String>, _: bool) -> Self::Predicate {
        1
    }
    fn number_format(&self, _: impl Into<String>, _: FormatComparison, _: f64) -> Self::Predicate {
        1
    }
    fn all_formats(&self, left: Self::Predicate, right: Self::Predicate) -> Self::Predicate {
        left + right + 1
    }
    fn not_format(&self, predicate: Self::Predicate) -> Self::Predicate {
        predicate + 1
    }
}

impl FormatSelectionAlg for CountSyntax {
    fn best_format(&self, predicate: Self::Predicate) -> Self::Selection {
        predicate + 1
    }
    fn worst_format(&self, predicate: Self::Predicate) -> Self::Selection {
        predicate + 1
    }
    fn merge_formats(&self, values: impl IntoIterator<Item = Self::Selection>) -> Self::Selection {
        values.into_iter().sum::<usize>() + 1
    }
    fn fallback_formats(&self, values: impl IntoIterator<Item = Self::Selection>) -> Self::Selection {
        values.into_iter().sum::<usize>() + 1
    }
}

#[test]
fn preferred_progressive_is_a_representation_independent_program() {
    assert_eq!(CountSyntax.preferred_progressive_format("18"), 7);
    assert_eq!(
        MediaSyntax.preferred_progressive_format("18"),
        FormatSelection::Fallback(vec![
            FormatSelection::Best(FormatPredicate::Id("18".to_owned())),
            FormatSelection::Best(FormatPredicate::All(
                Box::new(FormatPredicate::Flag { key: FORMAT_HAS_AUDIO.to_owned(), value: true }),
                Box::new(FormatPredicate::Flag { key: FORMAT_HAS_VIDEO.to_owned(), value: true }),
            )),
        ])
    );
}
