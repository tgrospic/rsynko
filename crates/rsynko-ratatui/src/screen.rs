use ratatui::Frame;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use rsynko_ui::{Emphasis, Gauge, ScreenSyntax, elided};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Draws screens with Ratatui widgets in a terminal.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct RatatuiScreen;

/// Holds one composed line and the in-place edit it may carry.
///
/// An edited line is windowed while it is drawn, because only then is the column budget known.
pub(crate) struct RenderedLine {
    spans: Vec<Span<'static>>,
    edit: Option<Edit>,
}

/// Holds text under edit until the width it is drawn in is known.
struct Edit {
    prefix: String,
    text: String,
    cursor: usize,
    style: Style,
}

/// Holds the content of one page.
pub(crate) enum RenderedBody {
    /// Holds a named collection of rows, and which one the cursor rests on.
    Rows {
        title: String,
        focused: Option<usize>,
        rows: Vec<Vec<RenderedLine>>,
    },
    /// Holds one message standing in for content there is none of yet.
    Message {
        title: String,
        message: String,
        style: Style,
    },
    /// Holds a text draft under edit, and the shapes of what may be written in it.
    Draft {
        title: String,
        placeholder: String,
        text: String,
        cursor: usize,
        examples: Vec<(String, String)>,
    },
    /// Holds a record read in order.
    Record { title: String, notes: Vec<String> },
    /// Holds one text with nothing drawn around it.
    Verbatim { text: String },
}

/// Holds one whole screen until it is drawn.
pub(crate) struct RenderedScreen {
    header: Vec<Span<'static>>,
    body: RenderedBody,
    status: String,
    footer: Vec<Span<'static>>,
}

impl ScreenSyntax for RatatuiScreen {
    type Text = Span<'static>;
    type Line = RenderedLine;
    type Row = Vec<RenderedLine>;
    type Body = RenderedBody;
    type Screen = RenderedScreen;

    fn text(&self, content: impl Into<String>, emphasis: Emphasis) -> Self::Text {
        Span::styled(content.into(), style(emphasis))
    }

    fn gauge(&self, gauge: Gauge) -> Self::Text {
        // The unfilled remainder is the span's own background rather than a character, so the bar
        // states how far it reaches without drawing anything to say so.
        Span::styled(gauge.text(), Style::default().bg(Color::DarkGray))
    }

    fn line(&self, runs: impl Iterator<Item = Self::Text>) -> Self::Line {
        RenderedLine {
            spans: runs.collect(),
            edit: None,
        }
    }

    fn edited_line(
        &self,
        prefix: impl Into<String>,
        text: &str,
        cursor: usize,
        emphasis: Emphasis,
    ) -> Self::Line {
        RenderedLine {
            spans: Vec::new(),
            edit: Some(Edit {
                prefix: prefix.into(),
                text: text.to_owned(),
                cursor,
                style: style(emphasis),
            }),
        }
    }

    fn row(&self, lines: impl Iterator<Item = Self::Line>) -> Self::Row {
        lines.collect()
    }

    fn rows(
        &self,
        title: impl Into<String>,
        focused: Option<usize>,
        rows: impl Iterator<Item = Self::Row>,
    ) -> Self::Body {
        RenderedBody::Rows {
            title: title.into(),
            focused,
            rows: rows.collect(),
        }
    }

    fn message(
        &self,
        title: impl Into<String>,
        message: impl Into<String>,
        emphasis: Emphasis,
    ) -> Self::Body {
        RenderedBody::Message {
            title: title.into(),
            message: message.into(),
            style: style(emphasis),
        }
    }

    fn draft(
        &self,
        title: impl Into<String>,
        placeholder: impl Into<String>,
        text: &str,
        cursor: usize,
        examples: impl Iterator<Item = (String, String)>,
    ) -> Self::Body {
        RenderedBody::Draft {
            title: title.into(),
            placeholder: placeholder.into(),
            text: text.to_owned(),
            cursor,
            examples: examples.collect(),
        }
    }

    fn record(&self, title: impl Into<String>, notes: impl Iterator<Item = String>) -> Self::Body {
        RenderedBody::Record {
            title: title.into(),
            notes: notes.collect(),
        }
    }

    fn verbatim(&self, text: impl Into<String>) -> Self::Body {
        RenderedBody::Verbatim { text: text.into() }
    }

    fn screen(
        &self,
        header: Self::Line,
        body: Self::Body,
        status: impl Into<String>,
        footer: Self::Line,
    ) -> Self::Screen {
        RenderedScreen {
            header: header.spans,
            body,
            status: status.into(),
            footer: footer.spans,
        }
    }

