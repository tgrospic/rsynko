//! Checks that construction stays parametric in the carrier, using a counting interpreter.

use rsynko_media::{FormatAlg, MediaSorts, MetadataAlg, MetadataExt};
use rsynko_memory::{InfoValue, MediaSyntax};

struct CountSyntax;

impl MediaSorts for CountSyntax {
    type Value = usize;
    type Metadata = usize;
    type Format = usize;
    type Artifact = ();
    type Media = ();
    type Extraction = ();
    type Extractor = ();
    type Predicate = ();
    type Selection = ();
    type Output = ();
}

impl MetadataAlg for CountSyntax {
    fn empty_metadata(&self) -> Self::Metadata {
        0
    }
    fn null_metadata(&self) -> Self::Value {
        1
    }
    fn boolean_metadata(&self, _: bool) -> Self::Value {
        1
    }
    fn integer_metadata(&self, _: i64) -> Self::Value {
        1
    }
    fn float_metadata(&self, _: f64) -> Self::Value {
        1
    }
    fn string_metadata(&self, _: impl Into<String>) -> Self::Value {
        1
    }
    fn list_metadata(&self, values: impl IntoIterator<Item = Self::Value>) -> Self::Value {
        values.into_iter().sum()
    }
    fn record_metadata(&self, record: Self::Metadata) -> Self::Value {
        record
    }
    fn metadata(&self, fields: impl IntoIterator<Item = (String, Self::Value)>) -> Self::Metadata {
        fields.into_iter().map(|(_, value)| value).sum()
    }
}

impl FormatAlg for CountSyntax {
    fn format(&self, _: impl Into<String>, metadata: Self::Metadata) -> Self::Format {
        metadata + 1
    }
}

#[test]
fn metadata_construction_does_not_fix_record_representation() {
    let syntax = MediaSyntax.text_metadata("title", "Example");
    assert_eq!(syntax.get("title"), Some(&InfoValue::String("Example".to_owned())));
    assert_eq!(CountSyntax.text_metadata("title", "Example"), 1);
}

#[test]
fn format_construction_composes_with_the_metadata_carrier() {
    let metadata = CountSyntax.text_metadata("title", "Example");
    assert_eq!(CountSyntax.format("18", metadata), 2);
}
