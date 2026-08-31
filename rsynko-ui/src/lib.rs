#![doc = include_str!("../README.md")]

mod binding;
mod format;
mod gauge;
mod key;
mod laws;
mod menu;
mod screen;
mod vocabulary;

pub use binding::{KeyBinding, KeyBindingExt, KeyInputExt, ManagerKeyBinding};
pub use format::{
    DiscoveryState, FormatChoiceViewAlg, FormatDescriptionAlg, FormatLabelExt, FormatRolesExt,
};
pub use gauge::Gauge;
pub use key::{EXIT_KEYSTROKE, Key, Keystroke};
pub use laws::{GaugeLaws, KeyBindingLaws, MenuPresentationLaws, ScreenLawFixture, ScreenLaws};
pub use menu::{MenuItem, MenuPresentationExt};
pub use screen::{
    ARMED_MODE, Application, CHOICE_KEY_COLUMNS, COMPACT_TITLE_COLUMNS, CURSOR_MARK, EXPANDED_MARK,
    EXPANDED_TITLE_COLUMNS, FIELD_LABEL_COLUMNS, FIELD_VALUE_COLUMNS, GAUGE_WIDTH,
    ManagerScreenExt, PATH_SEPARATOR, REHEARSING_MODE, SUBMISSION_EXAMPLES, ScreenSyntax,
};
pub use vocabulary::{
    ChangeVocabularyExt, ControlVocabularyExt, Emphasis, PhaseVocabularyExt, SpaceVocabularyExt,
    StreamVocabularyExt, UNSTATED, bytes_label, duration_label, elided,
};