    fn screen_text(&self, screen: &Self::Screen) -> impl Iterator<Item = String> {
        let header = span_text(&screen.header);
        let footer = span_text(&screen.footer);
        let body = match &screen.body {
            RenderedBody::Rows { title, rows, .. } => [title.clone()]
                .into_iter()
                .chain(rows.iter().flatten().map(line_text))
                .collect::<Vec<_>>(),
            RenderedBody::Message { title, message, .. } => vec![title.clone(), message.clone()],
            RenderedBody::Draft {
                title,
                placeholder,
                text,
                examples,
                ..
            } => [
                title.clone(),
                if text.is_empty() {
                    placeholder.clone()
                } else {
                    text.clone()
                },
            ]
            .into_iter()
            .chain(
                examples
                    .iter()
                    .map(|(shape, means)| format!("{shape}  {means}")),
            )
            .collect(),
            RenderedBody::Record { title, notes } => {
                [title.clone()].into_iter().chain(notes.clone()).collect()
            }
            RenderedBody::Verbatim { text } => vec![text.clone()],
        };
        [header]
            .into_iter()
            .chain(body)
            .chain([screen.status.clone(), footer])
    }
}

/// States one emphasis as the color and weight a terminal carries it with.
fn style(emphasis: Emphasis) -> Style {
    match emphasis {
        Emphasis::Plain => Style::default(),
        Emphasis::Name => Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        Emphasis::Heading => Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        Emphasis::Label => Style::default().add_modifier(Modifier::BOLD),
        Emphasis::Selected => Style::default().fg(Color::Black).bg(Color::Cyan),
        Emphasis::Muted => Style::default().fg(Color::DarkGray),
        Emphasis::Running => Style::default().fg(Color::Cyan),
        Emphasis::Held => Style::default().fg(Color::Magenta),
        Emphasis::Finishing => Style::default().fg(Color::Yellow),
        // Success and safety are the same green: one names work that ended well, the other an
        // action that cannot end badly.
        Emphasis::Succeeded | Emphasis::Safe => Style::default().fg(Color::Green),
        Emphasis::Failed => Style::default().fg(Color::LightRed),
        // Orange is not one of the sixteen names a terminal states, so it is stated as itself.
        Emphasis::Caution => Style::default().fg(Color::Rgb(255, 150, 40)),
    }
}

/// Reads one line back as the text it states.
fn line_text(line: &RenderedLine) -> String {
    line.edit.as_ref().map_or_else(
        || span_text(&line.spans),
        |edit| format!("{}{}", edit.prefix, edit.text),
    )
}

