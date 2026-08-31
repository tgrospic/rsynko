//! The reference renderer: a screen stated as the lines a reader would read.

use rsynko_ui::{Emphasis, Gauge, ScreenSyntax};

/// Renders screens as plain text.
///
/// A renderer states weight however its medium can. Plain text has no weight to state, so this
/// interpreter keeps every word and drops every emphasis, which is exactly what makes it the one
/// to read a screen's meaning back from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextScreen;

impl ScreenSyntax for TextScreen {
    type Text = String;
    type Line = String;
    type Row = Vec<String>;
    type Body = Vec<String>;
    type Screen = Vec<String>;

    fn text(&self, content: impl Into<String>, _emphasis: Emphasis) -> Self::Text {
        content.into()
    }

    fn gauge(&self, gauge: Gauge) -> Self::Text {
        gauge.text()
    }

    fn line(&self, runs: impl Iterator<Item = Self::Text>) -> Self::Line {
        runs.collect()
    }

    fn edited_line(
        &self,
        prefix: impl Into<String>,
        text: &str,
        cursor: usize,
        _emphasis: Emphasis,
    ) -> Self::Line {
        // The cursor is stated where a reader would see it, since text has no cursor of its own.
        let (before, after) = text.split_at(cursor.min(text.len()));
        format!("{}{before}|{after}", prefix.into())
    }

    fn row(&self, lines: impl Iterator<Item = Self::Line>) -> Self::Row {
        lines.collect()
    }

    fn rows(
        &self,
        title: impl Into<String>,
        _focused: Option<usize>,
        rows: impl Iterator<Item = Self::Row>,
    ) -> Self::Body {
        [title.into()].into_iter().chain(rows.flatten()).collect()
    }

    fn message(
        &self,
        title: impl Into<String>,
        message: impl Into<String>,
        _emphasis: Emphasis,
    ) -> Self::Body {
        vec![title.into(), message.into()]
    }

    fn draft(
        &self,
        title: impl Into<String>,
        placeholder: impl Into<String>,
        text: &str,
        cursor: usize,
        examples: impl Iterator<Item = (String, String)>,
    ) -> Self::Body {
        let content = if text.is_empty() {
            placeholder.into()
        } else {
            self.edited_line("", text, cursor, Emphasis::Plain)
        };
        [title.into(), content]
            .into_iter()
            .chain(examples.map(|(shape, means)| format!("{shape}  {means}")))
            .collect()
    }

    fn record(&self, title: impl Into<String>, notes: impl Iterator<Item = String>) -> Self::Body {
        [title.into()].into_iter().chain(notes).collect()
    }

    fn verbatim(&self, text: impl Into<String>) -> Self::Body {
        vec![text.into()]
    }

    fn screen(
        &self,
        header: Self::Line,
        body: Self::Body,
        status: impl Into<String>,
        footer: Self::Line,
    ) -> Self::Screen {
        [header]
            .into_iter()
            .chain(body)
            .chain([status.into(), footer])
            .collect()
    }

    fn screen_text(&self, screen: &Self::Screen) -> impl Iterator<Item = String> {
        screen.clone().into_iter()
    }
}
