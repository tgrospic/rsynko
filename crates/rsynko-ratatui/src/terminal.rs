use crate::downloads::Downloads;
use crate::inspections::Inspections;
use crate::screen::{RatatuiScreen, paint};
use crate::transfers::Transfers;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use rsynko_manager::*;
use rsynko_memory::{ManagerState, SourceRequest};
use rsynko_session::SessionExt;
use rsynko_ui::*;
use std::error::Error;
use std::io::{self, stdout};
use std::time::{Duration, Instant};

/// States how long the reader waits for a key before the screen is painted again.
const FRAME_INTERVAL: Duration = Duration::from_millis(50);

/// Names the keystroke people press out of habit to stop a terminal program, and what it answers.
///
/// The terminal reports `Ctrl+C` as an ordinary key, so nothing about it ends this program. It is
/// bound to nothing, which would leave whoever pressed it wondering; instead it says what it is
/// for, because the one thing it must not do is end a session somebody is in the middle of.
const GUARD_KEYSTROKE: Keystroke = Keystroke::control(Key::Character('c'));

/// States what one accidental interruption is answered with.
const GUARD_MESSAGE: &str = "Press Ctrl+C twice to exit";

/// States how close together the two have to be to be one thing a reader did on purpose.
const DOUBLE_INTERRUPTION: Duration = Duration::from_millis(150);

/// States how long a transfer told to stop is given to stop before the reader leaves without it.
const LEAVING_GRACE: Duration = Duration::from_secs(2);

/// Holds the terminal in the state a full-screen reader needs, and gives it back afterward.
struct TerminalSession;

impl TerminalSession {
    /// Takes the terminal over for as long as this value is held.
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        if let Err(error) = execute!(stdout(), EnterAlternateScreen) {
            let _raw_mode = disable_raw_mode();
            return Err(error);
        }
        Ok(Self)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _raw_mode = disable_raw_mode();
        let _screen = execute!(stdout(), LeaveAlternateScreen);
    }
}

/// Takes the terminal over, attends to everything running, and gives it back afterward.
///
/// # Errors
///
/// States nothing where the terminal cannot be taken over, and where painting or reading it fails
/// while it is held.
pub fn run(application: Application<'_>, requests: Vec<SourceRequest>) -> Result<(), Box<dyn Error>> {
    let mut manager = ManagerState::downloads();
    manager.apply_manager_event(ManagerIntentOp::AddSources { requests });

    let session = TerminalSession::enter()?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    let result = run_manager(&mut terminal, application, &mut manager);
    terminal.show_cursor()?;
    drop(terminal);
    drop(session);
    result.map_err(|message| io::Error::other(message).into())
}

/// Attends to everything running, paints what the manager states, and reads what the reader does.
fn run_manager(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    application: Application<'_>,
    manager: &mut ManagerState,
) -> Result<(), String> {
    let mut downloads = Vec::new();
    let mut transfers = Vec::new();
    let mut inspections = Vec::new();
    let mut leaving = None;
    let mut interrupted = None;
    loop {
        Downloads::attending(manager).attend(&mut downloads);
        Transfers::attending(manager).attend(&mut transfers);
        Inspections::attending(manager).attend(&mut inspections);
        // The question lasts exactly as long as the answer would count, and then it is over: what
        // was asked is unasked, and the screen stops saying it.
        if interrupted.is_some() && !interrupted_twice(interrupted, Instant::now()) {
            interrupted = None;
            manager.set_manager_message(None);
        }
        // Leaving is wanting nothing, so the ordinary pass ends every run: a transfer is told to
        // stop, and a download is asked to. One that has not started yet cannot be told, so it is
        // told again every pass, for as long as that is worth keeping the reader here.
        if manager.exit_requested() {
            let asked = *leaving.get_or_insert_with(Instant::now);
            let running = !transfers.is_empty() || !downloads.is_empty();
            if !running || asked.elapsed() > LEAVING_GRACE {
                return Ok(());
            }
        }
        terminal.draw(|frame| draw(frame, application, manager)).map_err(|error| error.to_string())?;
        if event::poll(FRAME_INTERVAL).map_err(|error| error.to_string())? {
            match event::read().map_err(|error| error.to_string())? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match interruption(key) {
                    Interruption::Guarded => {
                        if interrupted_twice(interrupted, Instant::now()) {
                            manager.apply_manager_event(ManagerIntentOp::SafeExitRequested {});
                        } else {
                            interrupted = Some(Instant::now());
                            manager.set_manager_message(Some(GUARD_MESSAGE.to_owned()));
                        }
                    }
                    Interruption::Ordinary => {
                        // Anything else says the reader is still here, so the question is over.
                        if interrupted.take().is_some() {
                            manager.set_manager_message(None);
                        }
                        handle_key(manager, key);
                    }
                },
                Event::Paste(text) => handle_paste(manager, &text),
                _ => {}
            }
        }
    }
}

