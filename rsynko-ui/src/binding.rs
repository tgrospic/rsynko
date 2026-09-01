use crate::*;
use alux_ext::ext;
use rsynko_manager::*;

/// Names the binding one manager's pages state, whatever that manager carries.
pub type ManagerKeyBinding<Manager> = KeyBinding<
    <Manager as ManagerSorts>::Id,
    <Manager as ManagerSorts>::Source,
    <Manager as ManagerSorts>::Format,
    <Manager as ManagerSorts>::Change,
>;

/// Binds keystrokes on one page to the intention they denote.
///
/// A binding states its keys, the menu action they count as, and the intention itself. An
/// intention with no action is ungated: text editing means what it means without a menu entry.
pub struct KeyBinding<Id, Source, Format, Change> {
    /// Names the menu action the binding counts as, when it counts as one.
    pub action: Option<ManagerAction>,
    /// Names every key reaching the binding, the one a menu states first.
    pub keys: Vec<Keystroke>,
    /// Denotes the intention the keys carry.
    pub intent: ManagerIntentOp<Id, Source, Format, Change>,
}

impl<Id, Source, Format, Change> KeyBinding<Id, Source, Format, Change> {
    /// Binds keys to one menu action and the intention that action denotes.
    fn action(
        action: ManagerAction,
        keys: impl IntoIterator<Item = Key>,
        intent: ManagerIntentOp<Id, Source, Format, Change>,
    ) -> Self {
        Self { action: Some(action), keys: keys.into_iter().map(Keystroke::plain).collect(), intent }
    }

    /// Binds keys to an intention no menu action gates.
    fn ungated(keys: impl IntoIterator<Item = Key>, intent: ManagerIntentOp<Id, Source, Format, Change>) -> Self {
        Self { action: None, keys: keys.into_iter().map(Keystroke::plain).collect(), intent }
    }

    /// Observes whether one keystroke reaches the binding.
    fn binds(&self, stroke: Keystroke) -> bool {
        self.keys.contains(&stroke)
    }
}

/// States the keys one page binds and what they denote there.
#[ext(name = KeyBindingExt)]
pub impl<This> This
where
    This: NavigationStateAlg,
{
    /// States every keystroke the current page binds, cursor movement first.
    ///
    /// Typed scalars are not bound here: on an editor page every scalar the table leaves free
    /// denotes insertion, which is one rule rather than one binding per character.
    fn page_bindings(&self) -> impl Iterator<Item = ManagerKeyBinding<This>> {
        let bindings = match self.page() {
            ManagerPage::Collection => vec![
                KeyBinding::action(
                    ManagerAction::Previous,
                    [Key::Up, Key::Character('k')],
                    ManagerIntentOp::SelectPrevious {},
                ),
                KeyBinding::action(
                    ManagerAction::Next,
                    [Key::Down, Key::Character('j')],
                    ManagerIntentOp::SelectNext {},
                ),
                KeyBinding::action(ManagerAction::Activate, [Key::Enter], ManagerIntentOp::OpenSelected {}),
                KeyBinding::action(
                    ManagerAction::AddSources,
                    [Key::Character('a')],
                    ManagerIntentOp::OpenAddSources {},
                ),
                KeyBinding::action(ManagerAction::Space, [Key::Character(' ')], ManagerIntentOp::ApplySelectedSpace {}),
                KeyBinding::action(ManagerAction::Remove, [Key::Delete], ManagerIntentOp::RemoveSelected {}),
            ],
            ManagerPage::Details(_) => vec![
                KeyBinding::action(
                    ManagerAction::Previous,
                    [Key::Up, Key::Character('k')],
                    ManagerIntentOp::SelectPreviousDetail {},
                ),
                KeyBinding::action(
                    ManagerAction::Next,
                    [Key::Down, Key::Character('j')],
                    ManagerIntentOp::SelectNextDetail {},
                ),
                KeyBinding::action(ManagerAction::Activate, [Key::Enter], ManagerIntentOp::ActivateDetail {}),
                KeyBinding::action(ManagerAction::Space, [Key::Character(' ')], ManagerIntentOp::ApplySelectedSpace {}),
                KeyBinding::action(ManagerAction::Back, [Key::Escape, Key::Backspace], ManagerIntentOp::Back {}),
            ],
            ManagerPage::Formats(_) => vec![
                KeyBinding::action(
                    ManagerAction::Previous,
                    [Key::Up, Key::Character('k')],
                    ManagerIntentOp::SelectPreviousFormat {},
                ),
                KeyBinding::action(
                    ManagerAction::Next,
                    [Key::Down, Key::Character('j')],
                    ManagerIntentOp::SelectNextFormat {},
                ),
                // Accepting a choice already applied leaves the page it was chosen on.
                KeyBinding::action(ManagerAction::Activate, [Key::Enter], ManagerIntentOp::Back {}),
                KeyBinding::action(ManagerAction::Back, [Key::Escape, Key::Backspace], ManagerIntentOp::Back {}),
            ],
            ManagerPage::Log(_) | ManagerPage::Report(_) | ManagerPage::Command(_) => {
                vec![KeyBinding::action(ManagerAction::Back, [Key::Escape, Key::Enter], ManagerIntentOp::Back {})]
            }
            ManagerPage::AddSources => editor_bindings(ManagerIntentOp::SubmitDraft {}),
            ManagerPage::Output(_) => editor_bindings(ManagerIntentOp::SubmitOutput {}),
            ManagerPage::Input(_) => editor_bindings(ManagerIntentOp::SubmitInput {}),
        };
        bindings.into_iter()
    }

    /// States what one keystroke denotes on the current page.
    ///
    /// Exit is bound on every page. Any other modified keystroke denotes nothing, so a chord the
    /// terminal reports is never mistaken for the unmodified key inside it.
    fn keystroke_meaning(&self, stroke: Keystroke) -> Option<ManagerKeyBinding<This>> {
        if stroke == EXIT_KEYSTROKE {
            return Some(KeyBinding::action(
                ManagerAction::Exit,
                [EXIT_KEYSTROKE.key],
                ManagerIntentOp::SafeExitRequested {},
            ));
        }
        if stroke.modified() {
            return None;
        }
        self.page_bindings().find(|binding| binding.binds(stroke)).or_else(|| self.typed_meaning(stroke.key.typed()?))
    }

    /// States what one typed scalar denotes on the current page.
    fn typed_meaning(&self, typed: char) -> Option<ManagerKeyBinding<This>> {
        self.page_accepts_text().then(|| {
            KeyBinding::ungated([Key::Character(typed)], ManagerIntentOp::InsertText { text: typed.to_string() })
        })
    }

    /// States what pasted text denotes on the current page.
    fn paste_meaning(&self, text: &str) -> Option<ManagerKeyBinding<This>> {
        self.page_accepts_text().then(|| KeyBinding::ungated([], ManagerIntentOp::InsertText { text: text.to_owned() }))
    }

    /// Observes whether the current page holds a text draft free scalars belong to.
    fn page_accepts_text(&self) -> bool {
        matches!(self.page(), ManagerPage::AddSources | ManagerPage::Output(_) | ManagerPage::Input(_))
    }

    /// States the keys reaching one action on the current page, the primary key first.
    fn action_keys(&self, action: ManagerAction) -> impl Iterator<Item = Keystroke> {
        if action == ManagerAction::Exit {
            return vec![EXIT_KEYSTROKE].into_iter();
        }
        self.page_bindings()
            .find(|binding| binding.action == Some(action))
            .map(|binding| binding.keys)
            .unwrap_or_default()
            .into_iter()
    }
}

