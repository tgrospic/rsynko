#![doc = include_str!("../README.md")]

mod arguments;
mod plain;
mod program;

use crate::program::{Executable, ExecutableCliExt};
use alux_cli::CliProgramBuilder;
use alux_cli_clap::CliClap;
use rsynko_ratatui::Application;
use std::sync::Arc;

/// Names this application and the version of it that is running.
const RSYNKO: Application<'static> = Application { name: env!("CARGO_BIN_NAME"), version: env!("CARGO_PKG_VERSION") };

/// Answers the running application to the command line.
struct Rsynko;

impl Executable for Rsynko {
    fn application(&self) -> Application<'static> {
        RSYNKO
    }
}

/// Reads what was asked for, and states it to whichever interpreter can be read.
fn main() {
    CliClap::new(Arc::new(Rsynko), CliProgramBuilder.rsynko_cli::<Rsynko>())
        .named(RSYNKO.name)
        .version(RSYNKO.version)
        .run_blocking(std::env::args_os())
        .exit()
}
