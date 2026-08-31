use crate::*;
use alux_ext::ext;
use rsynko_media::portable_user_file_name;
use std::path::{Path, PathBuf};

/// Derives collection selection and removal from ordered stable identities.
#[ext(name = CollectionNavigationExt)]
pub impl<This> This
where
    This: NavigationStateAlg + QueueCatalogAlg + QueueRemoveAlg,
    This::Id: Copy + Eq,
{
    /// Selects the adjacent entry with collection-end wraparound.
    fn select_relative(&mut self, forward: bool) {
        let ids = self.queue_ids().collect::<Vec<_>>();
        if ids.is_empty() {
            self.set_selected_queue_id(None);
            return;
        }
        let current = self
            .selected_queue_id()
            .and_then(|selected| ids.iter().position(|id| *id == selected))
            .unwrap_or(0);
        let next = if forward {
            (current + 1) % ids.len()
        } else if current == 0 {
            ids.len() - 1
        } else {
            current - 1
        };
        self.set_selected_queue_id(Some(ids[next]));
        if matches!(self.page(), ManagerPage::Details(_)) {
            self.set_page(ManagerPage::Details(ids[next]));
        }
    }

    /// Removes selection and chooses its nearest remaining neighbor.
    fn remove_selected(&mut self) {
        let ids = self.queue_ids().collect::<Vec<_>>();
        let Some(selected) = self.selected_queue_id() else {
            return;
        };
        let index = ids.iter().position(|id| *id == selected).unwrap_or(0);
        self.remove_queue_entry(selected);
        let remaining = self.queue_ids().collect::<Vec<_>>();
        let next = remaining
            .get(index.min(remaining.len().saturating_sub(1)))
            .copied();
        self.set_selected_queue_id(next);
        if matches!(self.page(), ManagerPage::Details(id) | ManagerPage::Formats(id) | ManagerPage::Output(id) | ManagerPage::Log(id) | ManagerPage::Command(id) if id == selected)
        {
            self.set_page(ManagerPage::Collection);
        }
    }
}

/// Derives expanded-row cursor movement and normalization.
#[ext(name = DetailNavigationExt)]
pub impl<This> This
where
    This: NavigationStateAlg + QueueCatalogAlg + DetailSelectionAlg,
    This::Entry: QueueEntryAlg,
    This::Id: Copy + Eq,
{
    /// Selects an adjacent expanded-row position, including the row itself.
    fn select_detail(&mut self, forward: bool) {
        let ManagerPage::Details(id) = self.page() else {
            return;
        };
        let Some(entry) = self.queue_entry(id) else {
            return;
        };
        let controls = entry.detail_controls();
        if controls.is_empty() {
            return;
        }
        let next = match self
            .selected_detail_control()
            .and_then(|selected| controls.iter().position(|control| *control == selected))
        {
            None if forward => Some(controls[0]),
            None => controls.last().copied(),
            Some(current) if forward && current + 1 == controls.len() => None,
            Some(current) if forward => Some(controls[current + 1]),
            Some(0) => None,
            Some(current) => Some(controls[current - 1]),
        };
        self.set_selected_detail_control(next);
    }

    /// Keeps row focus and replaces a detail cursor that is no longer meaningful.
    fn normalize_detail_control(&mut self, id: This::Id) {
        if self.page() != ManagerPage::Details(id) {
            return;
        }
        let Some(entry) = self.queue_entry(id) else {
            return;
        };
        let controls = entry.detail_controls();
        if self
            .selected_detail_control()
            .is_some_and(|selected| !controls.contains(&selected))
        {
            self.set_selected_detail_control(controls.first().copied());
        }
    }
}