/// Draws the screen the manager currently denotes.
fn draw(frame: &mut ratatui::Frame<'_>, application: Application<'_>, manager: &ManagerState) {
    let screen = manager.screen(&RatatuiScreen, application);
    paint(frame, &screen, frame.area());
}

/// States what one key press is, where one of them is a question rather than a meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Interruption {
    /// The keystroke that asks whether the reader meant to leave.
    Guarded,
    /// Anything the manager binds, or does not.
    Ordinary,
}

/// States whether the second interruption came close enough to the first to be one thing.
///
/// Two of them a second apart are two accidents; two of them together are a reader saying it twice
/// on purpose, which is the only way this key ends anything.
fn interrupted_twice(previous: Option<Instant>, now: Instant) -> bool {
    previous.is_some_and(|first| now.duration_since(first) <= DOUBLE_INTERRUPTION)
}

/// Reads one key press as the interruption it is.
fn interruption(key: KeyEvent) -> Interruption {
    if keystroke(key) == Some(GUARD_KEYSTROKE) { Interruption::Guarded } else { Interruption::Ordinary }
}

/// Applies what one reported key press denotes, when the terminal reports a key the manager names.
fn handle_key(manager: &mut ManagerState, key: KeyEvent) {
    if let Some(stroke) = keystroke(key) {
        manager.apply_keystroke(stroke);
    }
}

/// Applies what pasted text denotes.
fn handle_paste(manager: &mut ManagerState, text: &str) {
    manager.apply_paste(text);
}

/// Reads one reported key press as the keystroke the manager binds.
fn keystroke(event: KeyEvent) -> Option<Keystroke> {
    let key = match event.code {
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Enter => Key::Enter,
        KeyCode::Esc => Key::Escape,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Delete => Key::Delete,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::Char(character) => Key::Character(character),
        _ => return None,
    };
    Some(Keystroke {
        key,
        control: event.modifiers.contains(KeyModifiers::CONTROL),
        alternate: event.modifiers.contains(KeyModifiers::ALT),
    })
}

#[cfg(test)]
mod tests {
    /// Names the application these tests state screens for.
    const TESTING: Application<'static> = Application { name: "rsynko", version: "0.0.0" };

    use super::{GUARD_KEYSTROKE, Interruption, draw, handle_key, interrupted_twice, interruption, keystroke};
    use crate::downloads::Downloads;
    use crate::transfers::Transfers;
    use alux_sdk::IterTraversableExt;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use rsynko_manager::*;
    use rsynko_media::*;
    use rsynko_memory::{
        ChangeKind, DownloadOptions, Format, InfoValue, ManagerState, MediaSyntax, MemoryManager, SourceRequest,
    };
    use rsynko_reqwest::FIXTURE_BYTES;
    use rsynko_session::SessionExt;
    use rsynko_ui::*;
    use std::path::PathBuf;
    use std::thread;
    use std::time::{Duration, Instant};

    /// Reads the whole terminal back as the characters it is showing.
    fn rendered(terminal: &Terminal<TestBackend>) -> String {
        terminal.backend().buffer().content().iter().map(ratatui::buffer::Cell::symbol).collect()
    }

