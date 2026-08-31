use crate::ManagerSorts;
use ambassador::delegatable_trait;

/// Specifies the record of what an interpreter observed about one request.
///
/// A note is stated by whichever interpreter observed it — extraction, discovery, retrieval — so
/// the manager states only that the record exists, that notes are keyed by stable identity, and
/// that they are observed in the order they were stated.
#[delegatable_trait]
pub trait DownloadLogAlg: ManagerSorts {
    /// Appends one note to the record of one stable identity.
    fn note_download(&mut self, id: Self::Id, note: String);
}
