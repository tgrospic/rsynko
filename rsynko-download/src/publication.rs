use ambassador::delegatable_trait;
use std::path::Path;

/// Specifies atomic publication at a final path.
#[delegatable_trait]
pub trait AtomicPublishAlg {
    /// Denotes publication failure.
    type Error;
    /// Carries one unpublished destination representation.
    type Publication;

    /// Begins publication without making the final destination observable.
    ///
    /// # Errors
    ///
    /// Returns the interpreter-specific publication error.
    fn begin_publication(&self, destination: &Path) -> Result<Self::Publication, Self::Error>;

    /// Appends bytes to an unpublished destination.
    ///
    /// # Errors
    ///
    /// Returns the interpreter-specific publication error.
    fn write_publication(&self, publication: &mut Self::Publication, bytes: &[u8]) -> Result<(), Self::Error>;

    /// Atomically makes a complete publication observable at its final destination.
    ///
    /// # Errors
    ///
    /// Returns the interpreter-specific publication error.
    fn commit_publication(&self, publication: Self::Publication) -> Result<(), Self::Error>;

    /// Abandons an incomplete publication and removes its partial representation.
    fn abort_publication(&self, publication: Self::Publication);
}