/// Reads runs back as the text they state.
fn span_text(spans: &[Span<'static>]) -> String {
    spans.iter().map(|span| span.content.as_ref()).collect()
}

/// Draws one composed screen in the terminal frame.
pub(crate) fn paint(frame: &mut Frame<'_>, screen: &RenderedScreen, area: Rect) {
    let areas = ratatui::layout::Layout::vertical([
        ratatui::layout::Constraint::Length(3),
        ratatui::layout::Constraint::Min(5),
        ratatui::layout::Constraint::Length(3),
    ])
    .split(area);
    frame.render_widget(
        Paragraph::new(Line::from(screen.header.clone()))
            .block(Block::default().borders(Borders::ALL)),
        areas[0],
    );
    paint_body(frame, &screen.body, areas[1]);
    frame.render_widget(
        Paragraph::new(Line::from(elided_runs(
            &screen.footer,
            inner_columns(areas[2]),
        )))
        .block(
            Block::default()
                .title(format!(" {} ", screen.status))
                .borders(Borders::ALL),
        ),
        areas[2],
    );
}

/// States how many columns a bordered pane leaves for what it holds.
fn inner_columns(area: Rect) -> usize {
    usize::from(area.width.saturating_sub(2))
}

/// States as much of one line as the columns hold, ending what does not fit with an ellipsis.
///
/// The menu is composed without a width, because a specification does not know how wide a terminal
/// is. Cutting it where the pane ends would end a word mid-letter; this ends it where a reader can
/// see that something was left out.
fn elided_runs(runs: &[Span<'static>], columns: usize) -> Vec<Span<'static>> {
    if runs.iter().map(|run| run.content.width()).sum::<usize>() <= columns {
        return runs.to_vec();
    }
    // One column is left for the ellipsis, and the run that runs out of columns states it.
    runs.iter()
        .scan(columns.saturating_sub(1), |left, run| {
            match run.content.width() {
                _ if *left == 0 => None,
                width if width <= *left => {
                    *left -= width;
                    Some(run.clone())
                }
                _ => {
                    let ending = Span::styled(elided(&run.content, *left + 1), run.style);
                    *left = 0;
                    Some(ending)
                }
            }
        })
        .collect()
}

/// Draws the content of one page.
fn paint_body(frame: &mut Frame<'_>, body: &RenderedBody, area: Rect) {
    match body {
        RenderedBody::Rows {
            title,
            focused,
            rows,
        } => paint_rows(frame, title, *focused, rows, area),
        RenderedBody::Message {
            title,
            message,
            style,
        } => {
            frame.render_widget(
                Paragraph::new(Span::styled(message.clone(), *style))
                    .block(titled(title))
                    .wrap(Wrap { trim: true }),
                area,
            );
        }
        RenderedBody::Draft {
            title,
            placeholder,
            text,
            cursor,
            examples,
        } => paint_draft(frame, title, placeholder, text, *cursor, examples, area),
        RenderedBody::Record { title, notes } => {
            let lines = notes
                .iter()
                .map(|note| Line::styled(note.clone(), style(Emphasis::Muted)))
                .collect::<Vec<_>>();
            frame.render_widget(
                Paragraph::new(Text::from(lines))
                    .block(titled(title))
                    .wrap(Wrap { trim: false }),
                area,
            );
        }
        // Nothing is drawn around it, and the terminal wraps what does not fit: a reader selects
        // the text with the terminal's own selection, and takes away exactly the text.
        RenderedBody::Verbatim { text } => {
            frame.render_widget(
                Paragraph::new(Text::from(text.clone())).wrap(Wrap { trim: false }),
                area,
            );
        }
    }
}

/// Draws rows in order, keeping the row the cursor rests on in view.
///
/// A row states as many lines as it needs, and a pane states as many as it has room for. When
/// there are more lines than room, the pane is scrolled to the row the cursor rests on rather
/// than dropping it, because a row nobody can see is worse than a row that runs off the bottom.
fn paint_rows(
    frame: &mut Frame<'_>,
    title: &str,
    focused: Option<usize>,
    rows: &[Vec<RenderedLine>],
    area: Rect,
) {
    let width = area.width.saturating_sub(2);
    let inner = usize::from(area.height.saturating_sub(2));
    let mut cursor = None;
    let mut lines = Vec::new();
    let mut starts = Vec::new();
    for row in rows {
        starts.push(lines.len());
        for line in row {
            let drawn = match &line.edit {
                None => Line::from(line.spans.clone()),
                Some(edit) => {
                    let prefix = column_width(&edit.prefix);
                    let (visible, column) =
                        editor_window(&edit.text, edit.cursor, width.saturating_sub(prefix));
                    cursor = Some((
                        area.x
                            .saturating_add(1)
                            .saturating_add(prefix)
                            .saturating_add(column),
                        lines.len(),
                    ));
                    Line::styled(format!("{}{visible}", edit.prefix), edit.style)
                }
            };
            lines.push(drawn);
        }
    }
    let scroll = scrolled_to(focused, &starts, lines.len(), inner);
    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(titled(title))
            .scroll((u16::try_from(scroll).unwrap_or(u16::MAX), 0)),
        area,
    );
    if let Some((x, line)) = cursor
        && line >= scroll
        && line - scroll < inner
    {
        let y = area
            .y
            .saturating_add(1)
            .saturating_add(u16::try_from(line - scroll).unwrap_or(u16::MAX));
        if x < area.right().saturating_sub(1) {
            frame.set_cursor_position(Position { x, y });
        }
    }
}

/// States the first line a pane shows, so the row the cursor rests on stays in view.
fn scrolled_to(focused: Option<usize>, starts: &[usize], total: usize, inner: usize) -> usize {
    if total <= inner {
        return 0;
    }
    let Some(focused) = focused.filter(|index| *index < starts.len()) else {
        return 0;
    };
    // A row is read from its own beginning, so the pane starts there and lets the row run off
    // the bottom rather than opening it part-way down.
    starts[focused].min(total.saturating_sub(inner))
}

/// Draws one text draft, scrolled so its cursor stays visible.
fn paint_draft(
    frame: &mut Frame<'_>,
    title: &str,
    placeholder: &str,
    text: &str,
    cursor: usize,
    examples: &[(String, String)],
    area: Rect,
) {
    // The examples stand under the draft, in the room they need and no more.
    let (area, beneath) = if examples.is_empty() {
        (area, None)
    } else {
        let wanted = u16::try_from(examples.len())
            .unwrap_or(u16::MAX)
            .saturating_add(2);
        let split = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Min(4),
            ratatui::layout::Constraint::Length(wanted),
        ])
        .split(area);
        (split[0], Some(split[1]))
    };
    if let Some(beneath) = beneath {
        paint_examples(frame, examples, beneath);
    }
    let inner_width = area.width.saturating_sub(2).max(1);
    let inner_height = area.height.saturating_sub(2).max(1);
    let (cursor_column, cursor_row) = wrapped_cursor(text, cursor, inner_width);
    let scroll = cursor_row.saturating_sub(inner_height.saturating_sub(1));
    let content = if text.is_empty() {
        Text::from(Line::styled(placeholder.to_owned(), style(Emphasis::Muted)))
    } else {
        Text::from(text.to_owned())
    };
    frame.render_widget(
        Paragraph::new(content)
            .block(titled(title).border_style(Style::default().fg(Color::Cyan)))
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0)),
        area,
    );
    frame.set_cursor_position(Position {
        x: area.x.saturating_add(1).saturating_add(cursor_column),
        y: area
            .y
            .saturating_add(1)
            .saturating_add(cursor_row.saturating_sub(scroll)),
    });
}

