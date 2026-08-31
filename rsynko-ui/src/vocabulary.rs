use alux_ext::ext;
use rsynko_manager::*;
use std::time::Duration;

/// Denotes how much weight one piece of stated text carries.
///
/// Emphasis is presentation meaning, not mechanism: a renderer chooses the color, weight, or
/// inversion carrying it, and two renderers may disagree about that and still agree here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Emphasis {
    /// States ordinary content.
    Plain,
    /// States what the whole screen belongs to.
    Name,
    /// States what names the content under it.
    Heading,
    /// States what names one value beside it.
    Label,
    /// States the value the cursor rests on.
    Selected,
    /// States present but deliberately quiet content.
    Muted,
    /// States work in progress.
    Running,
    /// States work deliberately suspended.
    Held,
    /// States work reaching its final step.
    Finishing,
    /// States work that succeeded.
    Succeeded,
    /// States work that failed.
    Failed,
    /// States an action that cannot destroy anything.
    Safe,
    /// States an action that arms a change to somebody's files.
    Caution,
}

/// Names the lifecycle phases a reader sees.
#[ext(name = PhaseVocabularyExt)]
pub impl TransferPhase {
    /// Names the phase the way a reader reads it.
    fn phase_label(self) -> &'static str {
        match self {
            Self::Ready => "Ready",
            Self::Waiting => "Waiting",
            Self::Rehearsing => "Rehearsing",
            Self::Extracting => "Extracting",
            Self::Downloading => "Downloading",
            Self::Paused => "Paused",
            Self::Publishing => "Publishing",
            Self::Complete => "Complete",
            Self::Failed => "Failed",
        }
    }

    /// States how much weight the phase carries.
    fn phase_emphasis(self) -> Emphasis {
        match self {
            Self::Ready | Self::Waiting => Emphasis::Plain,
            Self::Rehearsing | Self::Extracting | Self::Downloading => Emphasis::Running,
            Self::Paused => Emphasis::Held,
            Self::Publishing => Emphasis::Finishing,
            Self::Complete => Emphasis::Succeeded,
            Self::Failed => Emphasis::Failed,
        }
    }

    /// States whether an interpreter is currently working on the entry.
    ///
    /// The marker is filled while something is happening to the request and open while the
    /// request rests, which is the one distinction a reader makes at a glance.
    fn phase_marker(self) -> &'static str {
        match self {
            Self::Ready | Self::Paused => "○",
            Self::Rehearsing => "◌",
            Self::Waiting
            | Self::Extracting
            | Self::Downloading
            | Self::Publishing
            | Self::Complete
            | Self::Failed => "●",
        }
    }
}

/// Names the details controls a reader sees.
#[ext(name = ControlVocabularyExt)]
pub impl DetailControl {
    /// Names the control the way a reader reads it.
    fn control_label(self) -> &'static str {
        match self {
            Self::Input => "Input",
            Self::Output => "File name",
            Self::Format => "Format",
            Self::Command => "Command",
            Self::Restart => "Restart",
            Self::DryRun => "Dry run",
            Self::Report => "Report",
            Self::Log => "Log",
            Self::Duplicate => "Duplicate",
            Self::Delete => "Delete",
        }
    }

    /// States whether the control is shown as a field stating a value rather than as an action.
    fn states_value(self) -> bool {
        matches!(
            self,
            Self::Input | Self::Output | Self::Format | Self::Command | Self::Report | Self::Log
        )
    }
}

/// Names the stream roles a reader chooses between.
#[ext(name = StreamVocabularyExt)]
pub impl MediaStreams {
    /// Names the roles the way a reader reads them.
    fn streams_label(self) -> &'static str {
        match self {
            Self::AudioVideo => "audio + video",
            Self::Audio => "audio only",
            Self::Video => "video only",
        }
    }
}

/// Names what a rehearsal states would happen.
#[ext(name = ChangeVocabularyExt)]
pub impl ChangeKind {
    /// Names the change the way a reader reads it.
    fn change_label(self) -> &'static str {
        match self {
            Self::Create => "new",
            Self::Update => "updated",
            Self::Delete => "deleted",
            Self::Unchanged => "unchanged",
        }
    }

    /// Marks the change so a reader tells one from another down a column.
    fn change_marker(self) -> &'static str {
        match self {
            Self::Create => "+",
            Self::Update => "~",
            Self::Delete => "-",
            Self::Unchanged => "=",
        }
    }

    /// States how much weight the change carries.
    fn change_emphasis(self) -> Emphasis {
        match self {
            Self::Create => Emphasis::Safe,
            Self::Update => Emphasis::Caution,
            Self::Delete => Emphasis::Failed,
            // What stays as it is is the part of a report a reader skims past.
            Self::Unchanged => Emphasis::Muted,
        }
    }
}

