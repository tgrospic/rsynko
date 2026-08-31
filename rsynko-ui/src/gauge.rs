/// States how much of a fixed-width track one completed share fills.
///
/// A terminal column divides into eighths, which is the finest a bar states progress at without
/// claiming a resolution the display does not have. The unfilled remainder is the track itself,
/// so a bar states how far it reaches without drawing anything to say so.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gauge {
    /// Counts the fully filled cells.
    pub filled: usize,
    /// Counts the eighths of the cell the fill ends inside, from none to seven.
    pub leading: usize,
    /// Counts the cells the fill has not reached.
    pub track: usize,
}

impl Gauge {
    /// Names the leading fraction of a cell, from none to seven eighths.
    pub const LEADING: [&'static str; 8] = ["", "▏", "▎", "▍", "▌", "▋", "▊", "▉"];

    /// States one completed share within a track of the stated width.
    #[must_use]
    pub fn of(percent: u16, width: usize) -> Self {
        let eighths = width * Self::LEADING.len() * usize::from(percent.min(100)) / 100;
        let filled = eighths / Self::LEADING.len();
        let leading = eighths % Self::LEADING.len();
        Self {
            filled,
            leading,
            track: width - filled - usize::from(leading > 0),
        }
    }

    /// Counts the cells the gauge occupies, filled and unfilled alike.
    #[must_use]
    pub fn width(&self) -> usize {
        self.filled + usize::from(self.leading > 0) + self.track
    }

    /// States the gauge as the text a renderer draws, the track left blank.
    #[must_use]
    pub fn text(&self) -> String {
        format!(
            "{}{}{}",
            "█".repeat(self.filled),
            Self::LEADING[self.leading],
            " ".repeat(self.track)
        )
    }
}
