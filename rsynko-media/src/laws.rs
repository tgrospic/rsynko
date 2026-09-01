//! Law scenarios, stated once over the capabilities.
//!
//! A scenario provides its own data. It builds every value through the algebras it is bound to, so
//! it constrains any interpreter of those sorts and a runner names it without supplying anything.
//! Where a law needs state only an interpreter can hold — a populated extractor catalog — the
//! interpreter supplies it through a fixture capability, and the scenario still authors the rest.

use crate::*;
use alux_ext::ext;
use anyhow::{Result, bail};
use rsynko_download::*;
use std::fmt::{Debug, Display};
use std::path::{Path, PathBuf};

/// Supplies the catalog-dependent inputs an extraction scenario cannot author for itself.
pub trait ExtractionLawFixture {
    /// Names a URL the interpreter's catalog accepts.
    fn accepted_law_url(&self) -> String;
    /// Names a URL no catalog entry accepts.
    fn unsupported_law_url(&self) -> String;
    /// Names the extractors applied so far, in application order.
    fn law_applied_extractors(&self) -> Vec<String>;
}

/// Authors the open-record laws.
#[ext(name = ObservationLaws)]
pub impl<This> This
where
    This: MetadataAlg,
{
    /// Checks that the empty record is neutral and later equal keys replace earlier ones.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn observation_laws(&self) -> Result<()>
    where
        This::Metadata: PartialEq + Debug,
        This::Value: Clone,
    {
        if self.metadata([]) != self.empty_metadata() {
            bail!("neutrality violated: a record of no fields differs from the empty record");
        }
        let stated = self.string_metadata("first");
        if self.metadata([("title".to_owned(), stated.clone())]) != self.text_metadata("title", "first") {
            bail!("single-field construction disagrees with its derived form");
        }
        let replaced =
            self.metadata([("title".to_owned(), stated), ("title".to_owned(), self.string_metadata("second"))]);
        if replaced != self.text_metadata("title", "second") {
            bail!("replacement violated: a later equal key did not replace the earlier field");
        }
        Ok(())
    }
}

/// Authors the selection-program laws.
#[ext(name = SelectionLaws)]
pub impl<This> This
where
    This: MetadataAlg
        + FormatAlg
        + FormatPredicateAlg
        + FormatSelectionAlg
        + FormatPredicateMatchAlg
        + FormatSelectionApplyAlg,
{
    /// Checks best, worst, merge, and fallback against a catalog the scenario authors.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn selection_laws(&self) -> Result<()>
    where
        This::Predicate: Clone,
    {
        let catalog = [false, true, true].map(|video| {
            self.format(
                if video { "video" } else { "audio" },
                self.metadata([(FORMAT_HAS_VIDEO.to_owned(), self.boolean_metadata(video))]),
            )
        });
        self.check_selection_against(&catalog, &self.has_video())?;
        self.check_selection_against(&catalog, &self.format_extension("never"))?;

        // Height is an ordinary observation, so bounding it needs no dedicated predicate.
        let sized = [360_i64, 720, 1080]
            .map(|height| self.format("sized", self.metadata([("height".to_owned(), self.integer_metadata(height))])));
        self.check_selection_against(&sized, &self.at_most("height", 720.0))?;
        self.check_selection_against(&sized, &self.greater_than("height", 720.0))?;
        // An unstated observation satisfies no predicate speaking about it.
        self.check_selection_against(&catalog, &self.at_most("height", 720.0))?;
        self.check_selection_against(&catalog, &self.observed_format("height"))
    }

    /// Checks the selection laws for one catalog and predicate.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn check_selection_against(&self, catalog: &[This::Format], predicate: &This::Predicate) -> Result<()>
    where
        This::Predicate: Clone,
    {
        let matching: Vec<&This::Format> =
            catalog.iter().filter(|format| self.format_matches(predicate, format)).collect();
        let best = self.select_formats(catalog, &self.best_format(predicate.clone()));
        let worst = self.select_formats(catalog, &self.worst_format(predicate.clone()));

        let (Some(first), Some(last)) = (matching.first(), matching.last()) else {
            if best.is_some() || worst.is_some() {
                bail!("selection chose a format although nothing matches the predicate");
            }
            return Ok(());
        };
        if !chose(best.as_deref(), last) {
            bail!("Best did not choose the most preferred matching format");
        }
        if !chose(worst.as_deref(), first) {
            bail!("Worst did not choose the least preferred matching format");
        }

        let merged = self.select_formats(
            catalog,
            &self.merge_formats([self.best_format(predicate.clone()), self.worst_format(predicate.clone())]),
        );
        let Some(merged) = merged else {
            bail!("Merge failed although every child succeeded");
        };
        if merged.len() != 2 || !std::ptr::eq(merged[0], *last) || !std::ptr::eq(merged[1], *first) {
            bail!("Merge did not preserve child-program order");
        }

        let fallback = self.select_formats(
            catalog,
            &self.fallback_formats([self.best_format(predicate.clone()), self.worst_format(predicate.clone())]),
        );
        if !chose(fallback.as_deref(), last) {
            bail!("Fallback was not left-biased");
        }
        Ok(())
    }
}

