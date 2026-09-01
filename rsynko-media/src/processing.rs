use crate::ProcessingSorts;
use alux_ext::ext;
use ambassador::delegatable_trait;

/// Provides the carriers and primitive constructors for processing programs.
#[delegatable_trait]
pub trait ProcessingProgramAlg: ProcessingSorts {
    /// Defines a processor identity.
    fn processor(&self, id: impl Into<String>) -> Self::Processor;
    /// Defines application of one processor at one stage.
    fn processing_step(&self, stage: ProcessingStage, processor: Self::Processor) -> Self::Step;
    /// Defines the neutral processing program.
    fn empty_processing(&self) -> Self::Program;
    /// Defines a program from steps in declaration order.
    fn processing(&self, steps: impl IntoIterator<Item = Self::Step>) -> Self::Program;
    /// Defines sequential program composition.
    fn then_processing(&self, first: Self::Program, next: Self::Program) -> Self::Program;
}

/// Specifies observation of a reified processing program.
pub trait ProcessingProgramViewAlg: ProcessingSorts {
    /// Observes steps in declaration order.
    fn processing_steps<'a>(program: &'a Self::Program) -> impl Iterator<Item = &'a Self::Step>
    where
        Self::Step: 'a;
    /// Observes the stage selecting one step.
    fn processing_stage(step: &Self::Step) -> ProcessingStage;
}

/// Specifies application of primitive artifact-processing steps.
pub trait ProcessingApplyAlg: ProcessingSorts {
    /// Denotes a processor-specific failure.
    type Error;

    /// Applies one named artifact transformation.
    ///
    /// # Errors
    ///
    /// Returns the interpreter's error when the named transformation cannot be applied.
    fn apply_processing_step(&mut self, step: &Self::Step) -> Result<(), Self::Error>;
}

/// Names the public processing stages in their product order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProcessingStage {
    /// Runs before entry filtering.
    PreProcess,
    /// Runs after entry filtering.
    AfterFilter,
    /// Runs on each video before downloading.
    Video,
    /// Runs immediately before downloading.
    BeforeDownload,
    /// Runs after downloading and before final movement.
    PostProcess,
    /// Runs after artifacts reach their final paths.
    AfterMove,
    /// Runs after all processing for one video.
    AfterVideo,
    /// Runs after all entries of one playlist.
    Playlist,
}

/// Derives concise processing-program construction.
#[ext(name = ProcessingProgramExt)]
pub impl<This> This
where
    This: ProcessingProgramAlg,
{
    /// Defines one-step processing.
    fn process_with(&self, stage: ProcessingStage, processor: impl Into<String>) -> This::Program {
        let processor = self.processor(processor);
        let step = self.processing_step(stage, processor);
        self.processing([step])
    }
}

/// Derives complete-program and single-stage processing from primitive step application.
#[ext(name = ProcessingExt)]
pub impl<This, Step, Program> This
where
    This: ProcessingApplyAlg<Step = Step> + ProcessingProgramViewAlg<Step = Step, Program = Program>,
{
    /// Interprets every step in declaration order.
    ///
    /// # Errors
    ///
    /// Returns the first step-application error and does not apply later steps.
    fn run_processing_program(&mut self, program: &Program) -> Result<(), This::Error> {
        This::processing_steps(program).try_for_each(|step| self.apply_processing_step(step))
    }

    /// Interprets one stage while preserving relative declaration order.
    ///
    /// # Errors
    ///
    /// Returns the first selected step-application error and does not apply later selected steps.
    fn run_processing_stage(&mut self, program: &Program, stage: ProcessingStage) -> Result<(), This::Error> {
        This::processing_steps(program)
            .filter(|step| This::processing_stage(step) == stage)
            .try_for_each(|step| self.apply_processing_step(step))
    }
}
