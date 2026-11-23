use crate::arguments::Arguments;
use crate::plain;
use alux_cli::{CliCommandsAlg, cli};
use alux_ext::ext;
use rsynko_manager::SubmissionAlg;
use rsynko_media::OutputTarget;
use rsynko_memory::{MemoryManager, SourceRequest};
use rsynko_ratatui::{Application, terminal};
use std::error::Error;
use std::io::{self, IsTerminal};
use std::path::PathBuf;

/// Names the application a command line runs.
pub(crate) trait Executable {
    /// Answers the name and version of the running application.
    fn application(&self) -> Application<'static>;
}

#[ext(name = ExecutableExt, defunc)]
pub(crate) impl<This> This
where
    This: Executable,
{
    /// Transfers paths and retrieves sources, in the terminal interface or plainly.
    fn transfer(&self, sources: Vec<String>, output: Option<PathBuf>, no_tui: bool) -> Result<(), Box<dyn Error>> {
        let arguments = Arguments { sources, output, no_tui };
        if arguments.no_tui || !io::stdin().is_terminal() || !io::stdout().is_terminal() {
            return plain::run(&arguments);
        }
        if arguments.output.is_some() && arguments.sources.len() > 1 {
            return Err("--output can be used only with one source".into());
        }
        terminal::run(self.application(), requested(&arguments))
    }
}

#[ext(name = ExecutableCliExt, defunc(via = cli))]
pub(crate) impl<This> This
where
    This: CliCommandsAlg,
{
    /// Declares the command line: the sources to add, where one goes, and whether to show the interface.
    fn rsynko_cli<Machine>(&self) {
        let sources = self.positional("Paths to transfer, or web addresses a source retrieves").many();
        let output = self.named("Final output path instead of one derived from the media title").short('o');
        let no_tui = self.named("Disables the interactive terminal interface").switch();
        self.commands().cmd_root(
            self.op(Machine::transfer)
                .arg("sources", sources)
                .arg("output", output.optional())
                .arg("no_tui", no_tui)
                .fallible(),
        )
    }
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