/// Observes whether a selection chose exactly the expected format.
fn chose<Format>(selected: Option<&[&Format]>, expected: &Format) -> bool {
    selected.is_some_and(|chosen| chosen.len() == 1 && std::ptr::eq(chosen[0], expected))
}

/// Authors the format-observation laws.
#[ext(name = FormatLaws)]
pub impl<This> This
where
    This: MetadataAlg + FormatAlg + FormatViewAlg,
{
    /// Checks that a format states back exactly what it was defined with.
    ///
    /// The laws checked are:
    ///
    /// 1. an observation a format states is the one it is observed under;
    /// 2. distinct observation names are independent;
    /// 3. an unstated observation is absent rather than defaulted.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn format_laws(&self) -> Result<()> {
        let source = "https://media.example/a.mp4";
        let stated = self.format(
            "18",
            self.metadata([
                (FORMAT_SOURCE.to_owned(), self.string_metadata(source)),
                (FORMAT_EXTENSION.to_owned(), self.string_metadata("mp4")),
            ]),
        );
        if self.format_text(&stated, FORMAT_SOURCE) != Some(source) {
            bail!("a stated source is not the one the format is observed under");
        }
        if self.format_text(&stated, FORMAT_EXTENSION) != Some("mp4") {
            bail!("stating a source disturbed the stated extension");
        }
        if let Some(observed) = self.format_text(&stated, "quality") {
            bail!("an unstated observation was observed as {observed}");
        }
        let bare = self.format("18", self.empty_metadata());
        if self.format_text(&bare, FORMAT_SOURCE).is_some() {
            bail!("a format stating nothing still locates its bytes");
        }
        Ok(())
    }
}

/// Authors the produced-artifact laws.
#[ext(name = ArtifactLaws)]
pub impl<This> This
where
    This: MetadataAlg + ArtifactAlg,
{
    /// Checks that an artifact is exactly its identity, its role, and its observations.
    ///
    /// The laws checked are:
    ///
    /// 1. definition is a function of identity, role, and observations;
    /// 2. each of the three is significant, so changing one changes the artifact.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn artifact_laws(&self) -> Result<()>
    where
        This::Artifact: PartialEq + Debug,
        This::Metadata: Clone,
    {
        let resting = self.metadata([(ARTIFACT_LOCATION.to_owned(), self.string_metadata("temporary.part"))]);
        let stated = self.artifact("media", ArtifactKind::Media, resting.clone());
        if stated != self.artifact("media", ArtifactKind::Media, resting.clone()) {
            bail!("definition is not a function of identity, role, and observations");
        }
        for (name, other) in [
            ("identity", self.artifact("subtitle", ArtifactKind::Media, resting.clone())),
            ("role", self.artifact("media", ArtifactKind::Subtitle, resting)),
            ("observations", self.artifact("media", ArtifactKind::Media, self.empty_metadata())),
        ] {
            if stated == other {
                bail!("artifact {name} is not significant: {stated:?} equals {other:?}");
            }
        }
        Ok(())
    }
}

/// Supplies the application trace a processing scenario cannot author for itself.
pub trait ProcessingLawFixture {
    /// Names the processors applied, in application order.
    fn law_applied(&self) -> Vec<String>;
    /// Refuses the named processor, so application fails when it is reached.
    fn refuse_law_processor(&mut self, id: &str);
}