/// Names the meanings Space carries.
#[ext(name = SpaceVocabularyExt)]
pub impl SpaceAction {
    /// Names the transition the way a reader reads it.
    fn space_label(self) -> &'static str {
        match self {
            Self::Start => "Start",
            Self::Rehearse => "Dry run",
            Self::Pause => "Pause",
            Self::Resume => "Resume",
        }
    }
}

/// States a value no observation supplied.
pub const UNSTATED: &str = "—";

/// States a byte count in the largest unit stating it in full.
#[must_use]
pub fn bytes_label(bytes: u64) -> String {
    /// Names each unit, every one a power of 1024 above the last.
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut divisor = 1_u64;
    let mut unit = 0_usize;
    while bytes / divisor >= 1024 && unit < UNITS.len() - 1 {
        divisor = divisor.saturating_mul(1024);
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        let whole = bytes / divisor;
        let tenth = bytes % divisor / (divisor / 10);
        format!("{whole}.{tenth} {}", UNITS[unit])
    }
}

/// States a duration in minutes and seconds, and in hours once it reaches one.
#[must_use]
pub fn duration_label(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let hours = seconds / 3600;
    let minutes = seconds % 3600 / 60;
    let seconds = seconds % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

/// States text within a column budget, ending elided text with an ellipsis.
#[must_use]
pub fn elided(text: &str, columns: usize) -> String {
    if text.chars().count() <= columns {
        return text.to_owned();
    }
    text.chars()
        .take(columns.saturating_sub(1))
        .collect::<String>()
        + "…"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ARMED_MODE, CURSOR_MARK, EXPANDED_MARK, Gauge, Key, PATH_SEPARATOR, REHEARSING_MODE,
        UNSTATED,
    };
    use rsynko_manager::{ChangeKind, DetailControl, MediaStreams, SpaceAction, TransferPhase};

    /// Names every character this vocabulary states that is not plain ASCII.
    ///
    /// A terminal lays out one column per character, and so does the reader of any transcript of
    /// one. What matters is not how wide a character is but whether everyone measuring it agrees:
    /// a symbol that a font may draw as a coloured emoji is one nobody can measure, and the row
    /// it sits in stops fitting. A symbol asked for as an emoji, by the selector that follows it,
    /// is two columns wide to everyone. Adding a character here is a decision to check which of
    /// those it is, not a formality: `🔒` was ambiguous and had to be taken back out.
    const STATED: [char; 21] = [
        '█', '▏', '▎', '▍', '▌', '▋', '▊', '▉', // the eighths a gauge fills
        '○', '●', '◌', // whether an interpreter is working on a request
        '▸', '▾', // where the cursor rests, and which request is open
        '›', // the separator between the pages a page rests under
        '—', // what nothing was stated about
        '↑', '↓', '←', '→', // the keys a menu names
        '\u{26A0}', '\u{FE0F}', // a warning, and the selector making it two columns wide
    ];

    /// States every word this vocabulary states, so nothing states a character unnoticed.
    fn stated_words() -> Vec<String> {
        let phases = TransferPhase::LIFECYCLE
            .into_iter()
            .flat_map(|phase| [phase.phase_label(), phase.phase_marker()]);
        let changes = ChangeKind::REPORTED
            .into_iter()
            .flat_map(|kind| [kind.change_label(), kind.change_marker()]);
        let streams = MediaStreams::OFFERED
            .into_iter()
            .map(MediaStreams::streams_label);
        let controls = [
            DetailControl::Input,
            DetailControl::Output,
            DetailControl::Format,
            DetailControl::Command,
            DetailControl::Restart,
            DetailControl::DryRun,
            DetailControl::Report,
            DetailControl::Log,
            DetailControl::Duplicate,
            DetailControl::Delete,
        ]
        .into_iter()
        .map(DetailControl::control_label);
        let spaces = [
            SpaceAction::Start,
            SpaceAction::Rehearse,
            SpaceAction::Pause,
            SpaceAction::Resume,
        ]
        .into_iter()
        .map(SpaceAction::space_label);
        phases
            .chain(changes)
            .chain(streams)
            .chain(controls)
            .chain(spaces)
            .chain([UNSTATED, REHEARSING_MODE, ARMED_MODE])
            .map(str::to_owned)
            .chain(Gauge::LEADING.iter().map(|leading| (*leading).to_owned()))
            .chain((0..=100_u16).map(|percent| Gauge::of(percent, 8).text()))
            .chain([Key::Up, Key::Down, Key::Left, Key::Right].map(Key::label))
            .chain([CURSOR_MARK, EXPANDED_MARK, PATH_SEPARATOR].map(str::to_owned))
            .collect()
    }

    #[test]
    fn every_stated_character_is_one_a_reader_has_been_told_about() {
        let unexpected = stated_words()
            .iter()
            .flat_map(|word| word.chars().collect::<Vec<_>>())
            .filter(|stated| !stated.is_ascii() && !STATED.contains(stated))
            .collect::<Vec<_>>();

        assert!(
            unexpected.is_empty(),
            "these are stated without having been checked for width: {unexpected:?}"
        );
    }
}
