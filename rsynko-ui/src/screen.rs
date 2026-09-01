use crate::*;
use alux_ext::ext;
use rsynko_manager::*;

/// Specifies the presentation vocabulary every renderer supplies.
///
/// A composition states what a screen says by naming these constructors; a renderer states what
/// they look like. Nothing here names a color, a widget, a border, or a terminal.
pub trait ScreenSyntax {
    /// Represents one styled run of text.
    type Text;
    /// Represents one composed line.
    type Line;
    /// Represents one collection row, which may state several lines.
    type Row;
    /// Represents the content of one page.
    type Body;
    /// Represents one whole screen.
    type Screen;

    /// States one run of text carrying one weight.
    fn text(&self, content: impl Into<String>, emphasis: Emphasis) -> Self::Text;

    /// States one completed share as a bar of fixed width.
    fn gauge(&self, gauge: Gauge) -> Self::Text;

    /// Composes runs into one line.
    fn line(&self, runs: impl Iterator<Item = Self::Text>) -> Self::Line;

    /// States one line whose text is being edited in place, with its cursor.
    fn edited_line(&self, prefix: impl Into<String>, text: &str, cursor: usize, emphasis: Emphasis) -> Self::Line;

    /// Composes lines into one row.
    fn row(&self, lines: impl Iterator<Item = Self::Line>) -> Self::Row;

    /// States a named, ordered collection of rows, and which one the cursor rests on.
    ///
    /// A renderer with less room than the rows need keeps the one the cursor rests on in view,
    /// which it can only do by being told which that is.
    fn rows(
        &self,
        title: impl Into<String>,
        focused: Option<usize>,
        rows: impl Iterator<Item = Self::Row>,
    ) -> Self::Body;

    /// States one named message standing in for content there is none of yet.
    fn message(&self, title: impl Into<String>, message: impl Into<String>, emphasis: Emphasis) -> Self::Body;

    /// States a named text draft under edit, with its cursor, what to write in it, and the
    /// shapes of what may be written.
    ///
    /// The examples stand beside the draft rather than inside it: they are read while writing,
    /// not written over.
    fn draft(
        &self,
        title: impl Into<String>,
        placeholder: impl Into<String>,
        text: &str,
        cursor: usize,
        examples: impl Iterator<Item = (String, String)>,
    ) -> Self::Body;

    /// States a named record, quiet and read in order.
    fn record(&self, title: impl Into<String>, notes: impl Iterator<Item = String>) -> Self::Body;

    /// States one text a reader takes away, with nothing drawn around it.
    ///
    /// A reader copies this the way they copy anything else in a terminal, by selecting it, so
    /// nothing is drawn beside it that selecting would take along: no border, no label, and no
    /// break the text did not already have. What does not fit is left to the terminal to wrap.
    fn verbatim(&self, text: impl Into<String>) -> Self::Body;

    /// Composes one page from what names it, what it holds, and what can be done to it.
    fn screen(
        &self,
        header: Self::Line,
        body: Self::Body,
        status: impl Into<String>,
        footer: Self::Line,
    ) -> Self::Screen;

    /// Observes what one composed screen states, one entry per stated line.
    ///
    /// Every renderer can say what its screen reads as, which is what makes a presentation law
    /// checkable against the renderer that will actually draw it.
    fn screen_text(&self, screen: &Self::Screen) -> impl Iterator<Item = String>;
}

/// Names the application one screen belongs to.
///
/// A screen states what is running it, because a reader looking at one wants to know which thing
/// and which of its versions they are looking at. Which application that is belongs to whoever
/// runs the manager, not to the manager.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Application<'a> {
    /// Names the application as it names itself.
    pub name: &'a str,
    /// States which version of it is running.
    pub version: &'a str,
}

/// Names the bar width every progress gauge is drawn in.
pub const GAUGE_WIDTH: usize = 8;

/// Names the column budget a compact row gives a title.
pub const COMPACT_TITLE_COLUMNS: usize = 34;

/// Names the column budget an expanded heading gives a title.
pub const EXPANDED_TITLE_COLUMNS: usize = 46;

