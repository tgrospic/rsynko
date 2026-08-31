use crate::*;
use alux_ext::ext;
use ambassador::delegatable_trait;

/// Specifies manager-page navigation.
#[delegatable_trait]
pub trait NavigationStateAlg: ManagerSorts {
    /// Observes the current page.
    fn page(&self) -> ManagerPage<Self::Id>;
    /// Sets the current page.
    fn set_page(&mut self, page: ManagerPage<Self::Id>);
}

/// Specifies selection of one visible expanded-details control.
#[delegatable_trait]
pub trait DetailSelectionAlg {
    /// Observes the selected details control.
    fn selected_detail_control(&self) -> Option<DetailControl>;
    /// Sets the selected details control.
    fn set_selected_detail_control(&mut self, control: Option<DetailControl>);
}

/// Specifies manager-wide status messages.
#[delegatable_trait]
pub trait ManagerStatusAlg {
    /// Observes the manager status message while one stands.
    fn manager_message(&self) -> Option<&str>;
    /// Sets or clears the manager status message.
    fn set_manager_message(&mut self, message: Option<String>);
}

/// Specifies safe-exit state.
#[delegatable_trait]
pub trait SafeExitAlg {
    /// Observes whether safe exit was requested.
    fn exit_requested(&self) -> bool;
    /// Records a safe-exit request.
    fn request_safe_exit(&mut self);
}

/// Names the collection every page rests under.
///
/// Transferring a path is what this application does, so the collection is named after that
/// rather than after the one source that fetches instead of transferring.
pub const COLLECTION: &str = "Transfers";

/// Derives the path from the collection to the current page.
#[ext(name = BreadcrumbExt)]
pub impl<This> This
where
    This: NavigationStateAlg + QueueCatalogAlg,
    This::Entry: QueueEntryAlg + RequestOptionsAlg,
    This::Id: Copy,
{
    /// Derives one breadcrumb per page the current page rests under, the collection first.
    fn breadcrumbs(&self) -> Vec<Breadcrumb> {
        let mut breadcrumbs = vec![segment(COLLECTION)];
        match self.page() {
            ManagerPage::Collection => {}
            ManagerPage::AddSources => breadcrumbs.push(segment("Add sources")),
            ManagerPage::Details(id) => breadcrumbs.push(segment(self.entry_label(id))),
            ManagerPage::Formats(id) => {
                // What a request chooses between is not the same thing for every request.
                let chooses_media = self
                    .queue_entry(id)
                    .is_some_and(|entry| entry.performer() == Performer::Retrieval);
                let choice = if chooses_media { "Formats" } else { "Transfer" };
                breadcrumbs.extend([segment(self.entry_label(id)), segment(choice)]);
            }
            ManagerPage::Input(id) => {
                breadcrumbs.extend([segment(self.entry_label(id)), segment("Input")]);
            }
            ManagerPage::Output(id) => {
                // A file this application names is not the same thing as a path somebody stated.
                let stated = self
                    .queue_entry(id)
                    .is_some_and(|entry| entry.output_naming() == OutputNaming::Stated);
                let named = if stated { "Output" } else { "File name" };
                breadcrumbs.extend([segment(self.entry_label(id)), segment(named)]);
            }
            ManagerPage::Log(id) => {
                breadcrumbs.extend([segment(self.entry_label(id)), segment("Log")]);
            }
            ManagerPage::Report(id) => {
                breadcrumbs.extend([segment(self.entry_label(id)), segment("Report")]);
            }
            ManagerPage::Command(id) => {
                breadcrumbs.extend([segment(self.entry_label(id)), segment("Command")]);
            }
        }
        breadcrumbs
    }

    /// Observes what one identity denotes, or that the collection no longer holds it.
    fn entry_label(&self, id: This::Id) -> &str {
        self.queue_entry(id)
            .map_or("Missing entry", QueueEntryAlg::label)
    }
}

/// Names one breadcrumb segment.
fn segment(label: impl Into<String>) -> Breadcrumb {
    Breadcrumb {
        label: label.into(),
    }
}