/// Authors the processing-application laws.
#[ext(name = ProcessingApplicationLaws)]
pub impl<This> This
where
    This: ProcessingProgramAlg + ProcessingProgramViewAlg + ProcessingApplyAlg + ProcessingLawFixture,
{
    /// Checks that interpreting one stage applies its subsequence in declaration order.
    ///
    /// # Errors
    ///
    /// Returns the violated law, or the interpreter's application failure.
    fn processing_stage_laws(&mut self) -> Result<()>
    where
        This::Error: Debug,
    {
        let program = self.processing([
            self.processing_step(ProcessingStage::PreProcess, self.processor("before")),
            self.processing_step(ProcessingStage::AfterMove, self.processor("first")),
            self.processing_step(ProcessingStage::AfterMove, self.processor("second")),
        ]);
        if let Err(error) = self.run_processing_stage(&program, ProcessingStage::AfterMove) {
            bail!("stage interpretation failed: {error:?}");
        }
        let applied = self.law_applied();
        if applied != ["first", "second"] {
            bail!("stage selection did not preserve relative declaration order: {applied:?}");
        }
        Ok(())
    }

    /// Checks that a failing step stops interpretation before every later step.
    ///
    /// # Errors
    ///
    /// Returns the violated law.
    fn processing_failure_laws(&mut self) -> Result<()> {
        let program = self.processing([
            self.processing_step(ProcessingStage::PostProcess, self.processor("first")),
            self.processing_step(ProcessingStage::PostProcess, self.processor("refused")),
            self.processing_step(ProcessingStage::PostProcess, self.processor("never")),
        ]);
        self.refuse_law_processor("refused");
        if self.run_processing_program(&program).is_ok() {
            bail!("a refused step did not fail the program");
        }
        let applied = self.law_applied();
        if applied != ["first"] {
            bail!("failure did not stop interpretation before later steps: {applied:?}");
        }
        Ok(())
    }
}

/// Authors the processing-program laws.
#[ext(name = ProcessingLaws)]
pub impl<This> This
where
    This: ProcessingProgramAlg + ProcessingProgramViewAlg,
{
    /// Checks that the empty program is an identity and concatenation is sequential composition.
    ///
    /// Stage selection and failure ordering are application laws: they constrain the order steps
    /// are *applied* in, which only an interpreter's trace can observe.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn processing_laws(&self) -> Result<()>
    where
        This::Step: Clone + PartialEq + Debug,
    {
        let authored: Vec<This::Step> =
            [(ProcessingStage::PostProcess, "first"), (ProcessingStage::AfterMove, "second")]
                .into_iter()
                .map(|(stage, name)| self.processing_step(stage, self.processor(name)))
                .collect();

        if Self::processing_steps(&self.processing([])).next().is_some() {
            bail!("the empty program is not empty");
        }
        let program = self.processing(authored.iter().cloned());
        let declared: Vec<This::Step> = Self::processing_steps(&program).cloned().collect();
        if declared != authored {
            bail!("construction did not preserve declaration order: {declared:?}");
        }
        for (side, composed) in [
            ("left", self.then_processing(self.processing([]), self.processing(authored.iter().cloned()))),
            ("right", self.then_processing(self.processing(authored.iter().cloned()), self.processing([]))),
        ] {
            let observed: Vec<This::Step> = Self::processing_steps(&composed).cloned().collect();
            if observed != authored {
                bail!("the empty program is not an identity on the {side}: {observed:?}");
            }
        }
        let doubled =
            self.then_processing(self.processing(authored.iter().cloned()), self.processing(authored.iter().cloned()));
        let observed: Vec<This::Step> = Self::processing_steps(&doubled).cloned().collect();
        let expected: Vec<This::Step> = authored.iter().chain(&authored).cloned().collect();
        if observed != expected {
            bail!("concatenation is not sequential composition: {observed:?}");
        }
        Ok(())
    }
}

/// Authors the portable-output-naming laws.
#[ext(name = OutputNameLaws)]
pub impl<This> This
where
    This: OutputNameAlg,
{
    /// Checks that naming is a function of its inputs and yields one safe file component.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn output_naming_laws(&self) -> Result<()> {
        let named = portable_file_name(Some("A/B"), "id", Some("mp4"));
        if named != portable_file_name(Some("A/B"), "id", Some("mp4")) {
            bail!("naming is not a function of title, fallback, and extension");
        }
        if portable_file_name(None, "id", Some("mp4")) != portable_file_name(Some("   "), "id", Some("mp4")) {
            bail!("a blank title did not select the stated fallback identity");
        }
        for candidate in [&named, &portable_user_file_name("a/b.mp4", Some("mp4"))] {
            if candidate.components().count() != 1 {
                bail!("naming produced a path rather than one file component: {}", candidate.display());
            }
        }
        if portable_user_file_name("edited.mp4", Some("bin")).extension().and_then(|value| value.to_str())
            != Some("mp4")
        {
            bail!("an edited name did not keep its valid explicit extension");
        }
        Ok(())
    }
}

