//! Checks that construction stays parametric in the carrier, using a counting interpreter.

use rsynko_media::{ProcessingProgramAlg, ProcessingProgramExt, ProcessingSorts, ProcessingStage};
use rsynko_memory::ProcessingSyntax;

struct CountSyntax;

impl ProcessingSorts for CountSyntax {
    type Processor = usize;
    type Step = usize;
    type Program = usize;
}

impl ProcessingProgramAlg for CountSyntax {
    fn processor(&self, _: impl Into<String>) -> Self::Processor {
        1
    }
    fn processing_step(&self, _: ProcessingStage, processor: Self::Processor) -> Self::Step {
        processor
    }
    fn empty_processing(&self) -> Self::Program {
        0
    }
    fn processing(&self, steps: impl IntoIterator<Item = Self::Step>) -> Self::Program {
        steps.into_iter().sum()
    }
    fn then_processing(&self, first: Self::Program, next: Self::Program) -> Self::Program {
        first + next
    }
}

#[test]
fn sequential_construction_has_a_neutral_program_for_multiple_interpreters() {
    let counted = CountSyntax.process_with(ProcessingStage::BeforeDownload, "prepare");
    assert_eq!(
        CountSyntax.then_processing(CountSyntax.empty_processing(), counted),
        1
    );

    let program = ProcessingSyntax.process_with(ProcessingStage::BeforeDownload, "prepare");
    let composed = ProcessingSyntax.then_processing(ProcessingSyntax.empty_processing(), program);
    assert_eq!(composed.steps().count(), 1);
}