/// Derives output-name editing from an editable entry and a text draft.
#[ext(name = OutputEditingExt)]
pub impl<This> This
where
    This: NavigationStateAlg
        + QueueCatalogAlg
        + OutputDraftAlg
        + InputDraftAlg
        + QueueOutputAlg
        + QueueSourceAlg
        + ManagerStatusAlg,
    This::Entry: QueueEntryAlg,
    This::Id: Copy,
{
    /// Opens input editing only while the selected request still permits its input to change.
    ///
    /// A request whose input was already read — extracted, described, named after — no longer
    /// permits it: changing the input then would make everything derived from it a lie.
    fn open_input(&mut self) {
        let Some(id) = self.selected_queue_id() else {
            return;
        };
        let Some(entry) = self.queue_entry(id) else {
            return;
        };
        if entry.is_editable() && entry.output_naming() == OutputNaming::Stated {
            self.set_input_draft(entry.source().to_owned());
            self.set_page(ManagerPage::Input(id));
            self.set_manager_message(None);
        }
    }

    /// Applies a non-empty input draft.
    fn submit_input(&mut self) {
        let ManagerPage::Input(id) = self.page() else {
            return;
        };
        if self.input_draft().trim().is_empty() {
            self.set_manager_message(Some("enter an input".to_owned()));
            return;
        }
        self.set_queue_source(id, self.input_draft().trim().to_owned());
        self.set_page(ManagerPage::Details(id));
        self.set_manager_message(None);
    }

    /// Opens output-file-name editing only while the selected request is editable.
    fn open_output(&mut self) {
        let Some(id) = self.selected_queue_id() else {
            return;
        };
        let Some(entry) = self.queue_entry(id) else {
            return;
        };
        if entry.is_editable() {
            self.set_output_draft(
                entry
                    .output()
                    .map_or_else(String::new, |path| path.display().to_string()),
            );
            self.set_page(ManagerPage::Output(id));
            self.set_manager_message(None);
        }
    }

    /// Applies a non-empty output draft, as the kind of name the request states it is.
    fn submit_output(&mut self) {
        let ManagerPage::Output(id) = self.page() else {
            return;
        };
        let Some(entry) = self.queue_entry(id) else {
            return;
        };
        let naming = entry.output_naming();
        let extension = entry
            .output()
            .and_then(Path::extension)
            .and_then(std::ffi::OsStr::to_str);
        if self.output_draft().trim().is_empty() {
            self.set_manager_message(Some("enter a name".to_owned()));
            return;
        }
        // A stated path is the answer already; making it portable would take its separators away.
        let output = match naming {
            OutputNaming::Portable => portable_user_file_name(self.output_draft(), extension),
            OutputNaming::Stated => PathBuf::from(self.output_draft().trim()),
        };
        self.set_queue_output(id, output);
        self.set_page(ManagerPage::Details(id));
        self.set_manager_message(None);
    }
}

/// Derives the editable format-selection transition.
#[ext(name = ChoiceEditingExt)]
pub impl<This> This
where
    This: NavigationStateAlg
        + QueueCatalogAlg
        + FormatCatalogStateAlg
        + QueueDryRunAlg
        + ManagerStatusAlg,
    This::Entry: QueueEntryAlg,
    This::Id: Copy,
{
    /// Opens the report of what the selected request would do.
    fn open_report(&mut self) {
        if let Some(id) = self.selected_queue_id()
            && self.queue_entry(id).is_some()
        {
            self.set_page(ManagerPage::Report(id));
            self.set_manager_message(None);
        }
    }

    /// Opens the record of what was observed about the selected request.
    fn open_log(&mut self) {
        if let Some(id) = self.selected_queue_id()
            && self.queue_entry(id).is_some()
        {
            self.set_page(ManagerPage::Log(id));
            self.set_manager_message(None);
        }
    }

    /// Opens the command the selected request would run, stated whole.
    fn open_command(&mut self) {
        if let Some(id) = self.selected_queue_id()
            && self.queue_entry(id).is_some()
        {
            self.set_page(ManagerPage::Command(id));
            self.set_manager_message(None);
        }
    }

    /// Turns the rehearsal mode of the selected request on or off.
    ///
    /// A request that states no rehearsal mode has none to turn, so the intention denotes nothing.
    fn toggle_dry_run(&mut self) {
        if let Some(id) = self.selected_queue_id()
            && let Some(dry_run) = self.queue_entry(id).and_then(QueueEntryAlg::dry_run)
        {
            self.set_queue_dry_run(id, !dry_run);
            self.set_manager_message(None);
        }
    }

    /// Opens format selection only while the selected request is editable.
    fn open_formats(&mut self) {
        let Some(id) = self.selected_queue_id() else {
            return;
        };
        if self.queue_entry(id).is_some_and(QueueEntryAlg::is_editable) {
            self.request_format_catalog(id);
            self.set_page(ManagerPage::Formats(id));
            self.set_manager_message(None);
        } else {
            self.set_manager_message(Some(
                "started downloads are immutable; duplicate to change options".to_owned(),
            ));
        }
    }
}

