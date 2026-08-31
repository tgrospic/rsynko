use crate::{FORMAT_EXTENSION, FORMAT_HAS_AUDIO, FORMAT_HAS_VIDEO, MediaSorts};
use alux_ext::ext;
use ambassador::delegatable_trait;

/// Provides the carrier and primitive constructors for format predicates.
#[delegatable_trait]
pub trait FormatPredicateAlg: MediaSorts {
    /// Defines the predicate accepting every format.
    fn any_format(&self) -> Self::Predicate;
    /// Defines the predicate accepting one format identity.
    fn format_id(&self, id: impl Into<String>) -> Self::Predicate;
    /// Defines the predicate accepting formats stating the named observation.
    fn observed_format(&self, key: impl Into<String>) -> Self::Predicate;
    /// Defines the predicate accepting one textual observation.
    fn text_format(&self, key: impl Into<String>, value: impl Into<String>) -> Self::Predicate;
    /// Defines the predicate accepting one stated truth.
    fn flag_format(&self, key: impl Into<String>, value: bool) -> Self::Predicate;
    /// Defines the predicate comparing one numeric observation.
    fn number_format(
        &self,
        key: impl Into<String>,
        comparison: FormatComparison,
        value: f64,
    ) -> Self::Predicate;
    /// Defines conjunction.
    fn all_formats(&self, left: Self::Predicate, right: Self::Predicate) -> Self::Predicate;
    /// Defines negation.
    fn not_format(&self, predicate: Self::Predicate) -> Self::Predicate;
}

/// Provides the carrier and primitive constructors for selection programs.
#[delegatable_trait]
pub trait FormatSelectionAlg: MediaSorts {
    /// Defines selection of the most preferred matching format.
    fn best_format(&self, predicate: Self::Predicate) -> Self::Selection;
    /// Defines selection of the least preferred matching format.
    fn worst_format(&self, predicate: Self::Predicate) -> Self::Selection;
    /// Defines ordered combination requiring every child selection.
    fn merge_formats(
        &self,
        selections: impl IntoIterator<Item = Self::Selection>,
    ) -> Self::Selection;
    /// Defines left-biased choice of the first successful child selection.
    fn fallback_formats(
        &self,
        selections: impl IntoIterator<Item = Self::Selection>,
    ) -> Self::Selection;
}

/// Denotes how an observed number is compared against a stated one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatComparison {
    /// Accepts an observation no greater than the stated number.
    AtMost,
    /// Accepts an observation equal to the stated number.
    Exactly,
    /// Accepts an observation no smaller than the stated number.
    AtLeast,
}

/// Specifies interpretation of reified format predicates.
///
/// Matching is total over the reified predicate, so the derived meaning is the default and an
/// interpreter implements this capability without restating it.
#[delegatable_trait]
pub trait FormatPredicateMatchAlg: MediaSorts {
    /// Observes whether a format satisfies a predicate.
    fn format_matches(&self, predicate: &Self::Predicate, format: &Self::Format) -> bool;
}

/// Specifies interpretation of a selection program against a preference-ordered catalog.
///
/// Interpreting reified syntax is an interpreter, not a derived operation: a program is walked, and
/// what walking means belongs to whoever chose the selection sort.
#[delegatable_trait]
pub trait FormatSelectionApplyAlg: MediaSorts {
    /// Chooses the formats one program denotes, in program order.
    fn select_formats<'a>(
        &self,
        formats: &'a [Self::Format],
        selection: &Self::Selection,
    ) -> Option<Vec<&'a Self::Format>>;
}

/// Derives named and comparative predicates from primitive observation construction.
#[ext(name = FormatPredicateExt)]
pub impl<This> This
where
    This: FormatPredicateAlg,
{
    /// Defines formats containing audio.
    fn has_audio(&self) -> This::Predicate {
        self.flag_format(FORMAT_HAS_AUDIO, true)
    }

    /// Defines formats containing video.
    fn has_video(&self) -> This::Predicate {
        self.flag_format(FORMAT_HAS_VIDEO, true)
    }

    /// Defines formats stating one container or filename extension.
    fn format_extension(&self, extension: impl Into<String>) -> This::Predicate {
        self.text_format(FORMAT_EXTENSION, extension)
    }

    /// Defines formats whose observation is no greater than the stated number.
    fn at_most(&self, key: impl Into<String>, value: f64) -> This::Predicate {
        self.number_format(key, FormatComparison::AtMost, value)
    }

    /// Defines formats whose observation is no smaller than the stated number.
    fn at_least(&self, key: impl Into<String>, value: f64) -> This::Predicate {
        self.number_format(key, FormatComparison::AtLeast, value)
    }

    /// Defines formats whose observation equals the stated number.
    fn exactly(&self, key: impl Into<String>, value: f64) -> This::Predicate {
        self.number_format(key, FormatComparison::Exactly, value)
    }

    /// Defines formats whose observation is smaller than the stated number.
    fn less_than(&self, key: impl Into<String>, value: f64) -> This::Predicate {
        self.not_format(self.at_least(key, value))
    }

    /// Defines formats whose observation is greater than the stated number.
    fn greater_than(&self, key: impl Into<String>, value: f64) -> This::Predicate {
        self.not_format(self.at_most(key, value))
    }

    /// Defines formats containing both audio and video.
    fn progressive_format(&self) -> This::Predicate {
        self.all_formats(self.has_audio(), self.has_video())
    }

    /// Defines formats containing audio and no video.
    fn audio_only_format(&self) -> This::Predicate {
        self.all_formats(self.has_audio(), self.not_format(self.has_video()))
    }

    /// Defines formats containing video and no audio.
    fn video_only_format(&self) -> This::Predicate {
        self.all_formats(self.has_video(), self.not_format(self.has_audio()))
    }
}

/// Derives common selection programs by equating predicate carriers.
#[ext(name = FormatSelectionProgramExt)]
pub impl<This, Predicate> This
where
    This: FormatPredicateAlg<Predicate = Predicate> + FormatSelectionAlg<Predicate = Predicate>,
{
    /// Defines the most preferred progressive format.
    fn best_progressive_format(&self) -> This::Selection {
        self.best_format(self.progressive_format())
    }

    /// Defines an exact format with a progressive fallback.
    fn preferred_progressive_format(&self, id: impl Into<String>) -> This::Selection {
        self.fallback_formats([
            self.best_format(self.format_id(id)),
            self.best_progressive_format(),
        ])
    }
}