/// Names the column budget every field label is stated in.
pub const FIELD_LABEL_COLUMNS: usize = 11;

/// Names the column budget one field value is stated in before it continues underneath itself.
pub const FIELD_VALUE_COLUMNS: usize = 83;

/// Names the column budget one offered choice states its own name in.
pub const CHOICE_KEY_COLUMNS: usize = 15;

/// Marks the value the cursor rests on.
pub const CURSOR_MARK: &str = "▸";

/// Marks the one request whose details are open.
pub const EXPANDED_MARK: &str = "▾";

/// Separates the pages one page rests under, from the outermost inward.
pub const PATH_SEPARATOR: &str = "  ›  ";

/// States the mode a request is in while it will only say what it would do.
pub const REHEARSING_MODE: &str = "Dry run — changes nothing";

/// States the mode a request is in once the next run of it will write.
///
/// The mark asks for its emoji presentation, and that is what makes it safe to state. A warning
/// sign on its own is ambiguous — a font may draw it one column wide or two, and a renderer
/// measuring it has no way to know which — so the row it sits in may or may not fit. Asked for as
/// an emoji it is two columns wide to everyone who measures it, and every layer agrees.
pub const ARMED_MODE: &str = "\u{26A0}\u{FE0F} Real run — writes files";

/// States the shapes a source may be written in, and what each of them means.
///
/// One line names one request. A line naming two ends names a transfer between them, which is
/// how a whole transfer command reads as well.
pub const SUBMISSION_EXAMPLES: [(&str, &str); 6] = [
    ("/home/dev/photos/2026", "a folder here, into a folder of the same name"),
    ("backup@nas.local:/volume1/photos  /home/dev/photos", "a folder on another machine, into a folder here"),
    ("/home/dev/photos  backup@nas.local:/volume1/photos", "the same transfer, the other way around"),
    ("rsync -a nas.local:/srv/data /mnt/data", "a whole command, read as the two ends it names"),
    ("https://www.youtube.com/watch?v=VIDEO_ID", "a web address a source recognizes, fetched instead"),
    ("https://x.com/user/status/1234567890", "a tweet, taking the media it carries"),
];

