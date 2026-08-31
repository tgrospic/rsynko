use alux_ext::ext;
use ambassador::delegatable_trait;

/// Specifies storage for the text editor active on the current manager page.
#[delegatable_trait]
pub trait TextEditorStateAlg {
    /// Observes active text and its UTF-8 byte cursor.
    fn active_text_editor(&self) -> Option<(&str, usize)>;
    /// Replaces active text and its UTF-8 byte cursor.
    fn set_active_text_editor(&mut self, text: String, cursor: usize);
}

/// Derives UTF-8-safe text editing independently of a renderer.
#[ext(name = TextCursorExt)]
pub impl<This> This
where
    This: TextEditorStateAlg,
{
    /// Inserts text at the active cursor and advances past the insertion.
    fn insert_text(&mut self, insertion: &str) {
        let Some((text, cursor)) = self.active_text_editor() else {
            return;
        };
        let mut edited = text.to_owned();
        edited.insert_str(cursor, insertion);
        self.set_active_text_editor(edited, cursor.saturating_add(insertion.len()));
    }

    /// Deletes the Unicode scalar immediately preceding the active cursor.
    fn delete_before_cursor(&mut self) {
        let Some((text, cursor)) = self.active_text_editor() else {
            return;
        };
        let previous = previous_boundary(text, cursor);
        let mut edited = text.to_owned();
        edited.drain(previous..cursor);
        self.set_active_text_editor(edited, previous);
    }

    /// Deletes the Unicode scalar at the active cursor.
    fn delete_at_cursor(&mut self) {
        let Some((text, cursor)) = self.active_text_editor() else {
            return;
        };
        let next = next_boundary(text, cursor);
        let mut edited = text.to_owned();
        edited.drain(cursor..next);
        self.set_active_text_editor(edited, cursor);
    }

    /// Moves the active cursor one Unicode scalar to the left.
    fn move_cursor_left(&mut self) {
        let Some((text, cursor)) = self.active_text_editor() else {
            return;
        };
        self.set_active_text_editor(text.to_owned(), previous_boundary(text, cursor));
    }

    /// Moves the active cursor one Unicode scalar to the right.
    fn move_cursor_right(&mut self) {
        let Some((text, cursor)) = self.active_text_editor() else {
            return;
        };
        self.set_active_text_editor(text.to_owned(), next_boundary(text, cursor));
    }

    /// Moves the active cursor to the beginning of its logical line.
    fn move_cursor_home(&mut self) {
        let Some((text, cursor)) = self.active_text_editor() else {
            return;
        };
        let home = text[..cursor].rfind('\n').map_or(0, |index| index + 1);
        self.set_active_text_editor(text.to_owned(), home);
    }

    /// Moves the active cursor to the end of its logical line.
    fn move_cursor_end(&mut self) {
        let Some((text, cursor)) = self.active_text_editor() else {
            return;
        };
        let end = text[cursor..]
            .find('\n')
            .map_or(text.len(), |index| cursor + index);
        self.set_active_text_editor(text.to_owned(), end);
    }
}

fn previous_boundary(text: &str, cursor: usize) -> usize {
    text[..cursor]
        .char_indices()
        .next_back()
        .map_or(0, |(index, _)| index)
}

fn next_boundary(text: &str, cursor: usize) -> usize {
    text[cursor..]
        .chars()
        .next()
        .map_or(cursor, |character| cursor + character.len_utf8())
}
