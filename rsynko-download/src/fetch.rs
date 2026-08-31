use ambassador::delegatable_trait;
use derive_new::new;

/// Specifies incremental retrieval of one resource: a byte-stream coalgebra of carrier and step.
#[delegatable_trait]
pub trait FetchStreamAlg<Source: ?Sized = str> {
    /// Denotes retrieval failure.
    type Error;
    /// Carries one interpreter-specific open resource stream.
    type Stream;

    /// Opens a resource stream and observes its expected byte count when known.
    ///
    /// # Errors
    ///
    /// Returns the interpreter-specific retrieval error.
    fn open_fetch(&self, source: &Source) -> Result<FetchStream<Self::Stream>, Self::Error>;

    /// Reads the next resource bytes into a caller-owned buffer.
    ///
    /// # Errors
    ///
    /// Returns the interpreter-specific retrieval error.
    fn read_fetch(
        &self,
        stream: &mut Self::Stream,
        buffer: &mut [u8],
    ) -> Result<usize, Self::Error>;
}

/// Denotes an opened resource stream and its expected byte count when known.
#[derive(Debug, new)]
pub struct FetchStream<Stream> {
    /// Carries the interpreter-specific open stream.
    pub stream: Stream,
    /// Denotes the expected complete byte count when the source provides one.
    pub total: Option<u64>,
}
