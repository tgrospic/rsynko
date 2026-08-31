use ambassador::delegatable_trait;

/// Specifies add-sources draft storage.
#[delegatable_trait]
pub trait DraftStateAlg {
    /// Observes the current editor draft.
    fn draft(&self) -> &str;
    /// Replaces the editor draft.
    fn set_draft(&mut self, draft: String);
}

/// Specifies input-editor draft storage.
#[delegatable_trait]
pub trait InputDraftAlg {
    /// Observes the input editor draft.
    fn input_draft(&self) -> &str;
    /// Replaces the input editor draft.
    fn set_input_draft(&mut self, draft: String);
}

/// Specifies output-file-name draft storage.
#[delegatable_trait]
pub trait OutputDraftAlg {
    /// Observes the output-file-name editor draft.
    fn output_draft(&self) -> &str;
    /// Replaces the output-file-name editor draft.
    fn set_output_draft(&mut self, draft: String);
}