/// Interprets input as the intentions it denotes.
#[ext(name = KeyInputExt)]
pub impl<This> This
where
    This: NavigationStateAlg
        + QueueCatalogAlg
        + QueueAppendAlg
        + QueueRemoveAlg
        + QueuePauseAlg
        + QueueDuplicateAlg
        + QueueFormatEditAlg
        + FormatCatalogStateAlg
        + QueueDryRunAlg
        + RehearsalStateAlg
        + TransferStateAlg
        + DetailSelectionAlg
        + DraftStateAlg
        + OutputDraftAlg
        + InputDraftAlg
        + QueueOutputAlg
        + QueueSourceAlg
        + SourceMetadataAlg
        + TextEditorStateAlg
        + ManagerStatusAlg
        + SafeExitAlg
        + SourceRequestAlg
        + SubmissionAlg
        + MediaOptionsAlg
        + OutputChoiceAlg,
    This::Entry: QueueEntryAlg + RequestOptionsAlg,
    This::Id: Copy + Eq,
{
    /// Applies what one keystroke denotes, and observes whether it denoted anything.
    ///
    /// A gated binding applies only while its action has meaning, so a disabled menu entry and a
    /// pressed key refuse the same intention.
    fn apply_keystroke(&mut self, stroke: Keystroke) -> bool {
        self.keystroke_meaning(stroke).is_some_and(|binding| self.apply_binding(binding))
    }

    /// Applies what pasted text denotes, and observes whether it denoted anything.
    fn apply_paste(&mut self, text: &str) -> bool {
        self.paste_meaning(text).is_some_and(|binding| self.apply_binding(binding))
    }

    /// Applies one binding while the action gating it has meaning.
    fn apply_binding(&mut self, binding: ManagerKeyBinding<This>) -> bool {
        let enabled = binding.action.is_none_or(|action| self.action_availability(action).is_enabled());
        if enabled {
            self.apply_manager_event(binding.intent);
        }
        enabled
    }
}

/// States the keys every text editor binds, and the intention submitting its draft.
fn editor_bindings<Id, Source, Format, Change>(
    submit: ManagerIntentOp<Id, Source, Format, Change>,
) -> Vec<KeyBinding<Id, Source, Format, Change>> {
    vec![
        KeyBinding::action(ManagerAction::Previous, [Key::Left], ManagerIntentOp::MoveCursorLeft {}),
        KeyBinding::action(ManagerAction::Next, [Key::Right], ManagerIntentOp::MoveCursorRight {}),
        KeyBinding::action(ManagerAction::Activate, [Key::Enter], submit),
        KeyBinding::action(ManagerAction::Back, [Key::Escape], ManagerIntentOp::Back {}),
        KeyBinding::ungated([Key::Backspace], ManagerIntentOp::DeleteBeforeCursor {}),
        KeyBinding::ungated([Key::Delete], ManagerIntentOp::DeleteAtCursor {}),
        KeyBinding::ungated([Key::Home], ManagerIntentOp::MoveCursorHome {}),
        KeyBinding::ungated([Key::End], ManagerIntentOp::MoveCursorEnd {}),
    ]
}
