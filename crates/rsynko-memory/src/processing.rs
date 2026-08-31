use crate::ProcessingSyntax;
use crate::{Artifact, ProcessingProgram, ProcessingStep, ProcessorId};
use rsynko_media::*;
use std::collections::BTreeMap;
use thiserror::Error;

/// Stores artifacts by identity for the in-memory processing interpreter.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ArtifactSet(BTreeMap<String, Artifact>);

impl ArtifactSet {
    /// Inserts an artifact by identity and returns the previous artifact, if any.
    pub fn insert(&mut self, artifact: Artifact) -> Option<Artifact> {
        self.0.insert(artifact.id.clone(), artifact)
    }

    /// Observes one artifact by identity.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Artifact> {
        self.0.get(id)
    }

    /// Observes one artifact mutably by identity.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Artifact> {
        self.0.get_mut(id)
    }

    /// Observes the number of stored artifacts.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Observes whether no artifacts are stored.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Denotes an inspectable artifact transformation in the reference interpreter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReferenceArtifactTransform {
    /// Denotes the identity transformation.
    Identity,
    /// Denotes moving one artifact to another path.
    Move {
        /// Identifies the artifact to move.
        artifact: String,
        /// Names its resulting path.
        path: String,
    },
    /// Denotes a transformation that always fails.
    Fail(String),
}

/// Interprets one named artifact transformation in memory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceProcessor {
    id: ProcessorId,
    transform: ReferenceArtifactTransform,
}

impl ReferenceProcessor {
    /// Constructs a deterministic artifact transformation.
    #[must_use]
    pub fn new(id: impl Into<String>, transform: ReferenceArtifactTransform) -> Self {
        Self {
            id: ProcessorId::new(id),
            transform,
        }
    }
}

/// Denotes failures of the deterministic processing interpreter.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReferenceProcessingError {
    /// Denotes application of an unregistered processor.
    #[error("unknown processor {0:?}")]
    UnknownProcessor(ProcessorId),
    /// Denotes failure of a registered processor.
    #[error("processor {processor:?} failed: {message}")]
    ProcessorFailed {
        /// Names the failing processor.
        processor: ProcessorId,
        /// Preserves its failure message.
        message: String,
    },
}

/// Interprets processing programs over an in-memory artifact set.
#[derive(Debug, Default)]
pub struct ReferenceProcessorEnv {
    artifacts: ArtifactSet,
    processors: BTreeMap<ProcessorId, ReferenceProcessor>,
    trace: Vec<ProcessingStep>,
}

impl ReferenceProcessorEnv {
    /// Constructs a processing environment with initial artifacts.
    #[must_use]
    pub fn new(artifacts: ArtifactSet) -> Self {
        Self {
            artifacts,
            processors: BTreeMap::default(),
            trace: Vec::default(),
        }
    }

    /// Registers a processor by semantic identity.
    pub fn register(&mut self, processor: ReferenceProcessor) -> Option<ReferenceProcessor> {
        self.processors.insert(processor.id.clone(), processor)
    }

    /// Observes the current artifact set.
    #[must_use]
    pub fn artifacts(&self) -> &ArtifactSet {
        &self.artifacts
    }

    /// Observes successfully applied steps in application order.
    #[must_use]
    pub fn trace(&self) -> &[ProcessingStep] {
        &self.trace
    }
}

impl ProcessingSorts for ReferenceProcessorEnv {
    type Processor = ();
    type Step = ProcessingStep;
    type Program = ProcessingProgram;
}

impl ProcessingApplyAlg for ReferenceProcessorEnv {
    type Error = ReferenceProcessingError;

    fn apply_processing_step(&mut self, step: &ProcessingStep) -> Result<(), Self::Error> {
        let processor = self
            .processors
            .get(&step.processor)
            .ok_or_else(|| ReferenceProcessingError::UnknownProcessor(step.processor.clone()))?;
        match &processor.transform {
            ReferenceArtifactTransform::Identity => {}
            ReferenceArtifactTransform::Move { artifact, path } => {
                let item = self.artifacts.get_mut(artifact).ok_or_else(|| {
                    ReferenceProcessingError::ProcessorFailed {
                        processor: step.processor.clone(),
                        message: format!("missing artifact {artifact}"),
                    }
                })?;
                item.metadata.insert(ARTIFACT_LOCATION, path.clone());
            }
            ReferenceArtifactTransform::Fail(message) => {
                return Err(ReferenceProcessingError::ProcessorFailed {
                    processor: step.processor.clone(),
                    message: message.clone(),
                });
            }
        }
        self.trace.push(step.clone());
        Ok(())
    }
}

impl ProcessingProgramViewAlg for ReferenceProcessorEnv {
    fn processing_steps<'a>(program: &'a Self::Program) -> impl Iterator<Item = &'a Self::Step>
    where
        Self::Step: 'a,
    {
        ProcessingSyntax::processing_steps(program)
    }

    fn processing_stage(step: &Self::Step) -> ProcessingStage {
        ProcessingSyntax::processing_stage(step)
    }
}
