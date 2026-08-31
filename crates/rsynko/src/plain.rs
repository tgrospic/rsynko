use crate::arguments::{Arguments, any_selection, progressive_selection};
use rsynko_manager::*;
use rsynko_media::{ApplicationError, ApplicationExt, MediaDownloadError, OutputTarget};
use rsynko_memory::{DownloadEvent, ManagerState, MemoryManager, PlannedChange, SourceRequest};
use rsynko_process::ProcessSyncEnv;
use rsynko_reqwest::RuntimeEnvironment;
use rsynko_rsync::SyncProgramExt;
use rsynko_yt::{YoutubeApplicationExt, media_failure, youtube_id};
use std::error::Error;
use std::sync::mpsc::channel;

/// States every source as lines, and ends when the last one has.
///
/// # Errors
///
/// States nothing where no source was named, where one output was named for several sources, and
/// where a source refused to be read or to come to rest.
pub(crate) fn run(arguments: &Arguments) -> Result<(), Box<dyn Error>> {
    if arguments.sources.is_empty() {
        return Err("at least one source is required with --no-tui or redirected output".into());
    }
    if arguments.output.is_some() && arguments.sources.len() > 1 {
        return Err("--output can be used only with one source".into());
    }
    for (index, source) in arguments.sources.iter().enumerate() {
        let request = submitted(source, arguments.target(index));
        // A path is performed rather than rehearsed here: a line-oriented run was told exactly
        // what to do, and the reader who would read a report is not watching.
        if request.options.performer() == Performer::Program {
            transfer(request)?;
        } else {
            retrieve(source, &arguments.target(index))?;
        }
    }
    Ok(())
}

/// States the request one argument names, with an explicit output overriding what it named.
fn submitted(source: &str, target: OutputTarget) -> SourceRequest {
    let mut request = MemoryManager.submitted(source);
    if let OutputTarget::Path(path) = target {
        request.output = Some(path);
    }
    request
}

/// Performs one path transfer and states every path it changed.
fn transfer(request: SourceRequest) -> Result<(), Box<dyn Error>> {
    // The command is the one the request states, and it is derived exactly once, so what a
    // reader is shown anywhere is what runs everywhere.
    let mut manager = ManagerState::downloads();
    manager.apply_manager_event(ManagerIntentOp::AddSources {
        requests: vec![request],
    });
    let id = manager
        .selected_id()
        .ok_or("the submitted transfer was not collected")?;
    manager.set_queue_dry_run(id, false);
    let command = manager
        .entry(id)
        .and_then(rsynko_memory::QueueEntry::transfer_command)
        .ok_or("the transfer states no command to run")?;
    let (sender, _watched) = channel();
    let changes = ProcessSyncEnv::new(sender).run_sync(&command)?;
    for change in &changes {
        render_change(change);
    }
    Ok(())
}

/// Retrieves one web address the way the source that claimed it retrieves.
fn retrieve(source: &str, target: &OutputTarget) -> Result<(), Box<dyn Error>> {
    let environment = RuntimeEnvironment::build()?;
    let selection = progressive_selection();
    let result = if youtube_id(source).is_some() {
        environment
            .download_youtube(source, &selection, target)
            .map_err(|error| error.to_string())
    } else {
        match environment.download_url(source, &selection, target) {
            // A source may carry nothing that plays: a tweet of photographs states no streams at
            // all, and the best of what it does state is what was asked for.
            Err(ApplicationError::Media(MediaDownloadError::NoMatchingFormat)) => environment
                .download_url(source, &any_selection(), target)
                .map_err(|error| error.to_string()),
            stated => stated.map_err(|error| error.to_string()),
        }
    };
    for event in environment.events() {
        render_terminal_event(&event);
    }
    result.map_err(|detail| media_failure(&detail))?;
    Ok(())
}

/// States one path a transfer changed.
fn render_change(change: &PlannedChange) {
    let did = match change.change_kind() {
        ChangeKind::Create => "added",
        ChangeKind::Update => "replaced",
        ChangeKind::Delete => "removed",
        ChangeKind::Unchanged => return,
    };
    println!("{did} {}", change.change_path());
}

pub(crate) fn render_terminal_event(event: &DownloadEvent) {
    match event {
        DownloadEvent::Succeeded { destination, bytes } => {
            println!(
                "download succeeded: {} ({bytes} bytes)",
                destination.display()
            );
        }
        DownloadEvent::Failed {
            destination,
            message,
        } => {
            eprintln!("download failed: {}: {message}", destination.display());
        }
    }
}
