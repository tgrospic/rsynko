# rsynko-media

`rsynko-media` states what a media item is, how it is described, chosen, produced, and named, and how those meanings compose into one download program. These specifications are always used together, so they live together as modules of one package. It chooses no extractor, network library, serialization format, or filesystem.

## Observation

An observation record is an open, keyed, heterogeneous collection of values. It is open because an extractor may state fields the core has never heard of, keyed because later fields with equal keys replace earlier ones, and heterogeneous because sources describe themselves in whatever shapes they use. [`MetadataAlg`] constructs records and values in an interpreter-chosen carrier; `InfoRecord` and `InfoValue` reify them.

A **format** denotes one alternative representation of a media item: an identity together with everything observed about it. It fixes no schema. Where its bytes rest, which streams it carries, its container, and every quantity selection compares are observations. An **artifact** denotes one produced file the same way, classified by the role it plays.

[`FormatAlg`] defines one format from an identity and everything observed about it, and [`FormatViewAlg`] observes one named value back. A derived operation composes the three without naming a representation:

```rust
use alux_ext::ext;
use rsynko_media::{FORMAT_EXTENSION, FORMAT_SOURCE, FormatAlg, FormatViewAlg, MetadataAlg};

#[ext(name = LocatedFormatExt)]
impl<This> This
where
    This: MetadataAlg + FormatAlg + FormatViewAlg,
{
    /// Defines one format from where its bytes rest and the container it states.
    fn located_format(&self, id: &str, url: &str, extension: &str) -> This::Format {
        self.format(
            id,
            self.metadata([
                (FORMAT_SOURCE.to_owned(), self.string_metadata(url)),
                (FORMAT_EXTENSION.to_owned(), self.string_metadata(extension)),
            ]),
        )
    }

    /// Observes where a format's bytes rest, exactly when it stated that.
    fn located_source<'a>(&self, format: &'a This::Format) -> Option<&'a str> {
        self.format_text(format, FORMAT_SOURCE)
    }
}
```
The well-known keys named here — `FORMAT_SOURCE`, `FORMAT_EXTENSION`, `FORMAT_HAS_AUDIO`, `FORMAT_HAS_VIDEO`, `ARTIFACT_LOCATION` — are conventions, not fields. An extractor that states them gains the derived observations; one that does not is still complete, and an operation needing an unstated observation says so rather than inventing a value.

## Extraction

An extraction result denotes one playable media description, a reference requiring another extraction, or an ordered collection of further results. `ExtractionCatalogAlg` exposes extractor order and acceptance, `ExtractionApplyAlg` applies one selected key, and `ExtractionExt` derives first-accepting choice and URL extraction from exactly those two.

[`ExtractionAlg`] defines those three shapes, [`ExtractionViewAlg`] observes when a result denotes exactly one media item, and [`MediaViewAlg`] observes that item's identity, title, and formats in extractor order.

## Selection

A catalog's order means increasing preference. Every predicate primitive except identity ranges over observations, so one set of primitives speaks about any field an extractor recorded. `FormatPredicateExt` derives the named predicates — `has_audio`, `has_video`, `format_extension` — and the comparisons `at_most`, `at_least`, `exactly`, `less_than`, and `greater_than` from them. `FormatPredicateMatchAlg` observes whether one format satisfies one predicate, and `FormatSelectionApplyAlg` interprets a whole program against a catalog. Interpreting reified syntax is an interpreter rather than a derived operation, so `interpret_selection` states the canonical fold and an interpreter that chose the reified sorts delegates to it.

[`FormatPredicateAlg`] constructs the primitives, [`FormatSelectionAlg`] composes them into best, worst, merge, and fallback programs, and [`FormatSelectionApplyAlg`] interprets a whole program against a catalog.

Interpreting that program against a catalog is `FormatSelectionApplyAlg`, and a reference interpreter for these sorts lives in `rsynko-memory`.

## Processing and output

A processing program denotes an ordered sequence of named artifact transformations, each assigned to one stage. Interpreting a program applies every step in declaration order; interpreting a stage applies the subsequence belonging to it without reordering. Portable output naming is a separate pure meaning: it admits one Linux and Windows safe file component, bounds the stem, and falls back to a stated identity.

## The composed program

`FormatSourceAlg` is the only bridge from a selected format to a retrieval carrier, and it may state that a format cannot be located from what was observed. `MediaDownloadExt::download_extraction` and `ApplicationExt::download_url` compose extraction, selection, naming, and one-resource download over that bridge. A protocol specification specializes the source description while preserving every other meaning.

```rust
use alux_ext::ext;
use rsynko_download::{
    AtomicPublishAlg, DownloadObservationAlg, DownloadProgressAlg, DownloadReportAlg,
    FetchStreamAlg,
};
use rsynko_media::{
    ExtractionViewAlg, FormatPredicateAlg, FormatSelectionAlg, FormatSelectionApplyAlg,
    FormatSelectionProgramExt, FormatSourceAlg, FormatViewAlg, MediaDownloadError,
    MediaDownloadExt, MediaViewAlg, OutputTarget,
};
use std::fmt::Display;
use std::path::PathBuf;

#[ext(name = PreferredMediaExt)]
impl<This, Source, FetchError, Stream, PublishError, Event, Progress> This
where
    This: ExtractionViewAlg
        + MediaViewAlg
        + FormatViewAlg
        + FormatPredicateAlg
        + FormatSelectionAlg
        + FormatSelectionApplyAlg
        + FormatSourceAlg<Source>
        + FetchStreamAlg<Source, Error = FetchError, Stream = Stream>
        + AtomicPublishAlg<Error = PublishError>
        + DownloadObservationAlg<Event = Event, Progress = Progress>
        + DownloadProgressAlg<Progress = Progress>
        + DownloadReportAlg<Event = Event>,
    FetchError: Display,
    PublishError: Display,
{
    /// Publishes one extraction through the named format, falling back to the best progressive one.
    fn download_preferred(
        &self,
        extraction: This::Extraction,
        format_id: &str,
        target: &OutputTarget,
    ) -> Result<PathBuf, MediaDownloadError<FetchError, PublishError>> {
        let selection = self.preferred_progressive_format(format_id);
        self.download_extraction(extraction, &selection, target)
    }
}
```

## Laws

- the empty observation record is neutral, later equal keys replace earlier ones, and observation is total into an option;
- an unstated truth denotes falsity, and a typed observation succeeds exactly when the stated value has that type;
- choosing returns the first accepting extractor, and no extractor exactly when no catalog entry accepts the URL;
- extracting a supported URL applies exactly the chosen extractor and an unsupported URL applies none;
- `Best` and `Worst` differ only by which end of the same preference order they choose;
- `Merge` succeeds exactly when every child succeeds and preserves child order, and `Fallback` is left-biased;
- a predicate speaking about an unstated observation accepts no format;
- the empty processing program is an identity, concatenation denotes sequential composition, stage selection preserves declaration order, and failure stops interpretation before later steps;
- portable naming depends only on the stated title, fallback identity, and extension;
- extraction happens before selection, exactly one selected format is required, naming depends only on the extraction, selected format, and output target, and the source `FormatSourceAlg` produces is passed unchanged to the generic download meaning.