/// Derives add-source submission from source construction and queue insertion.
#[ext(name = DraftSubmissionExt)]
pub impl<This> This
where
    This: NavigationStateAlg
        + QueueCatalogAlg
        + QueueAppendAlg
        + DraftStateAlg
        + ManagerStatusAlg
        + SourceRequestAlg
        + SubmissionAlg
        + MediaOptionsAlg
        + OutputChoiceAlg,
    This::Id: Copy,
{
    /// Adds requests, focuses the first new identity, and clears stale status.
    fn add_sources(&mut self, requests: Vec<This::Source>) {
        let added = self.append_sources(requests);
        if let Some(added) = added.first().copied() {
            self.set_selected_queue_id(Some(added));
        }
        self.set_manager_message(None);
    }

    /// Submits non-empty trimmed draft lines as progressive requests.
    fn submit_draft(&mut self) {
        let requests = self
            .draft()
            .lines()
            .map(str::trim)
            .filter(|source| !source.is_empty())
            .map(|line| self.submitted(line))
            .collect::<Vec<_>>();
        if requests.is_empty() {
            self.set_manager_message(Some("enter at least one source".to_owned()));
        } else {
            self.add_sources(requests);
            self.set_draft(String::new());
            self.set_page(ManagerPage::Collection);
        }
    }
}

/// Derives lifecycle actions for the selected stable identity.
#[ext(name = QueueLifecycleExt)]
pub impl<This> This
where
    This: NavigationStateAlg
        + QueueCatalogAlg
        + QueuePauseAlg
        + QueueDuplicateAlg
        + TransferStateAlg
        + DetailSelectionAlg
        + ManagerStatusAlg,
    This::Entry: QueueEntryAlg,
    This::Id: Copy + Eq,
{
    /// Applies the selected entry's state-dependent Space transition.
    ///
    /// Space names a collection entry, so it denotes nothing while another page is current.
    fn apply_selected_space(&mut self) {
        if !matches!(
            self.page(),
            ManagerPage::Collection | ManagerPage::Details(_)
        ) {
            return;
        }
        let Some(id) = self.selected_queue_id() else {
            return;
        };
        match self.queue_entry(id).and_then(QueueEntryAlg::space_action) {
            Some(SpaceAction::Start | SpaceAction::Rehearse) => self.set_waiting(id),
            Some(SpaceAction::Pause | SpaceAction::Resume) => self.toggle_queue_pause(id),
            None => {}
        }
        self.normalize_detail_control(id);
    }

    /// Duplicates selection as a fresh editable identity and opens its details.
    fn duplicate_selected(&mut self) {
        if let Some(id) = self.selected_queue_id()
            && let Some(duplicate) = self.duplicate_queue_entry(id)
        {
            self.set_selected_queue_id(Some(duplicate));
            self.set_page(ManagerPage::Details(duplicate));
            self.set_selected_detail_control(None);
            self.set_manager_message(Some("source duplicated as an editable download".to_owned()));
        }
    }

    /// Restarts the selected failed request with its fixed options.
    fn restart_selected(&mut self) {
        if let Some(id) = self.selected_queue_id() {
            self.restart_waiting(id);
            self.normalize_detail_control(id);
            self.set_manager_message(Some("failed download queued again".to_owned()));
        }
    }
}

/// Interprets manager intentions by composing independently derived meanings.
#[ext(name = ManagerIntentExt)]
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
        + InputDraftAlg
        + OutputDraftAlg
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
    /// Applies one manager intention or external observation.
    /// Applies one manager intention through the generated fold.
    fn apply_manager_event(
        &mut self,
        event: ManagerIntentOp<This::Id, This::Source, This::Format, This::Change>,
    ) {
        event.interpret(&mut IntentDispatch(self));
    }
}

/// Applies one manager intention to the state it addresses.
///
/// The dispatch is the generated fold, so this states only what each intention *means*.
struct IntentDispatch<'a, This>(&'a mut This);

