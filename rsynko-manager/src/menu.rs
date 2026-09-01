use crate::*;
use alux_ext::ext;

/// Derives action availability from renderer-neutral manager observations.
#[ext(name = ManagerMenuExt)]
pub impl<This> This
where
    This: NavigationStateAlg
        + QueueCatalogAlg
        + DraftStateAlg
        + InputDraftAlg
        + OutputDraftAlg
        + TextEditorStateAlg
        + DetailSelectionAlg,
    This::Entry: QueueEntryAlg + RequestOptionsAlg,
    This::Id: Copy + Eq,
{
    /// Derives whether one reusable action currently has meaning.
    fn action_availability(&self, action: ManagerAction) -> ActionAvailability {
        let enabled = match action {
            ManagerAction::Exit => true,
            ManagerAction::Back => self.page() != ManagerPage::Collection,
            ManagerAction::AddSources => self.page() == ManagerPage::Collection,
            ManagerAction::Space => {
                matches!(self.page(), ManagerPage::Collection | ManagerPage::Details(_))
                    && self
                        .selected_queue_id()
                        .and_then(|id| self.queue_entry(id))
                        .and_then(QueueEntryAlg::space_action)
                        .is_some()
            }
            ManagerAction::Remove => self.page() == ManagerPage::Collection && self.selected_queue_id().is_some(),
            ManagerAction::Previous => self.cursor_can_move(false),
            ManagerAction::Next => self.cursor_can_move(true),
            ManagerAction::Activate => self.selection_can_activate(),
        };
        if enabled { ActionAvailability::Enabled } else { ActionAvailability::Disabled }
    }

    /// Derives whether the current page cursor can move in one direction.
    fn cursor_can_move(&self, forward: bool) -> bool {
        match self.page() {
            ManagerPage::Collection => self.queue_ids().take(2).count() > 1,
            ManagerPage::AddSources | ManagerPage::Output(_) | ManagerPage::Input(_) => self
                .active_text_editor()
                .is_some_and(|(text, cursor)| if forward { cursor < text.len() } else { cursor > 0 }),
            ManagerPage::Details(id) => self.queue_entry(id).is_some_and(|entry| !entry.detail_controls().is_empty()),
            ManagerPage::Formats(id) => {
                self.queue_entry(id).is_some_and(|entry| entry.selectable_choices().next().is_some())
            }
            // A record, a report, and a command are read, not traversed: they state no cursor.
            ManagerPage::Log(_) | ManagerPage::Report(_) | ManagerPage::Command(_) => false,
        }
    }

    /// Derives whether the current cursor position denotes an activatable value.
    fn selection_can_activate(&self) -> bool {
        match self.page() {
            ManagerPage::Collection => self.selected_queue_id().is_some(),
            ManagerPage::AddSources => self.draft().lines().any(|line| !line.trim().is_empty()),
            ManagerPage::Output(id) => {
                self.queue_entry(id).is_some_and(QueueEntryAlg::is_editable) && !self.output_draft().trim().is_empty()
            }
            ManagerPage::Input(id) => {
                self.queue_entry(id).is_some_and(QueueEntryAlg::is_editable) && !self.input_draft().trim().is_empty()
            }
            ManagerPage::Details(id) => self.queue_entry(id).is_some_and(|entry| {
                self.selected_detail_control().is_none_or(|control| entry.detail_controls().contains(&control))
            }),
            ManagerPage::Formats(id) => {
                self.queue_entry(id).is_some_and(|entry| entry.selectable_choices().next().is_some())
            }
            ManagerPage::Log(_) | ManagerPage::Report(_) | ManagerPage::Command(_) => false,
        }
    }
}

/// Identifies one reusable manager-menu action independently of its input binding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManagerAction {
    /// Moves the current page's cursor to the preceding value.
    Previous,
    /// Moves the current page's cursor to the following value.
    Next,
    /// Activates the current page's selected value.
    Activate,
    /// Opens the add-sources editor.
    AddSources,
    /// Applies the selected queue entry's state-dependent Space action.
    Space,
    /// Removes the selected queue entry.
    Remove,
    /// Returns to the parent page.
    Back,
    /// Requests safe application exit.
    Exit,
}

/// Denotes whether a menu action currently has meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionAvailability {
    /// Denotes an action that may be selected and activated.
    Enabled,
    /// Denotes a visible action that must not be selected or activated.
    Disabled,
}

impl ActionAvailability {
    /// Observes whether the action may be activated.
    #[must_use]
    pub const fn is_enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}