/// Authors the ordered-extractor-choice laws over a catalog the interpreter supplies.
#[ext(name = ExtractionLaws)]
pub impl<This> This
where
    This: ExtractionCatalogAlg + ExtractionApplyAlg + ExtractionLawFixture,
{
    /// Checks that choice is first-accepting and that extraction follows exactly that choice.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn extraction_laws(&self) -> Result<()>
    where
        This::Extractor: PartialEq + Debug,
        This::Extraction: Debug,
        This::Error: Debug,
    {
        let accepted = self.accepted_law_url();
        let unsupported = self.unsupported_law_url();
        let expected = self.extractor_keys().find(|key| self.extractor_accepts(key, &accepted));
        let Some(expected) = expected else {
            bail!("the fixture URL {accepted} is accepted by no catalog entry");
        };
        match self.choose_extractor(&accepted) {
            Some(chosen) if chosen == expected => {}
            other => bail!("first-accepting choice violated: chose {other:?}"),
        }
        if let Some(chosen) = self.choose_extractor(&unsupported) {
            bail!("unsupported input chose {chosen:?}, expected no extractor");
        }
        if let Err(error) = self.extract_url(&accepted) {
            bail!("accepted input failed to extract: {error:?}");
        }
        let applied = self.law_applied_extractors();
        if applied.len() != 1 {
            bail!("extracting a supported URL applied {applied:?}, expected exactly one extractor");
        }
        match self.extract_url(&unsupported) {
            Err(ExtractUrlError::Unsupported { url }) if url == unsupported => {}
            other => bail!("unsupported input denoted {other:?}, expected an unsupported error"),
        }
        if self.law_applied_extractors() != applied {
            bail!("extracting an unsupported URL applied an extractor");
        }
        Ok(())
    }
}

/// Authors the composed media-download laws.
#[ext(name = MediaProgramLaws)]
pub impl<This, Source, Event, Progress> This
where
    This: MetadataAlg
        + FormatAlg
        + FormatViewAlg
        + ExtractionAlg
        + ExtractionViewAlg
        + MediaViewAlg
        + FormatPredicateAlg
        + FormatSelectionAlg
        + FormatSelectionApplyAlg
        + FormatSourceAlg<Source>
        + FetchStreamAlg<Source, Error: Display>
        + AtomicPublishAlg<Error: Display>
        + DownloadObservationAlg<Event = Event, Progress = Progress>
        + DownloadProgressAlg<Progress = Progress>
        + DownloadReportAlg<Event = Event>
        + MediaProgramLawFixture,
{
    /// Checks that the composed program selects one format and names its destination from it.
    ///
    /// The laws checked are:
    ///
    /// 1. a result that is not a single media item is refused before anything is retrieved;
    /// 2. naming depends only on the extraction, the selected format, and the output target;
    /// 3. the selected format's source reaches generic download unchanged, so the published bytes
    ///    are exactly the ones the source denotes.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn media_program_laws(&mut self) -> Result<()> {
        let format = self.format(
            "18",
            self.metadata([
                (FORMAT_SOURCE.to_owned(), self.string_metadata(self.located_law_source())),
                (FORMAT_EXTENSION.to_owned(), self.string_metadata(self.law_extension())),
            ]),
        );
        let extraction = self.media("law-media", self.empty_metadata(), [format]);
        let selection = self.best_format(self.any_format());

        let published = match self.download_extraction(extraction, &selection, &OutputTarget::MediaId) {
            Ok(path) => path,
            Err(error) => bail!("the composed program failed: {error}"),
        };
        let expected = PathBuf::from(format!("law-media.{}", self.law_extension()));
        if published != expected {
            bail!("naming derived {} rather than {}", published.display(), expected.display());
        }
        if self.law_published_bytes(&published).as_deref() != Some(&self.expected_law_bytes()[..]) {
            bail!("the selected format's source did not reach download unchanged");
        }

        let reference = self.url_reference("https://example.test/other", None, false);
        match self.download_extraction(reference, &selection, &OutputTarget::MediaId) {
            Err(MediaDownloadError::NotSingleMedia) => Ok(()),
            other => bail!("a non-media result was not refused: {}", describe(&other)),
        }
    }
}

/// Supplies the retrieval inputs a media-program scenario cannot author for itself.
pub trait MediaProgramLawFixture {
    /// Names a source the interpreter can retrieve.
    fn located_law_source(&self) -> String;
    /// Names the extension the scenario states on its format.
    fn law_extension(&self) -> String;
    /// States the exact bytes the source denotes.
    fn expected_law_bytes(&self) -> Vec<u8>;
    /// Observes bytes published at one path.
    fn law_published_bytes(&self, path: &Path) -> Option<Vec<u8>>;
}

/// Describes a composed-program outcome without requiring its carriers to be printable.
fn describe<Ok, Error: Display>(outcome: &Result<Ok, Error>) -> String {
    match outcome {
        Ok(_) => "a successful download".to_owned(),
        Err(error) => error.to_string(),
    }
}