/// Composes every manager page from what the manager states about itself.
#[ext(name = ManagerScreenExt)]
pub impl<This> This
where
    This: NavigationStateAlg
        + QueueCatalogAlg
        + DetailSelectionAlg
        + ManagerStatusAlg
        + TextEditorStateAlg
        + DraftStateAlg
        + OutputDraftAlg
        + InputDraftAlg,
    This::Entry: QueueEntryAlg
        + RequestOptionsAlg
        + TransferViewAlg
        + FormatChoiceViewAlg<Format: FormatDescriptionAlg>
        + RehearsalViewAlg<Change: PlannedChangeAlg>,
    This::Id: Copy + Eq,
{
    /// States the whole screen the current page denotes.
    fn screen<Syntax>(&self, syntax: &Syntax, application: Application<'_>) -> Syntax::Screen
    where
        Syntax: ScreenSyntax,
    {
        let header = self.header_line(syntax, application);
        let footer = self.footer_line(syntax);
        let status = self.manager_message().unwrap_or("Actions").to_owned();
        syntax.screen(header, self.body(syntax), status, footer)
    }

    /// States what the current page belongs to and the path that reached it.
    fn header_line<Syntax>(&self, syntax: &Syntax, application: Application<'_>) -> Syntax::Line
    where
        Syntax: ScreenSyntax,
    {
        // Every page rests under the collection, and the collection names itself: repeating it
        // here would say the same thing twice on every screen and add nothing on any.
        let path = self
            .breadcrumbs()
            .into_iter()
            .skip(1)
            .map(|breadcrumb| breadcrumb.label)
            .collect::<Vec<_>>()
            .join(PATH_SEPARATOR);
        syntax.line(
            [
                syntax.text(format!(" {} ", application.name), Emphasis::Name),
                syntax.text(format!(" {} ", application.version), Emphasis::Muted),
                syntax.text(format!(" {path}"), Emphasis::Plain),
            ]
            .into_iter(),
        )
    }

    /// States every menu entry the page offers, the unavailable ones stated quietly.
    fn footer_line<Syntax>(&self, syntax: &Syntax) -> Syntax::Line
    where
        Syntax: ScreenSyntax,
    {
        let runs = self
            .menu_items()
            .enumerate()
            .flat_map(|(index, item)| {
                let emphasis = match item.availability {
                    ActionAvailability::Enabled => Emphasis::Plain,
                    ActionAvailability::Disabled => Emphasis::Muted,
                };
                let separator = (index > 0).then(|| syntax.text("  ", Emphasis::Plain));
                separator.into_iter().chain([syntax.text(item.label(), emphasis)])
            })
            .collect::<Vec<_>>();
        syntax.line(runs.into_iter())
    }

    /// States what the current page holds.
    fn body<Syntax>(&self, syntax: &Syntax) -> Syntax::Body
    where
        Syntax: ScreenSyntax,
    {
        match self.page() {
            ManagerPage::Collection => self.collection_body(syntax, None),
            ManagerPage::Details(id) | ManagerPage::Output(id) | ManagerPage::Input(id) => {
                self.collection_body(syntax, Some(id))
            }
            ManagerPage::AddSources => self.draft_body(syntax),
            ManagerPage::Formats(id) => self.formats_body(syntax, id),
            ManagerPage::Log(id) => self.record_body(syntax, id),
            ManagerPage::Report(id) => self.report_body(syntax, id),
            ManagerPage::Command(id) => self.command_body(syntax, id),
        }
    }

    /// States the collection, one row per request, at most one of them expanded.
    fn collection_body<Syntax>(&self, syntax: &Syntax, expanded: Option<This::Id>) -> Syntax::Body
    where
        Syntax: ScreenSyntax,
    {
        let ids = self.queue_ids().collect::<Vec<_>>();
        let title = format!("{COLLECTION} ({})", ids.len());
        if ids.is_empty() {
            let add = self
                .action_keys(ManagerAction::AddSources)
                .next()
                .map_or_else(|| UNSTATED.to_owned(), Keystroke::label);
            let empty = syntax.row(
                [syntax.line(
                    [syntax.text(format!("  No sources. Press [{add}] to add or paste sources."), Emphasis::Plain)]
                        .into_iter(),
                )]
                .into_iter(),
            );
            return syntax.rows(title, None, [empty].into_iter());
        }
        let focused = self.selected_queue_id().and_then(|selected| ids.iter().position(|id| *id == selected));
        let rows = ids
            .into_iter()
            .filter_map(|id| {
                let entry = self.queue_entry(id)?;
                Some(if expanded == Some(id) {
                    self.expanded_row(syntax, id, entry)
                } else {
                    self.compact_row(syntax, id, entry)
                })
            })
            .collect::<Vec<_>>();
        syntax.rows(title, focused, rows.into_iter())
    }

    /// States one request in one line: how far it has come, what it is, and where it stands.
    fn compact_row<Syntax>(&self, syntax: &Syntax, id: This::Id, entry: &This::Entry) -> Syntax::Row
    where
        Syntax: ScreenSyntax,
    {
        let selected = self.selected_queue_id() == Some(id);
        let emphasis = if selected { Emphasis::Selected } else { entry.phase().phase_emphasis() };
        let marker = if selected { "▸" } else { " " };
        let tail = format!(
            " {:>3}% {:<width$} {:<12} {}",
            entry.percent().unwrap_or(0),
            elided(entry.label(), COMPACT_TITLE_COLUMNS),
            entry.phase().phase_label(),
            self.transfer_summary_text(entry),
            width = COMPACT_TITLE_COLUMNS,
        );
        syntax.row(
            [syntax.line(
                [
                    syntax.text(format!("{marker} {} ", entry.phase().phase_marker()), emphasis),
                    syntax.gauge(Gauge::of(entry.percent().unwrap_or(0), GAUGE_WIDTH)),
                    syntax.text(tail, emphasis),
                ]
                .into_iter(),
            )]
            .into_iter(),
        )
    }

    /// States everything one request holds, and everything that can be done to it.
    fn expanded_row<Syntax>(&self, syntax: &Syntax, id: This::Id, entry: &This::Entry) -> Syntax::Row
    where
        Syntax: ScreenSyntax,
    {
        let selected = self.selected_queue_id() == Some(id);
        let control = self.selected_detail_control();
        let heading = if selected && control.is_none() { Emphasis::Selected } else { Emphasis::Heading };
        let percent = entry.percent().unwrap_or(0);
        let head = syntax.line(
            [
                syntax.text(
                    format!("{} {} ", if selected { EXPANDED_MARK } else { " " }, entry.phase().phase_marker()),
                    heading,
                ),
                syntax.gauge(Gauge::of(percent, GAUGE_WIDTH)),
                syntax.text(
                    format!(
                        " {percent:>3}% {}  {}",
                        elided(entry.label(), EXPANDED_TITLE_COLUMNS),
                        entry.phase().phase_label()
                    ),
                    heading,
                ),
            ]
            .into_iter(),
        );
        let offered = entry.detail_controls();
        let mut lines = vec![head, self.input_line(syntax, id, entry), self.output_line(syntax, id, entry)];
        // What the choice is called depends on what is being chosen: one representation of a
        // media item, or one way of transferring a folder.
        if entry.performer() == Performer::Retrieval || entry.chosen_choice().is_some() {
            lines.push(choice_line(
                syntax,
                DetailControl::Format,
                choice_label(entry.performer()),
                self.format_choice_parts(entry),
                control,
            ));
        }
        // A request performed by naming a program shows the command Space would run, so the
        // fields above are read as what builds it. It is long, and the row states as much of it
        // as a column holds; activating the row states the whole of it, on its own.
        if let Some(command) = entry.stated_command() {
            lines.push(control_line(
                syntax,
                DetailControl::Command,
                DetailControl::Command.control_label(),
                elided(&command, FIELD_VALUE_COLUMNS),
                control,
            ));
        }
        // What the request is comes first, then how far along it is: the fields above build the
        // command, and the fields below watch it run.
        lines.extend([
            field_line(syntax, "State", entry.phase().phase_label()),
            field_line(syntax, "Downloaded", progress_text(entry)),
            field_line(syntax, "Speed", speed_text(entry)),
            field_line(syntax, "Elapsed", duration_label(entry.transfer_elapsed())),
            field_line(syntax, "Estimated", estimate_text(entry)),
        ]);
        if let Some(failure) = entry.transfer_summary() {
            lines.push(failure_line(syntax, failure));
        } else if let Some(detail) = entry.transfer_detail() {
            lines.push(field_line(syntax, "Note", detail));
        }
        if offered.contains(&DetailControl::Report) {
            lines.push(control_line(
                syntax,
                DetailControl::Report,
                DetailControl::Report.control_label(),
                report_summary(entry),
                control,
            ));
        }
        // The record is a field stating its most recent note; activating it opens the whole record.
        let latest = entry.download_log().last().unwrap_or_default();
        lines.push(control_line(
            syntax,
            DetailControl::Log,
            DetailControl::Log.control_label(),
            latest.trim().to_owned(),
            control,
        ));
        let dry_run = entry.dry_run();
        lines.extend(
            offered
                .into_iter()
                .filter(|offered| !offered.states_value())
                .map(|offered| action_line(syntax, offered, control, dry_run)),
        );
        syntax.row(lines.into_iter())
    }

    /// States where the request comes from, or the draft renaming that.
    fn input_line<Syntax>(&self, syntax: &Syntax, id: This::Id, entry: &This::Entry) -> Syntax::Line
    where
        Syntax: ScreenSyntax,
    {
        let label = end_labels(entry.output_naming()).0;
        if self.page() == ManagerPage::Input(id)
            && let Some((text, cursor)) = self.active_text_editor()
        {
            return syntax.edited_line(field_prefix(true, label), text, cursor, Emphasis::Selected);
        }
        // An input nothing may change is stated, not offered.
        if entry.detail_controls().contains(&DetailControl::Input) {
            return control_line(syntax, DetailControl::Input, label, entry.source(), self.selected_detail_control());
        }
        field_line(syntax, label, entry.source())
    }

    /// States where the request will be published, or the draft renaming it.
    fn output_line<Syntax>(&self, syntax: &Syntax, id: This::Id, entry: &This::Entry) -> Syntax::Line
    where
        Syntax: ScreenSyntax,
    {
        let control = self.selected_detail_control();
        if self.page() == ManagerPage::Output(id)
            && let Some((text, cursor)) = self.active_text_editor()
        {
            return syntax.edited_line(
                field_prefix(true, end_labels(entry.output_naming()).1),
                text,
                cursor,
                Emphasis::Selected,
            );
        }
        let output = entry
            .transfer_destination()
            .or_else(|| entry.output())
            .map_or_else(|| "derived from media ID".to_owned(), |path| path.display().to_string());
        control_line(syntax, DetailControl::Output, end_labels(entry.output_naming()).1, output, control)
    }

    /// States what the request chose: what it is called, and what choosing it does.
    ///
    /// A choice a reader would not recognize by name says what it does, and says it right after
    /// its name: the two are one phrase, and a column between them would break it in half.
    fn format_choice_parts(&self, entry: &This::Entry) -> (String, String) {
        let Some(fixed) = entry.chosen_choice() else {
            return (
                entry.media_streams().map_or_else(
                    || "the folder as it stands".to_owned(),
                    |streams| format!("Best matching {}", streams.streams_label()),
                ),
                String::new(),
            );
        };
        entry.described_formats().find(|format| format.format_identity() == fixed).map_or_else(
            || (fixed.to_owned(), entry.choice_summary(fixed).unwrap_or_default().to_owned()),
            |format| (format.format_label(), String::new()),
        )
    }

    /// States the whole command one request would run, and nothing else.
    fn command_body<Syntax>(&self, syntax: &Syntax, id: This::Id) -> Syntax::Body
    where
        Syntax: ScreenSyntax,
    {
        self.queue_entry(id).and_then(QueueEntryAlg::stated_command).map_or_else(
            || syntax.message("Command", "This request runs no command", Emphasis::Muted),
            |command| syntax.verbatim(command),
        )
    }

    /// States what a transfer has moved and how quickly, in one phrase.
    fn transfer_summary_text(&self, entry: &This::Entry) -> String {
        let speed = speed_text(entry);
        if speed == UNSTATED { progress_text(entry) } else { format!("{}  {speed}", progress_text(entry)) }
    }

    /// States the choices one request offers: the roles first, then everything discovered.
    fn formats_body<Syntax>(&self, syntax: &Syntax, id: This::Id) -> Syntax::Body
    where
        Syntax: ScreenSyntax,
    {
        let Some(entry) = self.queue_entry(id) else {
            return syntax.message("Formats", "Queue entry no longer exists", Emphasis::Muted);
        };
        // A folder offers every way it may be transferred, and never waits to find out what they
        // are: only a media item has a catalog to discover.
        let discovered = entry.performer() == Performer::Retrieval;
        match entry.discovery() {
            DiscoveryState::Unrequested | DiscoveryState::Waiting if discovered => {
                return syntax.message("Formats", "Waiting to inspect source formats…", Emphasis::Plain);
            }
            DiscoveryState::Inspecting if discovered => {
                return syntax.message("Formats", "Inspecting source formats…", Emphasis::Plain);
            }
            DiscoveryState::Failed(message) if discovered => {
                return syntax.message("Format inspection failed", message, Emphasis::Failed);
            }
            DiscoveryState::Unrequested
            | DiscoveryState::Waiting
            | DiscoveryState::Inspecting
            | DiscoveryState::Failed(_)
            | DiscoveryState::Described => {}
        }
        // A media item offers its preferred roles before every representation discovered for it;
        // a folder offers only the ways it may be transferred.
        let offered = if discovered {
            entry
                .offered_streams()
                .into_iter()
                .map(|streams| format!("Best {}", streams.streams_label()))
                .chain(entry.described_formats().map(FormatLabelExt::format_label))
                .collect::<Vec<_>>()
        } else {
            entry.selectable_choices().map(|choice| choice_text(entry, choice)).collect::<Vec<_>>()
        };
        let chosen = self.chosen_choice_row(entry, discovered);
        let rows = offered
            .into_iter()
            .enumerate()
            .map(|(index, label)| {
                let selected = index == chosen;
                let marker = if selected { format!("{CURSOR_MARK} ") } else { "  ".to_owned() };
                let emphasis = if selected { Emphasis::Selected } else { Emphasis::Plain };
                syntax.row([syntax.line([syntax.text(format!("{marker}{label}"), emphasis)].into_iter())].into_iter())
            })
            .collect::<Vec<_>>();

        syntax.rows(format!("{} ({})", choice_label(entry.performer()), rows.len()), Some(chosen), rows.into_iter())
    }

    /// States which offered row the request's current choice rests on.
    fn chosen_choice_row(&self, entry: &This::Entry, discovered: bool) -> usize {
        let Some(fixed) = entry.chosen_choice() else {
            return entry
                .offered_streams()
                .into_iter()
                .position(|streams| Some(streams) == entry.media_streams())
                .unwrap_or(0);
        };
        let stated = entry.selectable_choices().position(|choice| choice == fixed).unwrap_or(0);
        // A media item states its roles before its representations; a folder states no roles.
        if discovered { stated + entry.offered_streams().len() } else { stated }
    }

    /// States what the rehearsal said one request would do.
    fn report_body<Syntax>(&self, syntax: &Syntax, id: This::Id) -> Syntax::Body
    where
        Syntax: ScreenSyntax,
    {
        self.queue_entry(id).map_or_else(
            || syntax.message("Report", "Queue entry no longer exists", Emphasis::Muted),
            |entry| report_body(syntax, entry),
        )
    }

    /// States everything observed about one request, in the order it was observed.
    fn record_body<Syntax>(&self, syntax: &Syntax, id: This::Id) -> Syntax::Body
    where
        Syntax: ScreenSyntax,
    {
        let Some(entry) = self.queue_entry(id) else {
            return syntax.message("Log", "Queue entry no longer exists", Emphasis::Muted);
        };
        let notes = entry.download_log().map(str::to_owned).collect::<Vec<_>>();
        syntax.record(format!("Log ({})", notes.len()), notes.into_iter())
    }

    /// States the sources draft under edit, and the shapes a source may be written in.
    fn draft_body<Syntax>(&self, syntax: &Syntax) -> Syntax::Body
    where
        Syntax: ScreenSyntax,
    {
        let (text, cursor) = self.active_text_editor().unwrap_or(("", 0));
        syntax.draft(
            "Sources — editing",
            "Enter or paste one source per line…",
            text,
            cursor,
            SUBMISSION_EXAMPLES.into_iter().map(|(shape, means)| ((*shape).to_owned(), (*means).to_owned())),
        )
    }
}

/// States everything one transfer would do, the changes it would make before what it would leave.
fn report_body<Syntax, Entry>(syntax: &Syntax, entry: &Entry) -> Syntax::Body
where
    Syntax: ScreenSyntax,
    Entry: RehearsalViewAlg,
    Entry::Change: PlannedChangeAlg,
{
    match entry.rehearsal() {
        RehearsalState::Unrehearsed => {
            return syntax.message("Report", "This request has not been rehearsed yet.", Emphasis::Plain);
        }
        RehearsalState::Rehearsing => {
            return syntax.message("Report", "Stating what the transfer would do…", Emphasis::Plain);
        }
        RehearsalState::Failed(message) => {
            return syntax.message("Rehearsal failed", message, Emphasis::Failed);
        }
        RehearsalState::Reported => {}
    }
    let mut changes = entry.planned_changes().collect::<Vec<_>>();
    // What the transfer would alter is read first; what it would leave stands under it.
    changes.sort_by_key(|change| usize::from(!change.change_kind().alters()));
    let altered = changes.iter().filter(|change| change.change_kind().alters()).count();
    let rows = changes
        .iter()
        .map(|change| {
            let kind = change.change_kind();
            syntax.row(
                [syntax.line(
                    [syntax.text(
                        format!(
                            "  {} {:<9} {:>10}  {}",
                            kind.change_marker(),
                            kind.change_label(),
                            change.change_size().map_or_else(|| UNSTATED.to_owned(), bytes_label),
                            change.change_path()
                        ),
                        kind.change_emphasis(),
                    )]
                    .into_iter(),
                )]
                .into_iter(),
            )
        })
        .collect::<Vec<_>>();
    syntax.rows(format!("Report ({altered} of {} would change)", changes.len()), None, rows.into_iter())
}

/// States one field naming a value nothing can be done to.
fn field_line<Syntax>(syntax: &Syntax, label: &str, value: impl Into<String>) -> Syntax::Line
where
    Syntax: ScreenSyntax,
{
    syntax.line(
        [
            syntax.text(format!("    {label:<FIELD_LABEL_COLUMNS$}"), Emphasis::Label),
            syntax.text(value, Emphasis::Plain),
        ]
        .into_iter(),
    )
}

/// States why a transfer failed.
fn failure_line<Syntax>(syntax: &Syntax, failure: &str) -> Syntax::Line
where
    Syntax: ScreenSyntax,
{
    syntax.line(
        [
            syntax.text(format!("    {:<FIELD_LABEL_COLUMNS$}", "Error"), Emphasis::Failed),
            syntax.text(failure, Emphasis::Failed),
        ]
        .into_iter(),
    )
}

/// States one field a cursor can rest on, naming the value it holds.
///
/// A value that can no longer be changed is marked by nothing at all: the cursor cannot reach it,
/// and a field the cursor walks past is already saying so.
fn control_line<Syntax>(
    syntax: &Syntax,
    control: DetailControl,
    label: &str,
    value: impl Into<String>,
    selected: Option<DetailControl>,
) -> Syntax::Line
where
    Syntax: ScreenSyntax,
{
    let chosen = selected == Some(control);
    let emphasis = match (chosen, control) {
        (true, _) => Emphasis::Selected,
        // The record states its latest note without competing with the values above it.
        (false, DetailControl::Log) => Emphasis::Muted,
        (false, _) => Emphasis::Plain,
    };
    syntax.line([syntax.text(format!("{}{}", field_prefix(chosen, label), value.into()), emphasis)].into_iter())
}

/// States one editable control whose value names a choice and then says what choosing it does.
///
/// The name is what a reader picks between, so it carries the weight; what it does follows it
/// directly, quietly, as the rest of the same phrase.
fn choice_line<Syntax>(
    syntax: &Syntax,
    control: DetailControl,
    label: &str,
    (choice, summary): (String, String),
    selected: Option<DetailControl>,
) -> Syntax::Line
where
    Syntax: ScreenSyntax,
{
    let chosen = selected == Some(control);
    // The cursor takes the whole row; otherwise the name labels what follows it, and what
    // follows it stays quiet.
    let (field, name, rest) = if chosen {
        (Emphasis::Selected, Emphasis::Selected, Emphasis::Selected)
    } else {
        (Emphasis::Plain, Emphasis::Label, Emphasis::Muted)
    };
    let mut runs = vec![syntax.text(field_prefix(chosen, label), field), syntax.text(choice, name)];
    if !summary.is_empty() {
        runs.push(syntax.text(format!(" {summary}"), rest));
    }
    syntax.line(runs.into_iter())
}

/// States one action a cursor can rest on.
///
/// The rehearsal row is the exception: it states which mode the request is *in* rather than what
/// pressing it does, and carries the weight of being in it. Rehearsing is safe; being armed is a
/// caution. Activating the row moves between them.
fn action_line<Syntax>(
    syntax: &Syntax,
    control: DetailControl,
    selected: Option<DetailControl>,
    dry_run: Option<bool>,
) -> Syntax::Line
where
    Syntax: ScreenSyntax,
{
    let chosen = selected == Some(control);
    let (label, consequence) = match (control, dry_run) {
        (DetailControl::DryRun, Some(true)) => (REHEARSING_MODE, Emphasis::Safe),
        (DetailControl::DryRun, Some(false) | None) => (ARMED_MODE, Emphasis::Caution),
        (control, _) => (control.control_label(), Emphasis::Plain),
    };
    let emphasis = if chosen { Emphasis::Selected } else { consequence };
    syntax.line([syntax.text(format!("  {} [{label}]", cursor_marker(chosen)), emphasis)].into_iter())
}

/// States what a rehearsal found, counted by what it would do.
fn report_summary<Entry>(entry: &Entry) -> String
where
    Entry: RehearsalViewAlg,
    Entry::Change: PlannedChangeAlg,
{
    match entry.rehearsal() {
        RehearsalState::Unrehearsed => "not rehearsed yet".to_owned(),
        RehearsalState::Rehearsing => "stating what would happen…".to_owned(),
        RehearsalState::Failed(message) => format!("rehearsal failed: {message}"),
        RehearsalState::Reported => {
            let counted = ChangeKind::REPORTED
                .into_iter()
                .filter_map(|kind| {
                    let count = entry.planned_changes().filter(|change| change.change_kind() == kind).count();
                    (count > 0).then(|| format!("{count} {}", kind.change_label()))
                })
                .collect::<Vec<_>>();
            if counted.is_empty() { "nothing to transfer".to_owned() } else { counted.join(", ") }
        }
    }
}

/// States the cursor and the label one selectable field begins with.
fn field_prefix(chosen: bool, label: &str) -> String {
    format!("  {} {label:<FIELD_LABEL_COLUMNS$}", cursor_marker(chosen))
}

/// Names the two ends of one request the way a reader reads them.
///
/// A media item is fetched from a source and written to a file this application names; a transfer
/// goes from an input to an output, both of them stated by whoever asked for it.
const fn end_labels(naming: OutputNaming) -> (&'static str, &'static str) {
    match naming {
        OutputNaming::Portable => ("Source", "File name"),
        OutputNaming::Stated => ("Input", "Output"),
    }
}

/// Names what a request chooses between, which is not the same thing for every request.
const fn choice_label(performer: Performer) -> &'static str {
    match performer {
        Performer::Retrieval => "Format",
        Performer::Program => "Transfer",
    }
}

/// States one offered choice in aligned columns, so alternatives are compared by reading down.
fn choice_text<Entry>(entry: &Entry, choice: &str) -> String
where
    Entry: RequestOptionsAlg,
{
    entry
        .choice_summary(choice)
        .map_or_else(|| choice.to_owned(), |summary| format!("{choice:<CHOICE_KEY_COLUMNS$} {summary}"))
}

/// States whether the cursor rests here.
///
/// The mark is a small triangle rather than a large one: the large triangles carry an emoji
/// presentation, which a reader's font may draw two columns wide and shift the row it marks.
const fn cursor_marker(chosen: bool) -> &'static str {
    if chosen { "▸" } else { " " }
}

/// States how much of the transfer has arrived.
fn progress_text<Entry>(entry: &Entry) -> String
where
    Entry: TransferViewAlg,
{
    match entry.transfer_total() {
        Some(total) => format!("{} / {}", bytes_label(entry.transferred()), bytes_label(total)),
        None if entry.transferred() > 0 => {
            format!("{} downloaded", bytes_label(entry.transferred()))
        }
        None => UNSTATED.to_owned(),
    }
}

/// States how quickly the transfer is arriving.
fn speed_text<Entry>(entry: &Entry) -> String
where
    Entry: TransferViewAlg,
{
    entry.bytes_per_second().map_or_else(|| UNSTATED.to_owned(), |speed| format!("{}/s", bytes_label(speed)))
}

/// States how long the transfer still needs.
fn estimate_text<Entry>(entry: &Entry) -> String
where
    Entry: TransferViewAlg,
{
    entry.estimated_remaining().map_or_else(|| UNSTATED.to_owned(), duration_label)
}