impl<This, Id, Source, Format, Change> ManagerIntentInterpreter for IntentDispatch<'_, This>
where
    This: ManagerSorts<Id = Id, Source = Source, Format = Format, Change = Change>
        + NavigationStateAlg
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
        + InputDraftAlg
        + OutputDraftAlg
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
    This::Entry: QueueEntryAlg,
    Id: Copy + Eq,
{
    type Id = Id;
    type Source = Source;
    type Format = Format;
    type Change = Change;

    fn add_sources(&mut self, requests: Vec<Self::Source>) {
        self.0.add_sources(requests);
    }

    fn set_draft(&mut self, draft: String) {
        self.0.set_draft(draft);
    }

    fn insert_text(&mut self, text: String) {
        self.0.insert_text(&text);
    }

    fn set_output_draft(&mut self, draft: String) {
        self.0.set_output_draft(draft);
    }

    fn set_input_draft(&mut self, draft: String) {
        self.0.set_input_draft(draft);
    }

    fn open_input(&mut self) {
        self.0.open_input();
    }

    fn submit_input(&mut self) {
        self.0.submit_input();
    }

    fn source_metadata(&mut self, id: Self::Id, media_id: String, title: Option<String>) {
        self.0.apply_source_metadata(id, media_id, title);
    }

    fn format_catalog(&mut self, id: Self::Id, event: FormatDiscoveryOp<Self::Format>) {
        self.0.apply_format_catalog_event(id, event);
    }

    fn rehearsal(&mut self, id: Self::Id, event: RehearsalObservationOp<Self::Change>) {
        self.0.apply_rehearsal_event(id, event);
        self.0.normalize_detail_control(id);
    }

    fn open_report(&mut self) {
        self.0.open_report();
    }

    fn toggle_dry_run(&mut self) {
        self.0.toggle_dry_run();
    }

    fn transfer(&mut self, id: Self::Id, event: TransferObservationOp) {
        if matches!(event, TransferObservationOp::Started {}) {
            self.0.set_manager_message(None);
        }
        self.0.apply_transfer_event(id, event);
        self.0.normalize_detail_control(id);
    }

    fn open_add_sources(&mut self) {
        self.0.set_draft(String::new());
        self.0.set_page(ManagerPage::AddSources);
    }

    fn open_selected(&mut self) {
        if let Some(id) = self.0.selected_queue_id() {
            self.0.set_page(ManagerPage::Details(id));
            self.0.set_selected_detail_control(None);
        }
    }

    fn activate_detail(&mut self) {
        match self.0.selected_detail_control() {
            Some(DetailControl::Input) => self.0.open_input(),
            Some(DetailControl::Output) => self.0.open_output(),
            Some(DetailControl::Format) => self.0.open_formats(),
            Some(DetailControl::Restart) => self.0.restart_selected(),
            Some(DetailControl::Duplicate) => self.0.duplicate_selected(),
            Some(DetailControl::Log) => self.0.open_log(),
            Some(DetailControl::Command) => self.0.open_command(),
            Some(DetailControl::Report) => self.0.open_report(),
            Some(DetailControl::DryRun) => self.0.toggle_dry_run(),
            Some(DetailControl::Delete) => self.0.remove_selected(),
            None => self.0.set_page(ManagerPage::Collection),
        }
    }

    fn back(&mut self) {
        match self.0.page() {
            ManagerPage::Formats(id)
            | ManagerPage::Output(id)
            | ManagerPage::Input(id)
            | ManagerPage::Log(id)
            | ManagerPage::Report(id)
            | ManagerPage::Command(id) => {
                self.0.set_page(ManagerPage::Details(id));
            }
            ManagerPage::Collection | ManagerPage::AddSources | ManagerPage::Details(_) => {
                self.0.set_page(ManagerPage::Collection);
            }
        }
    }

    fn safe_exit_requested(&mut self) {
        self.0.request_safe_exit();
        self.0
            .set_manager_message(Some("cancelling active downloads".to_owned()));
    }

    fn select_previous_format(&mut self) {
        if let ManagerPage::Formats(id) = self.0.page() {
            self.0.cycle_queue_format(id, false);
        }
    }

    fn select_next_format(&mut self) {
        if let ManagerPage::Formats(id) = self.0.page() {
            self.0.cycle_queue_format(id, true);
        }
    }

    fn submit_draft(&mut self) {
        self.0.submit_draft();
    }

    fn delete_before_cursor(&mut self) {
        self.0.delete_before_cursor();
    }

    fn delete_at_cursor(&mut self) {
        self.0.delete_at_cursor();
    }

    fn move_cursor_left(&mut self) {
        self.0.move_cursor_left();
    }

    fn move_cursor_right(&mut self) {
        self.0.move_cursor_right();
    }

    fn move_cursor_home(&mut self) {
        self.0.move_cursor_home();
    }

    fn move_cursor_end(&mut self) {
        self.0.move_cursor_end();
    }

    fn select_previous(&mut self) {
        self.0.select_relative(false);
    }

    fn select_next(&mut self) {
        self.0.select_relative(true);
    }

    fn select_previous_detail(&mut self) {
        self.0.select_detail(false);
    }

    fn select_next_detail(&mut self) {
        self.0.select_detail(true);
    }

    fn open_formats(&mut self) {
        self.0.open_formats();
    }

    fn open_output(&mut self) {
        self.0.open_output();
    }

    fn submit_output(&mut self) {
        self.0.submit_output();
    }

    fn apply_selected_space(&mut self) {
        self.0.apply_selected_space();
    }

    fn duplicate_selected(&mut self) {
        self.0.duplicate_selected();
    }

    fn remove_selected(&mut self) {
        self.0.remove_selected();
    }

    fn restart_selected(&mut self) {
        self.0.restart_selected();
    }
}
