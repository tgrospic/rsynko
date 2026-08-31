use crate::MediaSorts;
use alux_ext::ext;
use ambassador::delegatable_trait;

/// Provides the carriers and constructors for open metadata.
#[delegatable_trait]
pub trait MetadataAlg: MediaSorts {
    /// Defines the neutral metadata record.
    fn empty_metadata(&self) -> Self::Metadata;
    /// Defines unavailable metadata.
    fn null_metadata(&self) -> Self::Value;
    /// Defines a boolean observation.
    fn boolean_metadata(&self, value: bool) -> Self::Value;
    /// Defines an integral observation.
    fn integer_metadata(&self, value: i64) -> Self::Value;
    /// Defines a non-integral observation.
    fn float_metadata(&self, value: f64) -> Self::Value;
    /// Defines textual metadata.
    fn string_metadata(&self, value: impl Into<String>) -> Self::Value;
    /// Defines an ordered metadata sequence.
    fn list_metadata(&self, values: impl IntoIterator<Item = Self::Value>) -> Self::Value;
    /// Defines a nested metadata record.
    fn record_metadata(&self, record: Self::Metadata) -> Self::Value;
    /// Defines a record from fields, with later equal keys replacing earlier fields.
    fn metadata(&self, fields: impl IntoIterator<Item = (String, Self::Value)>) -> Self::Metadata;
}

/// Specifies reading back what one metadata record states.
///
/// A record is open: it holds whatever an extractor observed. Reading one back is asking a
/// question of it, and a field stating something else than the question asked answers nothing.
#[delegatable_trait]
pub trait MetadataViewAlg: MediaSorts {
    /// Observes the text one field states, when it states text.
    fn metadata_text<'a>(&self, metadata: &'a Self::Metadata, key: &str) -> Option<&'a str>;
    /// Observes the whole number one field states, when it states one.
    fn metadata_number(&self, metadata: &Self::Metadata, key: &str) -> Option<i64>;
}

/// Derives concise metadata records from primitive construction.
#[ext(name = MetadataExt)]
pub impl<This> This
where
    This: MetadataAlg,
{
    /// Defines a record containing one textual field.
    fn text_metadata(&self, key: impl Into<String>, value: impl Into<String>) -> This::Metadata {
        self.metadata([(key.into(), self.string_metadata(value))])
    }
}