/// Draws the shapes a source may be written in, each beside what it means.
fn paint_examples(frame: &mut Frame<'_>, examples: &[(String, String)], area: Rect) {
    let widest = examples
        .iter()
        .map(|(shape, _)| shape.chars().count())
        .max()
        .unwrap_or(0);
    let lines = examples
        .iter()
        .map(|(shape, means)| {
            Line::from(vec![
                Span::styled(format!(" {shape:<widest$}  "), style(Emphasis::Plain)),
                Span::styled(means.clone(), style(Emphasis::Muted)),
            ])
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(Text::from(lines)).block(titled("Examples")),
        area,
    );
}

/// States one named, bordered region.
fn titled(title: &str) -> Block<'static> {
    Block::default()
        .title(format!(" {title} "))
        .borders(Borders::ALL)
}

/// Measures how many terminal columns text occupies.
fn column_width(text: &str) -> u16 {
    u16::try_from(UnicodeWidthStr::width(text)).unwrap_or(u16::MAX)
}

/// Locates a cursor in wrapped text, in terminal columns and wrapped rows.
pub(crate) fn wrapped_cursor(text: &str, cursor: usize, width: u16) -> (u16, u16) {
    let mut row = 0_u16;
    let mut lines = text[..cursor].split('\n').peekable();
    while let Some(line) = lines.next() {
        let columns = column_width(line);
        if lines.peek().is_some() {
            row = row.saturating_add(columns.saturating_sub(1) / width + 1);
        } else {
            row = row.saturating_add(columns / width);
            return (columns % width, row);
        }
    }
    (0, row)
}

/// Windows one-line text around its cursor, eliding what does not fit before it.
pub(crate) fn editor_window(text: &str, cursor: usize, width: u16) -> (String, u16) {
    let budget = width.saturating_sub(1);
    let mut start = cursor;
    for (index, _) in text[..cursor].char_indices().rev() {
        let prefix_width = column_width(&text[index..cursor]);
        let leading = u16::from(index > 0);
        if prefix_width.saturating_add(leading) > budget {
            break;
        }
        start = index;
    }
    let mut visible = if start > 0 {
        format!("…{}", &text[start..cursor])
    } else {
        text[..cursor].to_owned()
    };
    let cursor_column = column_width(&visible);
    for character in text[cursor..].chars() {
        let next_width =
            UnicodeWidthStr::width(visible.as_str()).saturating_add(character.width().unwrap_or(0));
        if next_width > usize::from(width) {
            break;
        }
        visible.push(character);
    }
    (visible, cursor_column)
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_menu_too_long_for_its_pane_ends_where_a_reader_can_see_it() {
        let runs = vec![
            Span::raw("[Esc] Transfers  ".to_owned()),
            Span::raw("[Ctrl+Q] Quit".to_owned()),
        ];

        // Everything fits, so everything is stated.
        assert_eq!(elided_runs(&runs, 40), runs);

        // What does not fit ends with an ellipsis rather than mid-word.
        let cut = elided_runs(&runs, 24);
        let stated = cut
            .iter()
            .map(|run| run.content.to_string())
            .collect::<String>();

        assert_eq!(stated, "[Esc] Transfers  [Ctrl+…");
        assert_eq!(stated.width(), 24);
    }

    use super::{Span, UnicodeWidthStr, editor_window, elided_runs, wrapped_cursor};

    #[test]
    fn editor_cursor_uses_terminal_columns_and_wrapping() {
        assert_eq!(wrapped_cursor("a界", "a界".len(), 10), (3, 0));
        assert_eq!(wrapped_cursor("abcdef", 6, 4), (2, 1));
        assert_eq!(wrapped_cursor("ab\n界", "ab\n界".len(), 4), (2, 1));
        assert_eq!(
            editor_window("long filename.mp4", "long filename.mp4".len(), 8),
            ("…me.mp4".to_owned(), 7)
        );
    }
}