    /// Describes one selectable format through the observations a renderer reads.
    fn described_format<'a>(id: &str, observations: impl IntoIterator<Item = (&'a str, InfoValue)>) -> Format {
        MediaSyntax
            .format(id, MediaSyntax.metadata(observations.into_iter().map(|(key, value)| (key.to_owned(), value))))
    }

    #[test]
    fn only_two_interruptions_together_end_anything() {
        let first = Instant::now();

        // Nobody interrupted before this one.
        assert!(!interrupted_twice(None, first));
        // Twice together is a reader saying it twice.
        assert!(interrupted_twice(Some(first), first + Duration::from_millis(120)));
        // Twice apart is two accidents.
        assert!(!interrupted_twice(Some(first), first + Duration::from_millis(900)));
    }

    #[test]
    fn an_accidental_interruption_is_asked_about_rather_than_obeyed() {
        let interrupt = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let exit = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        let other = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);

        assert_eq!(interruption(interrupt), Interruption::Guarded);
        // Exit is bound, so it is ordinary: the manager states what it means.
        assert_eq!(interruption(exit), Interruption::Ordinary);
        assert_eq!(interruption(other), Interruption::Ordinary);
    }

    #[test]
    fn reported_key_presses_decode_to_the_keystrokes_the_manager_binds() {
        let reported = |code, modifiers| keystroke(KeyEvent::new(code, modifiers));
        assert_eq!(reported(KeyCode::Up, KeyModifiers::NONE), Some(Keystroke::plain(Key::Up)));
        assert_eq!(reported(KeyCode::Enter, KeyModifiers::NONE), Some(Keystroke::plain(Key::Enter)));
        assert_eq!(reported(KeyCode::Char(' '), KeyModifiers::NONE), Some(Keystroke::plain(Key::Character(' '))));
        assert_eq!(reported(KeyCode::Char('q'), KeyModifiers::CONTROL), Some(EXIT_KEYSTROKE));
        assert_eq!(reported(KeyCode::Char('c'), KeyModifiers::CONTROL), Some(GUARD_KEYSTROKE));
        // A key the vocabulary does not name is not decoded into one that happens to be near it.
        assert_eq!(reported(KeyCode::F(1), KeyModifiers::NONE), None);

        let mut manager = ManagerState::downloads();
        handle_key(&mut manager, KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL));
        assert!(manager.exit_requested());

        // The interruption is this interpreter's own question, so the manager hears nothing of it.
        let mut undisturbed = ManagerState::downloads();
        handle_key(&mut undisturbed, KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(!undisturbed.exit_requested());
    }

    #[test]
    fn collection_and_details_render_from_manager_state() {
        let backend = TestBackend::new(100, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let mut manager = ManagerState::downloads();
        manager.apply_manager_event(ManagerIntentOp::AddSources {
            requests: vec![SourceRequest::new("fixture://single-video", None, DownloadOptions::progressive())],
        });
        let selected = manager.selected_id().expect("selected fixture");
        manager.apply_manager_event(ManagerIntentOp::Transfer {
            id: selected,
            event: TransferObservationOp::Progress {
                destination: PathBuf::from("single-video.mp4"),
                downloaded: 32,
                total: Some(64),
            },
        });
        manager.apply_manager_event(ManagerIntentOp::Transfer {
            id: selected,
            event: TransferObservationOp::Elapsed { elapsed: Duration::from_secs(2) },
        });
        manager.apply_manager_event(ManagerIntentOp::SourceMetadata {
            id: selected,
            media_id: "single-video".to_owned(),
            title: Some("Fetched video title".to_owned()),
        });
        terminal.draw(|frame| draw(frame, TESTING, &manager)).expect("render collection");
        manager.apply_manager_event(ManagerIntentOp::OpenSelected {});
        terminal.draw(|frame| draw(frame, TESTING, &manager)).expect("render details");

        let shown = rendered(&terminal);
        assert!(shown.contains(COLLECTION));
        assert!(shown.contains("single-video"));
        assert!(shown.contains("Downloaded"));
        assert!(shown.contains("Speed"));
        assert!(shown.contains("Elapsed"));
        assert!(shown.contains("Estimated"));
        assert!(shown.contains("Format"));
        assert!(shown.contains("Fetched video title"));
        assert!(!shown.contains("Error"));
        // The record is a selectable field stating its most recent note, not an action.
        assert!(shown.contains("Log"));
        assert!(shown.contains("extracted single-video"));
        assert!(!shown.contains("[Log]"));
        // The bar states its filled part; the rest is the span's background, not a character.
        assert!(shown.contains("████      50%"));
        assert!(!shown.contains("[File name]"));
    }

    #[test]
    fn one_format_reads_the_same_where_it_is_chosen_and_where_it_is_shown() {
        let backend = TestBackend::new(100, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let mut manager = ManagerState::downloads();
        manager.apply_manager_event(ManagerIntentOp::AddSources {
            requests: vec![SourceRequest::new("fixture://single-video", None, DownloadOptions::progressive())],
        });
        let selected = manager.selected_id().expect("selected format fixture");
        manager.apply_manager_event(ManagerIntentOp::FormatCatalog {
            id: selected,
            event: FormatDiscoveryOp::Available {
                formats: vec![described_format(
                    "140",
                    [
                        (FORMAT_EXTENSION, MediaSyntax.string_metadata("m4a")),
                        (FORMAT_HAS_AUDIO, MediaSyntax.boolean_metadata(true)),
                        (FORMAT_HAS_VIDEO, MediaSyntax.boolean_metadata(false)),
                        ("quality", MediaSyntax.string_metadata("medium")),
                        ("bitrate", MediaSyntax.integer_metadata(128 * 1024 * 8)),
                        ("codecs", MediaSyntax.string_metadata("mp4a.40.2")),
                    ],
                )],
            },
        });
        manager.apply_manager_event(ManagerIntentOp::OpenSelected {});
        manager.apply_manager_event(ManagerIntentOp::OpenFormats {});
        terminal.draw(|frame| draw(frame, TESTING, &manager)).expect("render formats");
        let offered = rendered(&terminal);
        // One chooser states the preferred roles first and every discovered format after, and
        // offers only the roles something described carries: this source states one audio-only
        // representation, so asking it for a picture is not among the things on offer.
        assert!(offered.contains("Best audio only"));
        assert!(!offered.contains("Best audio + video"));
        assert!(!offered.contains("Best video only"));
        let label = "140  m4a   audio only    medium  128.0 KiB/s  mp4a.40.2";
        assert!(offered.contains(label), "{offered}");

        // Choosing it and going back states the same columns, painted by the same words.
        while manager.selected().and_then(RequestOptionsAlg::chosen_choice) != Some("140") {
            manager.apply_manager_event(ManagerIntentOp::SelectNextFormat {});
        }
        manager.apply_manager_event(ManagerIntentOp::Back {});
        terminal.draw(|frame| draw(frame, TESTING, &manager)).expect("render details");
        assert!(rendered(&terminal).contains(label));
    }

    #[test]
    fn distinct_fixture_outputs_run_as_independent_downloads() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let outputs = [directory.path().join("one.mp4"), directory.path().join("two.mp4")];
        let mut manager = ManagerState::downloads();
        manager.apply_manager_event(ManagerIntentOp::AddSources {
            requests: outputs
                .iter()
                .map(|output| {
                    SourceRequest::new("fixture://single-video", Some(output.clone()), DownloadOptions::progressive())
                })
                .collect(),
        });
        manager.apply_manager_event(ManagerIntentOp::ApplySelectedSpace {});
        manager.apply_manager_event(ManagerIntentOp::SelectNext {});
        manager.apply_manager_event(ManagerIntentOp::ApplySelectedSpace {});

        let mut running = Vec::new();
        Downloads::attending(&mut manager).attend(&mut running);
        // One pass begins every request that wants work, rather than one of them.
        assert_eq!(running.len(), 2);

        let deadline = Instant::now() + Duration::from_secs(5);
        while !running.is_empty() && Instant::now() < deadline {
            Downloads::attending(&mut manager).attend(&mut running);
            thread::yield_now();
        }

        assert!(running.is_empty());
        assert!(manager.queue().iter().all(|entry| entry.transfer().phase() == TransferPhase::Complete));
        let contents = outputs.iter().traverse(std::fs::read).expect("fixture outputs");
        assert!(contents.iter().all(|bytes| bytes == FIXTURE_BYTES));
        assert!(outputs.iter().all(|output| !PathBuf::from(format!("{}.part", output.display())).exists()));
    }

    /// Observes whether the machine running the test has the transfer program.
    fn transfers_available() -> bool {
        std::process::Command::new(rsynko_rsync::SYNC_PROGRAM)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }

    #[test]
    fn a_rehearsal_run_by_the_manager_reports_what_it_would_remove() {
        if !transfers_available() {
            return;
        }
        let root = tempfile::tempdir().expect("a place to author folders");
        let source = root.path().join("source");
        let destination = root.path().join("destination");
        std::fs::create_dir_all(&source).expect("a source");
        std::fs::create_dir_all(&destination).expect("a destination");
        std::fs::write(source.join("kept.txt"), b"from the source").expect("a kept file");
        std::fs::write(destination.join("extra.txt"), b"nobody asked").expect("an extra file");

        let mut manager = ManagerState::downloads();
        manager.apply_manager_event(ManagerIntentOp::AddSources {
            requests: vec![MemoryManager.submitted(&format!("{}/ {}", source.display(), destination.display()))],
        });
        let id = manager.selected_id().expect("the submitted transfer");
        // Mirroring is what removes, so the way of transferring is walked onto it.
        manager.apply_manager_event(ManagerIntentOp::OpenSelected {});
        manager.apply_manager_event(ManagerIntentOp::OpenFormats {});
        manager.apply_manager_event(ManagerIntentOp::SelectNextFormat {});
        manager.apply_manager_event(ManagerIntentOp::Back {});
        assert_eq!(manager.entry(id).and_then(RequestOptionsAlg::chosen_choice), Some("mirror"));
        manager.apply_manager_event(ManagerIntentOp::ApplySelectedSpace {});

        let mut running = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            Transfers::attending(&mut manager).attend(&mut running);
            if running.is_empty() || Instant::now() >= deadline {
                break;
            }
            thread::yield_now();
        }

        let entry = manager.entry(id).expect("the rehearsed transfer");
        let removed = entry
            .planned_changes()
            .filter(|change| change.change_kind() == ChangeKind::Delete)
            .map(|change| change.change_path().to_owned())
            .collect::<Vec<_>>();
        assert_eq!(removed, vec!["extra.txt".to_owned()], "{:?}", entry.rehearsal());
        // Nothing was moved: a rehearsal states what it would do.
        assert!(destination.join("extra.txt").exists());
        assert!(!destination.join("kept.txt").exists());
    }
}
