use clap::Parser;
use rsynko_media::{FormatPredicateAlg, FormatPredicateExt, FormatSelectionAlg, OutputTarget};
use rsynko_memory::{FormatSelection, MediaSyntax};
use std::path::PathBuf;

#[derive(Clone, Debug, Parser)]
#[command(name = "rsynko", version, about)]
/// States what a reader asked for on the command line.
pub(crate) struct Arguments {
    /// Names the paths to transfer, or the web addresses a source retrieves, to add initially.
    #[arg(value_name = "SOURCE")]
    pub(crate) sources: Vec<String>,
    /// Names the final output path instead of deriving it from the media title.
    #[arg(short, long)]
    pub(crate) output: Option<PathBuf>,
    /// Disables the interactive terminal interface.
    #[arg(long)]
    pub(crate) no_tui: bool,
}

impl Arguments {
    pub(crate) fn target(&self, index: usize) -> OutputTarget {
        if self.sources.len() == 1 && index == 0 {
            self.output
                .clone()
                .map_or(OutputTarget::Title, OutputTarget::Path)
        } else {
            OutputTarget::Title
        }
    }
}

pub(crate) fn progressive_selection() -> FormatSelection {
    MediaSyntax.best_format(MediaSyntax.progressive_format())
}

/// Selects the best of whatever a source states, whether or not any of it plays.
pub(crate) fn any_selection() -> FormatSelection {
    MediaSyntax.best_format(MediaSyntax.any_format())
}
