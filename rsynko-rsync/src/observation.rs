use crate::*;
use alux_ext::ext;
use ambassador::delegatable_trait;
use rsynko_manager::ChangeKind;

/// Provides the carrier and constructor for one path a transfer would change.
#[delegatable_trait]
pub trait SyncChangeAlg: RsyncSorts {
    /// Defines one change from the path it names, what happens to it, and what it moves.
    fn sync_change(
        &self,
        path: impl Into<String>,
        kind: ChangeKind,
        size: Option<u64>,
    ) -> Self::Change;
}

/// Provides the carriers and constructors for what a running transfer states.
#[delegatable_trait]
pub trait SyncObservationAlg: RsyncSorts {
    /// Defines the observation of one path the transfer would change, or has changed.
    fn observed_change(&self, change: Self::Change) -> Self::Observation;
    /// Defines the observation of how far the whole transfer has come.
    fn observed_progress(&self, transferred: u64, percent: u16) -> Self::Observation;
    /// Defines the observation of a line naming nothing this specification reads.
    fn observed_nothing(&self) -> Self::Observation;
}

/// Specifies what one observation of a running transfer states.
#[delegatable_trait]
pub trait SyncObservationViewAlg: RsyncSorts {
    /// Observes the changed path, exactly when the observation states one.
    fn observation_change<'a>(
        &self,
        observation: &'a Self::Observation,
    ) -> Option<&'a Self::Change>;
    /// Observes the bytes moved and the share completed, exactly when the observation states them.
    fn observation_progress(&self, observation: &Self::Observation) -> Option<(u64, u16)>;
}

/// Names the mark a removal states instead of a change flag.
pub const DELETION_MARK: &str = "*deleting";

/// Names the field separator each stated path is written with.
pub const FIELD_MARK: char = '|';

/// Derives what one line a running transfer wrote states.
#[ext(name = SyncReadExt)]
pub impl<This> This
where
    This: SyncChangeAlg + SyncObservationAlg,
{
    /// Reads one line the transfer program wrote as what it states.
    ///
    /// A change is stated in three fields: the flags naming what happens, the byte count, and the
    /// path. Anything else on the stream is progress, or is nothing this specification reads.
    fn read_sync_line(&self, line: &str) -> This::Observation {
        let line = line.trim_end();
        if let Some(change) = self.read_sync_change(line) {
            return self.observed_change(change);
        }
        read_advance(line).map_or_else(
            || self.observed_nothing(),
            |(transferred, percent)| self.observed_progress(transferred, percent),
        )
    }

    /// Reads one itemized line as the change it denotes, and states nothing for a line naming
    /// only that a folder was opened on the way to what is inside it.
    fn read_sync_change(&self, line: &str) -> Option<This::Change> {
        let mut fields = line.splitn(3, FIELD_MARK);
        let flags = fields.next()?.trim();
        let size = fields.next()?.trim();
        let path = fields.next()?.trim();
        if path.is_empty() || touches_folder(flags) {
            return None;
        }
        let kind = read_kind(flags)?;
        // A removal is stated without a size, because the transfer never looked at what it holds.
        let size = (kind != ChangeKind::Delete)
            .then(|| read_count(size))
            .flatten();
        Some(self.sync_change(path, kind, size))
    }
}

/// Observes whether the flags name a folder that is only having its own attributes brought up
/// to date.
///
/// Transferring a folder's contents restates the folder itself, which a reader has already been
/// told about by the paths inside it. A folder that did not exist before is stated as it is.
fn touches_folder(flags: &str) -> bool {
    let mut marks = flags.chars();
    let happening = marks.next();
    let kind_of_path = marks.next();
    matches!(kind_of_path, Some('d'))
        && matches!(happening, Some('.' | 'c' | '>' | '<'))
        && !marks.as_str().starts_with("+++")
}

/// Reads the itemized flags as what would happen to the path they name.
///
/// A removal states itself in words. Everything else states eleven marks: what is happening, what
/// kind of path it is, and then one mark per attribute that differs. All-new content states
/// pluses; nothing differing states dots.
fn read_kind(flags: &str) -> Option<ChangeKind> {
    if flags.starts_with(DELETION_MARK) {
        return Some(ChangeKind::Delete);
    }
    let mut marks = flags.chars();
    let happening = marks.next()?;
    if !matches!(happening, '<' | '>' | 'c' | 'h' | '.') {
        return None;
    }
    let _kind_of_path = marks.next()?;
    let attributes = marks.as_str();
    if attributes.starts_with("+++") {
        return Some(ChangeKind::Create);
    }
    if happening == '.' && attributes.chars().all(|mark| mark == '.' || mark == ' ') {
        return Some(ChangeKind::Unchanged);
    }
    Some(ChangeKind::Update)
}

/// Reads one progress line as the bytes it has moved and the share it has completed.
fn read_advance(line: &str) -> Option<(u64, u16)> {
    let mut fields = line.split_whitespace();
    let transferred = read_count(fields.next()?)?;
    let percent = fields.next()?.strip_suffix('%')?.parse::<u16>().ok()?;
    Some((transferred, percent.min(100)))
}

/// Reads a count the transfer program wrote, which it groups for a reader.
fn read_count(text: &str) -> Option<u64> {
    let digits = text.replace([',', '.', '\u{a0}'], "");
    (!digits.is_empty() && digits.chars().all(|digit| digit.is_ascii_digit()))
        .then(|| digits.parse().ok())
        .flatten()
}
