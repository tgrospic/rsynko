use crate::*;
use alux_ext::ext;
use rsynko_manager::*;

/// States one offered menu entry: what reaches it, what it does, and whether it means anything.
pub struct MenuItem {
    /// Names the action the entry performs.
    pub action: ManagerAction,
    /// Names the keys reaching the entry, the one a reader tries first at the front.
    pub keys: Vec<Keystroke>,
    /// Names what activating the entry does here.
    pub verb: String,
    /// States whether the entry means anything in the current state.
    pub availability: ActionAvailability,
}

impl MenuItem {
    /// States the entry as a reader reads it: the keys, then what they do.
    #[must_use]
    pub fn label(&self) -> String {
        let keys = self.keys.iter().map(|key| key.label()).collect::<String>();
        format!("[{keys}] {}", self.verb)
    }
}

/// Derives the menu one page offers from the keys it binds and the actions it enables.
#[ext(name = MenuPresentationExt)]
pub impl<This> This
where
    This: NavigationStateAlg
        + QueueCatalogAlg
        + DraftStateAlg
        + OutputDraftAlg
        + InputDraftAlg
        + TextEditorStateAlg
        + DetailSelectionAlg,
    This::Entry: QueueEntryAlg + RequestOptionsAlg,
    This::Id: Copy + Eq,
{
    /// States every entry the current page offers, in the order a reader scans them.
    ///
    /// Exit closes every menu, because it is the one action every page offers.
    fn menu_items(&self) -> impl Iterator<Item = MenuItem> {
        let page = self.page();
        let items = match page {
            ManagerPage::Collection => vec![
                self.cursor_item("Select"),
                self.menu_item(ManagerAction::Activate, "Details"),
                self.menu_item(ManagerAction::AddSources, "Add"),
                self.menu_item(ManagerAction::Space, self.space_verb()),
                self.menu_item(ManagerAction::Remove, "Remove"),
            ],
            ManagerPage::Details(_) => vec![
                self.cursor_item("Select field or action"),
                self.menu_item(
                    ManagerAction::Activate,
                    if self.selected_detail_control().is_none() { "Close details" } else { "Activate" },
                ),
                self.menu_item(ManagerAction::Space, self.space_verb()),
                self.menu_item(ManagerAction::Back, COLLECTION),
            ],
            ManagerPage::Formats(id) => vec![
                self.cursor_item(self.choice_verb(id)),
                self.menu_item(ManagerAction::Activate, "Accept"),
                self.menu_item(ManagerAction::Back, "Details"),
            ],
            ManagerPage::Log(_) | ManagerPage::Report(_) | ManagerPage::Command(_) => {
                vec![self.menu_item(ManagerAction::Back, "Details")]
            }
            ManagerPage::AddSources => vec![
                self.menu_item(ManagerAction::Previous, "Move"),
                self.menu_item(ManagerAction::Next, "Move"),
                self.menu_item(ManagerAction::Activate, "Add sources"),
                self.menu_item(ManagerAction::Back, COLLECTION),
            ],
            ManagerPage::Output(_) | ManagerPage::Input(_) => vec![
                self.menu_item(ManagerAction::Previous, "Move"),
                self.menu_item(ManagerAction::Next, "Move"),
                self.menu_item(ManagerAction::Activate, "Save"),
                self.menu_item(ManagerAction::Back, "Details"),
            ],
        };
        items.into_iter().chain([self.menu_item(ManagerAction::Exit, "Quit")])
    }

    /// States one entry naming one action, with the keys the page binds to it.
    fn menu_item(&self, action: ManagerAction, verb: impl Into<String>) -> MenuItem {
        MenuItem {
            action,
            keys: self.action_keys(action).take(1).collect(),
            verb: verb.into(),
            availability: self.action_availability(action),
        }
    }

    /// States the one entry walking a list, which both cursor directions reach.
    fn cursor_item(&self, verb: impl Into<String>) -> MenuItem {
        let keys = self
            .action_keys(ManagerAction::Previous)
            .take(1)
            .chain(self.action_keys(ManagerAction::Next).take(1))
            .collect();
        MenuItem {
            action: ManagerAction::Next,
            keys,
            verb: verb.into(),
            availability: self.action_availability(ManagerAction::Next),
        }
    }

    /// Names what the chooser page chooses between for one request.
    fn choice_verb(&self, id: This::Id) -> &'static str {
        let chooses_media = self.queue_entry(id).is_some_and(|entry| entry.performer() == Performer::Retrieval);
        if chooses_media { "Choose format" } else { "Choose transfer" }
    }

    /// Names what Space would do to the selected entry, whether or not it is offered.
    fn space_verb(&self) -> String {
        let entry = self.selected_queue_id().and_then(|id| self.queue_entry(id));
        let Some(entry) = entry else {
            return UNSTATED.to_owned();
        };
        entry
            .space_action()
            .map_or_else(
                || match entry.phase() {
                    TransferPhase::Extracting | TransferPhase::Downloading => "Pause",
                    TransferPhase::Paused => "Resume",
                    TransferPhase::Ready => "Start",
                    TransferPhase::Waiting
                    | TransferPhase::Rehearsing
                    | TransferPhase::Publishing
                    | TransferPhase::Complete
                    | TransferPhase::Failed => UNSTATED,
                },
                SpaceAction::space_label,
            )
            .to_owned()
    }
}
