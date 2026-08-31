#![doc = include_str!("../README.md")]

mod arguments;
mod plain;

use crate::arguments::Arguments;
use clap::Parser;
use rsynko_manager::SubmissionAlg;
use rsynko_media::OutputTarget;
use rsynko_memory::{MemoryManager, SourceRequest};
use rsynko_ratatui::{Application, terminal};
use std::error::Error;
use std::io::{self, IsTerminal};

/// Names this application and the version of it that is running.
const RSYNKO: Application<'static> = Application {
    name: env!("CARGO_BIN_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

/// Reads what was asked for, and states it to whichever interpreter can be read.
fn main() -> Result<(), Box<dyn Error>> {
    let arguments = Arguments::parse();
    if arguments.no_tui || !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return plain::run(&arguments);
    }
    if arguments.output.is_some() && arguments.sources.len() > 1 {
        return Err("--output can be used only with one source".into());
    }

    terminal::run(RSYNKO, requested(&arguments))
}

/// States every source a reader named as one request, with the output they stated for it.
fn requested(arguments: &Arguments) -> Vec<SourceRequest> {
    arguments
        .sources
        .iter()
        .enumerate()
        .map(|(index, source)| {
            let output = match arguments.target(index) {
                OutputTarget::Path(path) => Some(path),
                OutputTarget::MediaId | OutputTarget::Title => None,
            };
            // A submitted line names what it names; an explicit output overrides what it named.
            let mut request = MemoryManager.submitted(source);
            if let Some(output) = output {
                request.output = Some(output);
            }
            request
        })
        .collect()
}
