# rsynko-ui

`rsynko-ui` defines what a transfer manager *looks like* without saying what draws it. It owns the key map, the menu a page offers, the composition of every page, the words a reader reads, the weight each of them carries, and the resolution a progress bar states a share at. `rsynko-manager` owns what the application means; this specification owns how that meaning is presented.

A renderer supplies mechanism only: colors, widgets, borders, layout, cursors, and input decoding. Two renderers that disagree about every one of those still agree about everything here.

## Specification surface

Input is a value, not a callback:

- [`Key`] names one key independently of the mechanism reporting it;
- [`Keystroke`] adds the modifiers that change what a key denotes, and [`EXIT_KEYSTROKE`] names the one chord every page binds, which is what leaves the application;
- [`KeyBinding`] states the keys reaching one intention and the menu action gating it.

`KeyBindingExt` derives the map from the current page: `page_bindings` states every binding a page holds, `keystroke_meaning` states what one keystroke denotes there, and `action_keys` states which keys reach one action. `KeyInputExt::apply_keystroke` applies what a keystroke denotes and refuses it when the gating action has no meaning, so a disabled menu entry and a pressed key refuse the same intention for the same reason.

```rust
use rsynko_ui::{EXIT_KEYSTROKE, Key, Keystroke};

assert_eq!(Keystroke::plain(Key::Up).label(), "↑");
assert_eq!(Keystroke::plain(Key::Character(' ')).label(), "Space");
assert_eq!(EXIT_KEYSTROKE.label(), "Ctrl+Q");
```

[`MenuItem`] states one offered entry: the action, the keys reaching it, the verb it performs here, and whether it currently means anything. `MenuPresentationExt::menu_items` derives the whole menu from the page's bindings and the manager's own availability, so a menu never states a key the page does not bind.

```rust
use rsynko_manager::{ActionAvailability, ManagerAction};
use rsynko_ui::{Key, Keystroke, MenuItem};

let item = MenuItem {
    action: ManagerAction::Activate,
    keys: vec![Keystroke::plain(Key::Enter)],
    verb: "Details".to_owned(),
    availability: ActionAvailability::Enabled,
};

assert_eq!(item.label(), "[Enter] Details");
```

[`Emphasis`] is the weight a run of text carries — heading, selected, muted, running, failed — never a color. `PhaseVocabularyExt`, `ControlVocabularyExt`, `StreamVocabularyExt`, and `SpaceVocabularyExt` name the manager's own values the way a reader reads them, and [`bytes_label`], [`duration_label`], and [`elided`] state quantities in the width a column has.

```rust
use rsynko_manager::TransferPhase;
use rsynko_ui::{Emphasis, PhaseVocabularyExt, bytes_label, elided};

assert_eq!(TransferPhase::Downloading.phase_label(), "Downloading");
assert_eq!(TransferPhase::Downloading.phase_emphasis(), Emphasis::Running);
assert_eq!(TransferPhase::Paused.phase_marker(), "○");
assert_eq!(bytes_label(19_084_083), "18.2 MiB");
assert_eq!(elided("a portable video title", 10), "a portabl…");
```

[`Gauge`] states a completed share at an eighth of a cell, the finest resolution a terminal column has. The unfilled remainder is counted, not drawn, so a renderer states the track however its medium can.

```rust
use rsynko_ui::Gauge;

let gauge = Gauge::of(28, 8);

assert_eq!((gauge.filled, gauge.leading, gauge.track), (2, 1, 5));
assert_eq!(gauge.width(), 8);
assert_eq!(gauge.text(), "██▏     ");
```

[`FormatDescriptionAlg`] states what distinguishes one selectable format from its alternatives, and `FormatLabelExt::format_label` derives the aligned line a chooser compares them by. [`FormatChoiceViewAlg`] states what discovery has said about one request's formats, as [`DiscoveryState`].

## Screens

[`ScreenSyntax`] is the presentation vocabulary itself: runs of text carrying an emphasis, lines, rows, bodies that hold rows or a message or a draft or a record, and one screen composed from a header, a body, a status, and a footer. `ManagerScreenExt::screen` composes every manager page out of that vocabulary alone. Nothing in the composition names a widget, a border, a color, or a terminal.

[`ScreenSyntax::screen_text`] closes the loop: every renderer can say what its own screen reads as, which is what lets one presentation law run against the renderer that will actually draw it rather than against a stand-in.

## Laws

`GaugeLaws::gauge_laws` checks that a gauge occupies its width whatever share it states, never states a share before it is reached, and states nothing at zero and everything at one hundred.

`KeyBindingLaws::key_binding_laws` checks that no page binds one keystroke twice, that every bound keystroke denotes its binding, that exit is bound everywhere and no other modified keystroke denotes anything, that typed scalars insert exactly where a draft is held, and that a refused key leaves the manager as it was.

`MenuPresentationLaws::menu_presentation_laws` checks that every entry states its keys in brackets, that every stated key denotes that entry's action, that every page offers exit last, and that an entry is stated as unavailable exactly when its action has no meaning.

`ScreenLaws::screen_laws` checks that every page names the path that reached it, states every menu entry it offers, names the key that fills an empty collection, states every offered details control exactly once, and states every note a record holds. A scenario reads screens back through the renderer [`ScreenLawFixture`] supplies, so a renderer is checked, not simulated.

Composing a screen is an extension, and its bounds are the vocabulary. Nothing here names a color, a widget, or a terminal:

```rust
use alux_ext::ext;
use rsynko_ui::{Emphasis, GAUGE_WIDTH, Gauge, ScreenSyntax};

#[ext(name = ExampleHeadingExt)]
impl<This> This
where
    This: ScreenSyntax,
{
    /// States one transfer as its name, how far it has come, and the share it has completed.
    fn example_heading(&self, title: &str, percent: u16) -> This::Line {
        self.line(
            [
                self.text(format!("{title}  "), Emphasis::Heading),
                self.gauge(Gauge::of(percent, GAUGE_WIDTH)),
                self.text(format!(" {percent:>3}%"), Emphasis::Plain),
            ]
            .into_iter(),
        )
    }

    /// States one page holding rows under a name, the cursor resting on the first of them.
    fn example_page(&self, title: &str, rows: impl Iterator<Item = This::Line>) -> This::Screen {
        let rows = rows.map(|line| self.row([line].into_iter())).collect::<Vec<_>>();
        let focused = (!rows.is_empty()).then_some(0);
        self.screen(
            self.line([self.text(title, Emphasis::Name)].into_iter()),
            self.rows(title, focused, rows.into_iter()),
            "Actions",
            self.line([].into_iter()),
        )
    }
}
```

What a chooser reads about one format is derived the same way, from what the format states about itself:

```rust
use alux_ext::ext;
use rsynko_manager::MediaStreams;
use rsynko_ui::{FormatDescriptionAlg, FormatLabelExt};

#[ext(name = ExampleChoiceExt)]
impl<This> This
where
    This: FormatDescriptionAlg,
{
    /// States one offered choice, marking the one the request currently fixes.
    fn example_choice(&self, chosen: bool) -> String {
        let marker = if chosen { "▸" } else { " " };
        format!("{marker} {}", self.format_label())
    }

    /// Observes whether the format carries everything a viewer needs by itself.
    fn example_complete(&self) -> bool {
        self.format_streams() == Some(MediaStreams::AudioVideo)
    }
}
```
