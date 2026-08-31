/// Names one key independently of the mechanism that reports it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    /// Names the key moving a cursor toward the preceding value.
    Up,
    /// Names the key moving a cursor toward the following value.
    Down,
    /// Names the key moving a text cursor toward the preceding scalar.
    Left,
    /// Names the key moving a text cursor toward the following scalar.
    Right,
    /// Names the key activating the selected value.
    Enter,
    /// Names the key leaving the current page.
    Escape,
    /// Names the key deleting before a text cursor.
    Backspace,
    /// Names the key deleting at a text cursor.
    Delete,
    /// Names the key moving a text cursor to its line beginning.
    Home,
    /// Names the key moving a text cursor to its line end.
    End,
    /// Names one typed Unicode scalar.
    Character(char),
}

impl Key {
    /// Names the key the way a menu states it.
    #[must_use]
    pub fn label(self) -> String {
        match self {
            Self::Up => "↑".to_owned(),
            Self::Down => "↓".to_owned(),
            Self::Left => "←".to_owned(),
            Self::Right => "→".to_owned(),
            Self::Enter => "Enter".to_owned(),
            Self::Escape => "Esc".to_owned(),
            Self::Backspace => "Backspace".to_owned(),
            Self::Delete => "Del".to_owned(),
            Self::Home => "Home".to_owned(),
            Self::End => "End".to_owned(),
            Self::Character(' ') => "Space".to_owned(),
            Self::Character(character) => character.to_string(),
        }
    }

    /// Observes the scalar a typed key contributes to a text draft.
    #[must_use]
    pub const fn typed(self) -> Option<char> {
        match self {
            Self::Character(character) => Some(character),
            Self::Up
            | Self::Down
            | Self::Left
            | Self::Right
            | Self::Enter
            | Self::Escape
            | Self::Backspace
            | Self::Delete
            | Self::Home
            | Self::End => None,
        }
    }
}

/// Denotes one key press together with the modifiers that change what it denotes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Keystroke {
    /// Names the pressed key.
    pub key: Key,
    /// States whether the control modifier was held.
    pub control: bool,
    /// States whether the alternate modifier was held.
    pub alternate: bool,
}

impl Keystroke {
    /// Denotes one unmodified key press.
    #[must_use]
    pub const fn plain(key: Key) -> Self {
        Self {
            key,
            control: false,
            alternate: false,
        }
    }

    /// Denotes one key press with the control modifier held.
    #[must_use]
    pub const fn control(key: Key) -> Self {
        Self {
            key,
            control: true,
            alternate: false,
        }
    }

    /// Observes whether any modifier was held.
    #[must_use]
    pub const fn modified(self) -> bool {
        self.control || self.alternate
    }

    /// Names the keystroke the way a menu states it.
    #[must_use]
    pub fn label(self) -> String {
        // A modified character names itself in upper case, the way keyboards print it.
        let key = if self.modified() {
            self.key.label().to_uppercase()
        } else {
            self.key.label()
        };
        match (self.control, self.alternate) {
            (false, false) => key,
            (true, false) => format!("Ctrl+{key}"),
            (false, true) => format!("Alt+{key}"),
            (true, true) => format!("Ctrl+Alt+{key}"),
        }
    }
}

/// Names the keystroke that leaves the application from every page.
pub const EXIT_KEYSTROKE: Keystroke = Keystroke::control(Key::Character('q'));
