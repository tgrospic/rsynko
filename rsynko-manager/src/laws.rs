//! Law scenarios for the manager, stated once over the capabilities.
//!
//! A scenario authors its own collection through the capabilities it is bound to, so it constrains
//! any interpreter of the manager sorts and a runner names it without supplying anything. One
//! bundle states one module's meaning, and its bounds name exactly the capabilities that meaning
//! is derived from.

use crate::*;
use alux_ext::ext;
use anyhow::{Result, bail};
use rsynko_media::*;
use rsynko_session::Wanted;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Supplies the collection observation a downloads scenario cannot author for itself.
pub trait DownloadsLawFixture: ManagerSorts {
    /// Observes the inputs one downloads collection holds, in collection order.
    fn law_collected_sources(&self, downloads: &Self::Downloads) -> Vec<String>;
}

/// Authors the collection every stateful manager scenario starts from.
#[ext(name = ManagerLawAuthoring)]
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
    /// Authors the stated number of media requests and observes the identities they were given.
    ///
    /// The requests are stated rather than submitted: what a line reads as is what one
    /// interpreter recognizes, and a scenario about the collection is not about that.
    fn author_law_queue(&mut self, count: usize) -> Vec<This::Id> {
        let requests = (0..count)
            .map(|index| {
                self.source(
                    law_source(index),
                    self.suggested_output(),
                    self.progressive(),
                )
            })
            .collect();
        let ids = self.append_sources(requests);
        // Where the cursor rests after adding is what submitting states; a scenario about
        // anything else starts from the collection already focused.
        self.set_selected_queue_id(ids.first().copied());
        ids
    }
}

/// Names the source one authored entry submits.
fn law_source(index: usize) -> String {
    format!("law://source-{index}")
}

/// Authors the collection-navigation laws.
#[ext(name = NavigationLaws)]
pub impl<This> This
where
    This: NavigationStateAlg
        + QueueCatalogAlg
        + QueueAppendAlg
        + QueueRemoveAlg
        + DraftStateAlg
        + ManagerStatusAlg
        + SourceRequestAlg
        + SubmissionAlg
        + MediaOptionsAlg
        + OutputChoiceAlg,
    This::Entry: QueueEntryAlg + RequestOptionsAlg,
    This::Id: Copy + Eq + Debug,
{
    /// Checks that selection, expansion, and removal preserve stable queue identity.
    ///
    /// The laws checked are:
    ///
    /// 1. adding sources focuses the first new identity;
    /// 2. selection advances in collection order and wraps at both ends;
    /// 3. expanded details name the selected identity and move with it;
    /// 4. removing the selected entry chooses a remaining neighbor;
    /// 5. removing an expanded entry denotes the collection again;
    /// 6. breadcrumbs are rooted at the collection and name the expanded identity.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn navigation_laws(&mut self) -> Result<()> {
        let ids = self.author_law_queue(3);
        if ids.len() != 3 {
            bail!("authoring three sources produced {} entries", ids.len());
        }
        self.select_relative(true);
        if self.selected_queue_id() != ids.get(1).copied() {
            bail!("selecting next did not advance in collection order");
        }
        self.select_relative(false);
        self.select_relative(false);
        if self.selected_queue_id() != ids.last().copied() {
            bail!("selecting previous did not wrap to the last identity");
        }
        self.select_relative(true);
        if self.selected_queue_id() != ids.first().copied() {
            bail!("selecting next did not wrap to the first identity");
        }

        self.set_page(ManagerPage::Details(ids[0]));
        self.select_relative(true);
        if self.page() != ManagerPage::Details(ids[1]) {
            bail!("expanded details did not move with the selection");
        }
        if self.selected_queue_id() != Some(ids[1]) {
            bail!("expanding details displaced the collection selection");
        }

        self.remove_selected();
        let remaining: Vec<This::Id> = self.queue_ids().collect();
        if remaining != [ids[0], ids[2]] {
            bail!("removal did not preserve the surviving collection order: {remaining:?}");
        }
        if self.selected_queue_id() != Some(ids[2]) {
            bail!("removing the selected entry did not choose a remaining neighbor");
        }
        if self.page() != ManagerPage::Collection {
            bail!("removing the expanded entry did not denote the collection again");
        }

        let rooted = self.breadcrumbs();
        if rooted.len() != 1 {
            bail!("the collection rests under {} pages", rooted.len() - 1);
        }
        self.set_page(ManagerPage::Details(ids[2]));
        let expanded = self.breadcrumbs();
        if expanded.first() != rooted.first() {
            bail!("expanded details are not rooted at the collection");
        }
        let named = self.queue_entry(ids[2]).map(QueueEntryAlg::label);
        if expanded.last().map(|crumb| crumb.label.as_str()) != named {
            bail!("the breadcrumb of an expanded entry does not name what it denotes");
        }
        Ok(())
    }
}

/// Authors the editor-draft laws.
#[ext(name = DraftLaws)]
pub impl<This> This
where
    This: NavigationStateAlg
        + QueueCatalogAlg
        + QueueAppendAlg
        + DraftStateAlg
        + OutputDraftAlg
        + InputDraftAlg
        + ManagerStatusAlg
        + SourceRequestAlg
        + SubmissionAlg
        + MediaOptionsAlg
        + OutputChoiceAlg,
    This::Entry: QueueEntryAlg,
    This::Id: Copy + Eq + Debug,
{
    /// Checks that a draft is observed back and submits exactly the sources it names.
    ///
    /// The laws checked are:
    ///
    /// 1. a draft is observed back exactly as it was replaced;
    /// 2. the source draft and the output-name draft are independent;
    /// 3. submitting adds the non-blank trimmed lines in draft order and nothing else;
    /// 4. submitting clears the draft and denotes the collection;
    /// 5. submitting a draft naming no source adds nothing and moves no focus.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn draft_laws(&mut self) -> Result<()> {
        self.set_draft("first\nsecond".to_owned());
        if self.draft() != "first\nsecond" {
            bail!("a replaced draft is not the one observed back");
        }
        self.set_output_draft("name.mp4".to_owned());
        if self.draft() != "first\nsecond" {
            bail!("replacing the output-name draft disturbed the source draft");
        }
        if self.output_draft() != "name.mp4" {
            bail!("a replaced output-name draft is not the one observed back");
        }

        self.set_draft(String::new());
        self.submit_draft();
        if self.queue_ids().next().is_some() {
            bail!("submitting a draft naming no source still added an entry");
        }

        self.set_page(ManagerPage::AddSources);
        self.set_draft(format!("  {}  \n\n{}\n", law_source(0), law_source(1)));
        self.submit_draft();
        let submitted: Vec<String> = self
            .queue_ids()
            .filter_map(|id| self.queue_entry(id))
            .map(|entry| entry.label().to_owned())
            .collect();
        if submitted != [law_source(0), law_source(1)] {
            bail!("submitting did not add exactly the sources the draft names: {submitted:?}");
        }
        if !self.draft().is_empty() {
            bail!("submitting did not clear the draft");
        }
        if self.page() != ManagerPage::Collection {
            bail!("submitting did not denote the collection again");
        }

        let focused = self.selected_queue_id();
        self.set_draft("   ".to_owned());
        self.submit_draft();
        if self.selected_queue_id() != focused {
            bail!("submitting a draft naming no source moved the focus");
        }
        Ok(())
    }
}

/// Authors the text-editing laws.
#[ext(name = TextLaws)]
pub impl<This> This
where
    This: NavigationStateAlg + TextEditorStateAlg,
{
    /// Checks that editing moves by Unicode scalar and never splits one.
    ///
    /// The laws checked are:
    ///
    /// 1. the cursor moves by whole scalars in both directions;
    /// 2. insertion advances past exactly the inserted bytes;
    /// 3. deleting before and at the cursor removes one whole scalar;
    /// 4. every observed cursor rests on a UTF-8 boundary.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn text_editor_laws(&mut self) -> Result<()> {
        // An editor exists on the page that offers one, so the scenario opens one first.
        self.set_page(ManagerPage::AddSources);
        self.set_active_text_editor("a😀b".to_owned(), "a😀".len());
        self.move_cursor_left();
        self.check_law_editor("a😀b", 1)?;
        self.move_cursor_right();
        self.check_law_editor("a😀b", "a😀".len())?;
        self.delete_before_cursor();
        self.check_law_editor("ab", 1)?;

        self.insert_text("😀");
        self.check_law_editor("a😀b", "a😀".len())?;
        self.move_cursor_home();
        self.check_law_editor("a😀b", 0)?;
        self.delete_at_cursor();
        self.check_law_editor("😀b", 0)?;
        self.move_cursor_end();
        self.check_law_editor("😀b", "😀b".len())
    }

    /// Checks the observed text and cursor against the meaning one edit denotes.
    ///
    /// # Errors
    ///
    /// Returns the violated law.
    fn check_law_editor(&self, text: &str, cursor: usize) -> Result<()> {
        let Some((observed, position)) = self.active_text_editor() else {
            bail!("the editor an edit was applied to observes no text");
        };
        if observed != text || position != cursor {
            bail!("editing denoted {observed:?} at {position} rather than {text:?} at {cursor}");
        }
        if !observed.is_char_boundary(position) {
            bail!("the observed cursor {position} splits a Unicode scalar in {observed:?}");
        }
        Ok(())
    }
}

/// Authors the keyed-transfer laws.
#[ext(name = QueueLaws)]
pub impl<This> This
where
    This: NavigationStateAlg
        + QueueCatalogAlg
        + QueueAppendAlg
        + TransferStateAlg
        + SourceMetadataAlg
        + DraftStateAlg
        + ManagerStatusAlg
        + SourceRequestAlg
        + SubmissionAlg
        + MediaOptionsAlg
        + OutputChoiceAlg,
    This::Entry: QueueEntryAlg + TransferViewAlg,
    This::Id: Copy + Eq + Debug,
{
    /// Checks that observations are keyed and that derived progress follows only from them.
    ///
    /// The laws checked are:
    ///
    /// 1. an observation updates exactly the identity it addresses;
    /// 2. byte rate, remaining time, and completed share are derived from bytes and elapsed time;
    /// 3. full byte progress denotes publication, while only terminal success denotes completion;
    /// 4. the submitted source is the label until extraction supplies a title.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn queue_laws(&mut self) -> Result<()> {
        let ids = self.author_law_queue(2);
        let destination = PathBuf::from("law.bin");

        self.apply_transfer_event(
            ids[0],
            TransferObservationOp::Progress {
                destination: destination.clone(),
                downloaded: 4_000_000,
                total: Some(10_000_000),
            },
        );
        self.apply_transfer_event(
            ids[0],
            TransferObservationOp::Elapsed {
                elapsed: Duration::from_secs(2),
            },
        );
        if self.law_transferred(ids[1]) != Some(0) {
            bail!("a keyed observation reached an identity it does not address");
        }

        let observed = self.law_entry(ids[0])?;
        if observed.bytes_per_second() != Some(2_000_000) {
            bail!("byte rate is not derived from observed bytes and elapsed time");
        }
        if observed.estimated_remaining() != Some(Duration::from_secs(3)) {
            bail!("remaining time is not derived from observed bytes and elapsed time");
        }
        if observed.percent() != Some(40) {
            bail!("the completed share is not derived from observed bytes and expected size");
        }

        self.apply_transfer_event(
            ids[0],
            TransferObservationOp::Progress {
                destination: destination.clone(),
                downloaded: 10_000_000,
                total: Some(10_000_000),
            },
        );
        let observed = self.law_entry(ids[0])?;
        if observed.percent() != Some(100) {
            bail!("full byte progress is not observed as a complete share");
        }
        if observed.transfer_complete() {
            bail!("full byte progress denoted completion before terminal success");
        }
        self.apply_transfer_event(
            ids[0],
            TransferObservationOp::Completed {
                destination,
                bytes: 10_000_000,
            },
        );
        if !self.law_entry(ids[0])?.transfer_complete() {
            bail!("terminal success did not denote completion");
        }

        if self.law_entry(ids[1])?.label() != law_source(1) {
            bail!("the submitted source is not the label before extraction supplies a title");
        }
        self.apply_source_metadata(ids[1], "law".to_owned(), Some("Law Title".to_owned()));
        if self.law_entry(ids[1])?.label() != "Law Title" {
            bail!("an extracted title is not the observed label");
        }
        Ok(())
    }

    /// Observes one entry by stable identity.
    ///
    /// # Errors
    ///
    /// Returns a violation when the collection holds no such identity.
    fn law_entry(&self, id: This::Id) -> Result<&This::Entry> {
        match self.queue_entry(id) {
            Some(entry) => Ok(entry),
            None => bail!("the collection holds no entry for {id:?}"),
        }
    }

    /// Observes the bytes one identity has retrieved, when it names an entry.
    fn law_transferred(&self, id: This::Id) -> Option<u64> {
        self.queue_entry(id).map(TransferViewAlg::transferred)
    }
}

/// Authors the derived-transition laws.
#[ext(name = TransitionLaws)]
pub impl<This> This
where
    This: NavigationStateAlg
        + QueueCatalogAlg
        + QueueAppendAlg
        + QueuePauseAlg
        + QueueDuplicateAlg
        + QueueOutputAlg
        + QueueSourceAlg
        + TransferStateAlg
        + SourceMetadataAlg
        + DetailSelectionAlg
        + DraftStateAlg
        + OutputDraftAlg
        + InputDraftAlg
        + ManagerStatusAlg
        + SourceRequestAlg
        + SubmissionAlg
        + MediaOptionsAlg
        + OutputChoiceAlg,
    This::Entry: QueueEntryAlg,
    This::Id: Copy + Eq + Debug,
{
    /// Checks that Space, restart, duplication, and output editing denote their transitions.
    ///
    /// The laws checked are:
    ///
    /// 1. Space schedules exactly the selected ready entry and fixes its request;
    /// 2. pause is offered only while the active interpreter advertises that capability;
    /// 3. a failed request offers restart and restarts without becoming editable again;
    /// 4. duplication creates a fresh editable identity with its own transfer state;
    /// 5. an extracted title is the default output stem;
    /// 6. an output name is editable before first start and normalized when applied.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn transition_laws(&mut self) -> Result<()> {
        let ids = self.author_law_queue(2);
        for (index, id) in ids.iter().copied().enumerate() {
            self.apply_source_metadata(id, format!("law-{index}"), Some(format!("Law {index}")));
        }
        if self.law_space(ids[0])? != Some(SpaceAction::Start) {
            bail!("a ready entry naming an output does not offer Start");
        }
        let derived = self.law_entry(ids[0])?.output().map(Path::to_owned);
        let stem = derived
            .as_deref()
            .and_then(Path::file_stem)
            .and_then(OsStr::to_str);
        if stem != Some("Law 0") {
            bail!("the extracted title is not the default output stem: {stem:?}");
        }

        self.set_selected_queue_id(Some(ids[0]));
        self.set_page(ManagerPage::Output(ids[0]));
        self.set_output_draft("My: copy".to_owned());
        self.submit_output();
        let named = self.law_entry(ids[0])?;
        let Some(output) = named.output() else {
            bail!("applying an output-name draft named no output");
        };
        if output.components().count() != 1 || output.to_string_lossy().contains(':') {
            bail!(
                "an applied output name is not one portable file component: {}",
                output.display()
            );
        }

        self.apply_selected_space();
        if self.law_entry(ids[0])?.is_editable() {
            bail!("first start did not fix the request options");
        }
        if self.law_space(ids[1])? != Some(SpaceAction::Start) {
            bail!("scheduling one entry disturbed another entry's Space meaning");
        }
        if self.law_space(ids[0])?.is_some() {
            bail!("a waiting entry offers a Space action");
        }

        self.apply_transfer_event(ids[0], TransferObservationOp::Started {});
        if self.law_space(ids[0])?.is_some() {
            bail!("an active transfer offers pause although no interpreter advertises it");
        }
        self.apply_transfer_event(
            ids[0],
            TransferObservationOp::PauseCapability { supported: true },
        );
        if self.law_space(ids[0])? != Some(SpaceAction::Pause) {
            bail!("an advertised pause capability does not offer pause");
        }
        self.apply_selected_space();
        if self.law_space(ids[0])? != Some(SpaceAction::Resume) {
            bail!("a paused transfer does not offer resumption");
        }
        self.apply_selected_space();
        if self.law_space(ids[0])? != Some(SpaceAction::Pause) {
            bail!("a resumed transfer does not offer pause again");
        }

        self.apply_transfer_event(
            ids[0],
            TransferObservationOp::ProgramFailed {
                summary: "no matching format".to_owned(),
                detail: "no directly retrievable format matches the request".to_owned(),
            },
        );
        if !self
            .law_entry(ids[0])?
            .detail_controls()
            .contains(&DetailControl::Restart)
        {
            bail!("a failed request offers no restart");
        }
        self.restart_selected();
        if self.law_entry(ids[0])?.is_editable() {
            bail!("restarting a failed request made it editable again");
        }

        self.duplicate_selected();
        let Some(duplicate) = self.selected_queue_id() else {
            bail!("duplication focused no identity");
        };
        if duplicate == ids[0] {
            bail!("duplication reused the identity it duplicates");
        }
        let fresh = self.law_entry(duplicate)?;
        if !fresh.is_editable() {
            bail!("a duplicate is not editable");
        }
        if fresh.space_action() != Some(SpaceAction::Start) {
            bail!("a duplicate does not start from a ready transfer state");
        }
        if fresh.label() != self.law_entry(ids[0])?.label() {
            bail!("duplication did not preserve what the source denotes");
        }
        Ok(())
    }

    /// Observes one entry by stable identity.
    ///
    /// # Errors
    ///
    /// Returns a violation when the collection holds no such identity.
    fn law_entry(&self, id: This::Id) -> Result<&This::Entry> {
        match self.queue_entry(id) {
            Some(entry) => Ok(entry),
            None => bail!("the collection holds no entry for {id:?}"),
        }
    }

    /// Observes the Space meaning one identity currently offers.
    ///
    /// # Errors
    ///
    /// Returns a violation when the collection holds no such identity.
    fn law_space(&self, id: This::Id) -> Result<Option<SpaceAction>> {
        Ok(self.law_entry(id)?.space_action())
    }
}

/// Authors the menu-availability laws.
#[ext(name = MenuLaws)]
pub impl<This> This
where
    This: NavigationStateAlg
        + QueueCatalogAlg
        + QueueAppendAlg
        + QueuePauseAlg
        + QueueDuplicateAlg
        + TransferStateAlg
        + SourceMetadataAlg
        + DetailSelectionAlg
        + DraftStateAlg
        + OutputDraftAlg
        + InputDraftAlg
        + TextEditorStateAlg
        + ManagerStatusAlg
        + SourceRequestAlg
        + SubmissionAlg
        + MediaOptionsAlg
        + OutputChoiceAlg,
    This::Entry: QueueEntryAlg + RequestOptionsAlg,
    This::Id: Copy + Eq + Debug,
{
    /// Checks that availability is derived from page, selection, and queue observations.
    ///
    /// The laws checked are:
    ///
    /// 1. an empty collection enables only the actions that still have meaning;
    /// 2. a selected ready entry enables Space, Remove, and activation;
    /// 3. cursor movement is enabled exactly when the page holds another value to move to;
    /// 4. activation on the add-sources page is enabled exactly when the draft names a source;
    /// 5. a disabled action denotes no operation, so applying it changes nothing observable.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn menu_laws(&mut self) -> Result<()> {
        for (action, expected) in [
            (ManagerAction::Space, ActionAvailability::Disabled),
            (ManagerAction::Remove, ActionAvailability::Disabled),
            (ManagerAction::Activate, ActionAvailability::Disabled),
            (ManagerAction::Next, ActionAvailability::Disabled),
            (ManagerAction::AddSources, ActionAvailability::Enabled),
            (ManagerAction::Exit, ActionAvailability::Enabled),
        ] {
            if self.action_availability(action) != expected {
                bail!("an empty collection denotes {action:?} as {expected:?} nowhere");
            }
        }

        let ids = self.author_law_queue(1);
        self.apply_source_metadata(ids[0], "law".to_owned(), Some("Law".to_owned()));
        for action in [
            ManagerAction::Space,
            ManagerAction::Remove,
            ManagerAction::Activate,
        ] {
            if self.action_availability(action) != ActionAvailability::Enabled {
                bail!("a selected ready entry does not enable {action:?}");
            }
        }
        if self.action_availability(ManagerAction::Next) != ActionAvailability::Disabled {
            bail!("a collection of one entry offers cursor movement");
        }
        let before = self.law_space(ids[0])?;
        self.author_law_queue(1);
        if self.action_availability(ManagerAction::Next) != ActionAvailability::Enabled {
            bail!("a collection of two entries does not offer cursor movement");
        }

        self.set_page(ManagerPage::AddSources);
        self.set_draft("   ".to_owned());
        if self.action_availability(ManagerAction::Activate) != ActionAvailability::Disabled {
            bail!("a draft naming no source enables activation");
        }
        if self.action_availability(ManagerAction::Back) != ActionAvailability::Enabled {
            bail!("a page below the collection does not enable returning to it");
        }
        self.set_draft(law_source(2));
        if self.action_availability(ManagerAction::Activate) != ActionAvailability::Enabled {
            bail!("a draft naming a source does not enable activation");
        }

        // Space has no meaning away from the collection, so applying it must deny that meaning.
        if self.action_availability(ManagerAction::Space) != ActionAvailability::Disabled {
            bail!("the add-sources page enables Space");
        }
        self.set_selected_queue_id(Some(ids[0]));
        self.apply_selected_space();
        if self.law_space(ids[0])? != before {
            bail!("a disabled action denoted an operation when it was applied");
        }
        Ok(())
    }

    /// Observes one entry's Space meaning by stable identity.
    ///
    /// # Errors
    ///
    /// Returns a violation when the collection holds no such identity.
    fn law_space(&self, id: This::Id) -> Result<Option<SpaceAction>> {
        match self.queue_entry(id) {
            Some(entry) => Ok(entry.space_action()),
            None => bail!("the collection holds no entry for {id:?}"),
        }
    }
}

/// Authors the downloads-collection laws.
#[ext(name = DownloadsLaws)]
pub impl<This, Source> This
where
    This: ManagerSorts<Source = Source, Downloads: DownloadCollectionAlg<Source = Source>>
        + DownloadsAlg
        + SourceRequestAlg
        + SubmissionAlg
        + MediaOptionsAlg
        + OutputChoiceAlg
        + DownloadsLawFixture,
{
    /// Checks that a collection holds exactly the sources it was given, in the order given.
    ///
    /// The laws checked are:
    ///
    /// 1. the empty collection holds nothing;
    /// 2. a collection holds its sources in declaration order;
    /// 3. adding appends after what the collection already holds.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn downloads_laws(&self) -> Result<()> {
        let empty = self.empty_downloads();
        if !self.law_collected_sources(&empty).is_empty() {
            bail!("the empty collection holds a source");
        }
        let stated = [law_source(0), law_source(1)];
        let collected = self.progressive_downloads(stated.clone());
        if self.law_collected_sources(&collected) != stated {
            bail!("a collection does not hold its sources in declaration order");
        }
        let appended = collected.add_sources([self.source(
            law_source(2),
            self.suggested_output(),
            self.progressive(),
        )]);
        if self.law_collected_sources(&appended)
            != [stated[0].clone(), stated[1].clone(), law_source(2)]
        {
            bail!("adding did not append after what the collection already holds");
        }
        Ok(())
    }
}

/// Authors the download-record laws.
#[ext(name = LogLaws)]
pub impl<This> This
where
    This: NavigationStateAlg
        + QueueCatalogAlg
        + QueueAppendAlg
        + FormatCatalogStateAlg
        + QueueDryRunAlg
        + RehearsalStateAlg
        + DownloadLogAlg
        + DetailSelectionAlg
        + DraftStateAlg
        + ManagerStatusAlg
        + SourceRequestAlg
        + SubmissionAlg
        + MediaOptionsAlg
        + OutputChoiceAlg,
    This::Entry: QueueEntryAlg,
    This::Id: Copy + Eq + Debug,
{
    /// Checks that the record is keyed, ordered, and reachable from the entry it describes.
    ///
    /// The laws checked are:
    ///
    /// 1. a note reaches exactly the identity it addresses;
    /// 2. notes are observed in the order they were stated;
    /// 3. every entry offers its record among its details controls;
    /// 4. activating that control denotes the record of that identity.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn log_laws(&mut self) -> Result<()> {
        let ids = self.author_law_queue(2);
        let unaddressed: Vec<String> = self.law_notes(ids[1])?;
        self.note_download(ids[0], "first note".to_owned());
        self.note_download(ids[0], "second note".to_owned());

        if self.law_notes(ids[1])? != unaddressed {
            bail!("a note reached an identity it does not address");
        }
        let stated = self.law_notes(ids[0])?;
        let tail: Vec<&String> = stated.iter().rev().take(2).collect();
        if tail != [&"second note".to_owned(), &"first note".to_owned()] {
            bail!("notes are not observed in the order they were stated: {stated:?}");
        }

        let controls = self
            .queue_entry(ids[0])
            .map(QueueEntryAlg::detail_controls)
            .unwrap_or_default();
        if !controls.contains(&DetailControl::Log) {
            bail!("an entry offers no record among its controls: {controls:?}");
        }
        self.set_selected_queue_id(Some(ids[0]));
        self.set_page(ManagerPage::Details(ids[0]));
        self.set_selected_detail_control(Some(DetailControl::Log));
        self.open_log();
        if self.page() != ManagerPage::Log(ids[0]) {
            bail!("activating the record control did not denote that record");
        }
        Ok(())
    }

    /// Observes the notes stated about one identity.
    ///
    /// # Errors
    ///
    /// Returns a violation when the collection holds no such identity.
    fn law_notes(&self, id: This::Id) -> Result<Vec<String>> {
        match self.queue_entry(id) {
            Some(entry) => Ok(entry.download_log().map(str::to_owned).collect()),
            None => bail!("the collection holds no entry for {id:?}"),
        }
    }
}

/// Supplies the sources a submission scenario cannot author for itself.
///
/// Which texts name a folder and which name a media item is what an interpreter recognizes; that
/// there are two kinds, and what each becomes, is what this specification states.
pub trait SubmissionLawFixture {
    /// Names one line this interpreter reads as a transfer, and the two ends it names.
    fn law_folder_submission(&self) -> (String, String, String);

    /// Names one line naming where a transfer comes from, and where it should come to rest.
    fn law_lone_submission(&self) -> (String, String);

    /// Names one line this interpreter reads as a media item.
    fn law_media_source(&self) -> String;

    /// Names one line no source recognizes, which is therefore one path to transfer.
    fn law_unrecognized_source(&self) -> String;
}

/// Authors the submission laws.
#[ext(name = SubmissionLaws)]
pub impl<This> This
where
    This: NavigationStateAlg
        + QueueCatalogAlg
        + QueueAppendAlg
        + DraftStateAlg
        + TextEditorStateAlg
        + ManagerStatusAlg
        + SourceRequestAlg
        + SubmissionAlg
        + MediaOptionsAlg
        + OutputChoiceAlg
        + SourceRecognitionAlg
        + SubmissionLawFixture,
    This::Entry: QueueEntryAlg + RequestOptionsAlg,
    This::Id: Copy + Eq + Debug,
{
    /// Checks that what a source names decides what transferring it means.
    ///
    /// The laws checked are:
    ///
    /// 1. a line naming two ends states one request from the first end into the second;
    /// 2. a folder chooses no media role, and a media item does;
    /// 3. a folder offers the ways it may be transferred, and always transfers one of them;
    /// 4. a folder states a rehearsal mode, begins with it on, and Space rehearses it;
    /// 5. a media item states no rehearsal mode, and Space starts it once it names an output;
    /// 6. a request performed by naming a program states that command, and one fetched does not;
    /// 7. a transfer names where it would come to rest without anything being extracted first,
    ///    and names it after what is being transferred rather than after where anybody stands;
    /// 8. submitting sources focuses the first of them;
    /// 9. a line no source recognizes is transferred, whatever else it looks like;
    /// 10. exactly the requests performed by naming a program state the command they run.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn submission_laws(&mut self) -> Result<()> {
        let (folder, input, output) = self.law_folder_submission();
        let media = self.law_media_source();
        self.set_draft(format!("{folder}\n{media}"));
        self.submit_draft();
        let ids = self.queue_ids().collect::<Vec<_>>();
        let [folder_id, media_id] = ids[..] else {
            bail!("submitting two sources did not append two requests");
        };

        if self.selected_queue_id() != Some(folder_id) {
            bail!("submitting sources did not focus the first of them");
        }

        let submitted = self.submission_law_entry(folder_id)?;
        if submitted.source() != input {
            bail!("a transfer does not come from the end its line named");
        }
        if submitted.output().map(Path::to_path_buf) != Some(PathBuf::from(&output)) {
            bail!("a transfer does not come to rest at the end its line named");
        }
        if submitted.performer() != Performer::Program {
            bail!("a transfer is not performed by naming a program");
        }
        if submitted.media_streams().is_some() {
            bail!("a folder chose a media role");
        }
        // A folder chooses no representation, and chooses one way of transferring instead.
        if submitted.selectable_choices().next().is_none() {
            bail!("a folder offers no way of transferring itself");
        }
        if submitted.chosen_choice().is_none() {
            bail!("a folder transfers no stated way");
        }
        if submitted
            .selectable_choices()
            .any(|choice| submitted.choice_summary(choice).is_none())
        {
            bail!("a way of transferring does not say what it does");
        }
        if submitted.dry_run() != Some(true) {
            bail!("a folder does not begin by stating what it would do");
        }
        if submitted.stated_command().is_none() {
            bail!("a folder does not state the command that would perform it");
        }
        if submitted.space_action() != Some(SpaceAction::Rehearse) {
            bail!("Space does not rehearse a folder nobody has rehearsed");
        }

        self.check_unrecognized_laws()?;
        for id in [folder_id, media_id] {
            let submitted = self.submission_law_entry(id)?;
            // What performs a request decides whether there is a command to read: a program is
            // named and read back, and a retrieval is not named at all.
            let names_a_program = submitted.performer() == Performer::Program;
            if submitted.stated_command().is_some() != names_a_program {
                bail!("what performs a request and whether it states a command disagree");
            }
        }

        let submitted = self.submission_law_entry(media_id)?;
        if submitted.performer() != Performer::Retrieval {
            bail!("a line a source claimed is not retrieved");
        }
        if submitted.media_streams().is_none() {
            bail!("a media item chose no media role");
        }
        if submitted.dry_run().is_some() {
            bail!("a media item states a rehearsal mode it cannot have");
        }
        if submitted.chosen_choice().is_some() {
            bail!("a media item fixed a representation nobody chose");
        }
        if submitted.stated_command().is_some() {
            bail!("a media item states a command, and it is fetched rather than run");
        }
        Ok(())
    }

    /// Checks that a line nobody claimed is a path rather than something to fetch.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn check_unrecognized_laws(&mut self) -> Result<()> {
        let line = self.law_unrecognized_source();
        if self.recognizes_source(&line) {
            bail!("the line authored as unrecognized is recognized by a source");
        }
        self.set_draft(line.clone());
        self.submit_draft();
        let Some(id) = self.selected_queue_id() else {
            bail!("submitting {line} appended nothing");
        };
        let submitted = self.submission_law_entry(id)?;
        if submitted.source() != line {
            bail!("a transfer does not come from the path its line named");
        }
        if submitted.performer() != Performer::Program {
            bail!("a line no source recognizes is not performed by naming a program");
        }
        if submitted.stated_command().is_none() {
            bail!("a line no source recognizes states no command to perform it");
        }
        Ok(())
    }

    /// Observes the entry one identity denotes, or that the collection no longer holds it.
    fn submission_law_entry(&self, id: This::Id) -> Result<&This::Entry> {
        let Some(entry) = self.queue_entry(id) else {
            bail!("the collection no longer holds {id:?}");
        };
        Ok(entry)
    }
}
/// Supplies the changes a rehearsal scenario cannot author for itself.
pub trait RehearsalLawFixture: ManagerSorts {
    /// Defines one change a rehearsal would make to a path.
    fn law_change(&self, path: &str, kind: ChangeKind, size: Option<u64>) -> Self::Change;

    /// Turns the rehearsal mode on for one request, the way a folder source would.
    fn law_offer_rehearsal(&mut self, id: Self::Id);
}

/// Authors the rehearsal laws.
#[ext(name = RehearsalLaws)]
pub impl<This> This
where
    This: NavigationStateAlg
        + QueueCatalogAlg
        + QueueAppendAlg
        + QueueDryRunAlg
        + QueueDuplicateAlg
        + QueueOutputAlg
        + QueueSourceAlg
        + RehearsalStateAlg
        + DetailSelectionAlg
        + DraftStateAlg
        + ManagerStatusAlg
        + SourceRequestAlg
        + SubmissionAlg
        + MediaOptionsAlg
        + OutputChoiceAlg
        + RehearsalLawFixture,
    This::Entry: QueueEntryAlg + RehearsalViewAlg<Change: PlannedChangeAlg>,
    This::Id: Copy + Eq + Debug,
{
    /// Checks that stating what a transfer would do is not doing it.
    ///
    /// The laws checked are:
    ///
    /// 1. a request states a rehearsal mode exactly when it offers the control turning it;
    /// 2. the mode decides what Space means, and nothing else about the request;
    /// 3. a rehearsal leaves the request exactly as editable as it found it;
    /// 4. a reported rehearsal states every change it was given, and offers its report;
    /// 5. a rehearsed request still permits its input to change, and states that it does;
    /// 6. changing what would be run forgets the report describing what would have been;
    /// 7. the mode changes what the request states would be run;
    /// 8. a failed rehearsal is not a failed request: Space still means something afterward;
    /// 9. a duplicate keeps both ends and is not armed, however armed the first one was.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn rehearsal_laws(&mut self) -> Result<()> {
        let ids = self.author_law_queue(1);
        let id = ids[0];
        if self.rehearsal_law_entry(id)?.dry_run().is_some() {
            bail!("a request states a rehearsal mode it was never given");
        }
        if self
            .rehearsal_law_controls(id)?
            .contains(&DetailControl::DryRun)
        {
            bail!("a request with no rehearsal mode offers the control turning it");
        }
        self.law_offer_rehearsal(id);
        // A folder transfer names the folder it would write into, the same as any other request.
        self.set_queue_output(id, PathBuf::from("law-destination"));
        if self.rehearsal_law_entry(id)?.dry_run() != Some(true) {
            bail!("a folder request does not begin by stating what it would do");
        }
        if !self
            .rehearsal_law_controls(id)?
            .contains(&DetailControl::DryRun)
        {
            bail!("a request with a rehearsal mode does not offer the control turning it");
        }
        if self
            .rehearsal_law_controls(id)?
            .contains(&DetailControl::Report)
        {
            bail!("an unrehearsed request offers a report of what it would do");
        }
        if self.rehearsal_law_entry(id)?.space_action() != Some(SpaceAction::Rehearse) {
            bail!("Space does not rehearse while the rehearsal mode is on");
        }

        let editable = self.rehearsal_law_entry(id)?.is_editable();
        self.apply_rehearsal_event(id, RehearsalObservationOp::Started {});
        if self.rehearsal_law_entry(id)?.phase() != TransferPhase::Rehearsing {
            bail!("a started rehearsal is not observable as one");
        }
        let changes = vec![
            self.law_change("kept.txt", ChangeKind::Unchanged, Some(2)),
            self.law_change("new.txt", ChangeKind::Create, Some(4)),
        ];
        self.apply_rehearsal_event(id, RehearsalObservationOp::Reported { changes });
        if self.rehearsal_law_entry(id)?.is_editable() != editable {
            bail!("a rehearsal changed what the request permits");
        }
        if self.rehearsal_law_entry(id)?.phase() != TransferPhase::Ready {
            bail!("a finished rehearsal left the request in the phase that ran it");
        }
        let reported = self
            .rehearsal_law_entry(id)?
            .planned_changes()
            .map(|change| (change.change_path().to_owned(), change.change_kind()))
            .collect::<Vec<_>>();
        if reported
            != vec![
                ("kept.txt".to_owned(), ChangeKind::Unchanged),
                ("new.txt".to_owned(), ChangeKind::Create),
            ]
        {
            bail!("a report does not state the changes the rehearsal stated: {reported:?}");
        }
        if !self
            .rehearsal_law_controls(id)?
            .contains(&DetailControl::Report)
        {
            bail!("a rehearsed request does not offer its report");
        }

        self.check_open_input_laws(id)?;
        let rehearsed = self.rehearsal_law_entry(id)?.stated_command();
        self.set_queue_dry_run(id, false);
        let armed = self.rehearsal_law_entry(id)?.stated_command();
        if rehearsed.is_some() && rehearsed == armed {
            bail!("turning the rehearsal mode off did not change what would be run");
        }
        if self.rehearsal_law_entry(id)?.space_action() != Some(SpaceAction::Start) {
            bail!("Space does not transfer once the rehearsal mode is off");
        }
        if !self
            .rehearsal_law_controls(id)?
            .contains(&DetailControl::DryRun)
        {
            bail!("turning the rehearsal mode off took away the control turning it back on");
        }
        self.apply_rehearsal_event(
            id,
            RehearsalObservationOp::Failed {
                message: "the far end refused".to_owned(),
            },
        );
        if self.rehearsal_law_entry(id)?.space_action().is_none() {
            bail!("a failed rehearsal left the request with nothing Space can do");
        }
        self.check_duplicate_laws(id)
    }

    /// Checks that a request duplicated from an armed one is not itself armed.
    ///
    /// Duplicating states a fresh request, and a fresh request states what it would do before it
    /// does it, however it came to exist. A transfer is the pair of ends it names, so both are
    /// kept: a duplicate nobody can run without naming them again is not a duplicate.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn check_duplicate_laws(&mut self, id: This::Id) -> Result<()> {
        let armed = self.rehearsal_law_entry(id)?;
        let (source, output) = (
            armed.source().to_owned(),
            armed.output().map(Path::to_path_buf),
        );
        let Some(duplicate) = self.duplicate_queue_entry(id) else {
            bail!("a request that may be duplicated was not");
        };
        let fresh = self.rehearsal_law_entry(duplicate)?;
        if fresh.dry_run() != Some(true) {
            bail!("a duplicate of an armed request is armed itself");
        }
        if fresh.source() != source {
            bail!("a duplicated transfer does not come from where the first one did");
        }
        if fresh.output().map(Path::to_path_buf) != output {
            bail!("a duplicated transfer does not come to rest where the first one did");
        }
        Ok(())
    }

    /// Checks that a rehearsed request still permits its input to change.
    ///
    /// A rehearsal is not a start, so everything the request states is still open afterward.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn check_open_input_laws(&mut self, id: This::Id) -> Result<()> {
        let replacement = "law://replaced-input";
        self.set_queue_source(id, replacement.to_owned());
        if self.rehearsal_law_entry(id)?.source() != replacement {
            bail!("a rehearsed request refused to change its input");
        }
        if !self
            .rehearsal_law_controls(id)?
            .contains(&DetailControl::Input)
        {
            bail!("a request that permits its input to change does not offer to change it");
        }
        Ok(())
    }

    /// Observes the entry one identity denotes, or that the collection no longer holds it.
    fn rehearsal_law_entry(&self, id: This::Id) -> Result<&This::Entry> {
        let Some(entry) = self.queue_entry(id) else {
            bail!("the collection no longer holds {id:?}");
        };
        Ok(entry)
    }

    /// Observes the controls one identity offers.
    fn rehearsal_law_controls(&self, id: This::Id) -> Result<Vec<DetailControl>> {
        Ok(self.rehearsal_law_entry(id)?.detail_controls())
    }
}

/// Authors the attendance laws.
#[ext(name = AttentionLaws)]
pub impl<This> This
where
    This: NavigationStateAlg
        + QueueCatalogAlg
        + QueueAppendAlg
        + QueueRemoveAlg
        + QueuePauseAlg
        + QueueOutputAlg
        + DraftStateAlg
        + ManagerStatusAlg
        + SafeExitAlg
        + SourceRequestAlg
        + SubmissionAlg
        + MediaOptionsAlg
        + OutputChoiceAlg
        + TransferStateAlg
        + RehearsalStateAlg,
    This::Entry: QueueEntryAlg,
    This::Change: PlannedChangeAlg,
    This::Id: Copy + Eq + Debug,
{
    /// Checks what the collection states to whoever is doing the work it asks for.
    ///
    /// The laws checked are:
    ///
    /// 1. exactly the requests waiting for an interpreter want work done;
    /// 2. a request whose work has begun is no longer waiting for it;
    /// 3. a request being worked on wants its run, a paused one wants it held, and a removed one
    ///    wants none;
    /// 4. once the reader is leaving, no request wants its run;
    /// 5. a destination is taken exactly while another request is writing to it.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn attention_laws(&mut self) -> Result<()> {
        let ids = self.author_law_queue(2);
        // Both name one destination, which is the situation the collection has to state.
        let destination = PathBuf::from("law-destination");
        for id in &ids {
            self.set_queue_output(*id, destination.clone());
        }
        if !self.wanting_work().is_empty() {
            bail!("a request nobody asked for wants work done");
        }
        self.set_waiting(ids[0]);
        if self.wanting_work() != vec![ids[0]] {
            bail!("the requests wanting work are not the requests waiting for it");
        }
        if self.destination_is_taken(ids[1]) {
            bail!("a destination nobody is writing to is taken");
        }

        self.begun(ids[0], true);
        if !self.wanting_work().is_empty() {
            bail!("a request whose work began is still waiting for it to begin");
        }
        if self.wanted(ids[0]) != Wanted::Running {
            bail!("a request being worked on does not want its run");
        }
        if !self.destination_is_taken(ids[1]) {
            bail!("a destination another request is writing to is free");
        }
        if self.destination_is_taken(ids[0]) {
            bail!("a request takes its own destination from itself");
        }

        self.toggle_queue_pause(ids[0]);
        if self.wanted(ids[0]) != Wanted::Held {
            bail!("a paused request does not want its run held still");
        }
        if !self.destination_is_taken(ids[1]) {
            bail!("a destination a held request still holds open is free");
        }
        self.toggle_queue_pause(ids[0]);
        if self.wanted(ids[0]) != Wanted::Running {
            bail!("a resumed request does not want its run to carry on");
        }

        self.remove_queue_entry(ids[0]);
        if self.wanted(ids[0]) != Wanted::Unwanted {
            bail!("a removed request still wants its run");
        }
        if self.destination_is_taken(ids[1]) {
            bail!("a removed request still holds its destination");
        }

        self.set_waiting(ids[1]);
        self.request_safe_exit();
        if self.wanted(ids[1]) != Wanted::Unwanted {
            bail!("a request still wants its run once the reader is leaving");
        }
        Ok(())
    }
}

/// Authors the request-options laws.
#[ext(name = OptionsLaws)]
pub impl<This, Format> This
where
    This: ManagerSorts<Format = Format>
        + MediaSorts<Format = Format>
        + MetadataAlg
        + FormatAlg
        + FormatPredicateAlg
        + FormatPredicateMatchAlg
        + NavigationStateAlg
        + QueueCatalogAlg
        + QueueAppendAlg
        + QueueFormatEditAlg
        + FormatCatalogStateAlg
        + QueueDryRunAlg
        + RehearsalStateAlg
        + SourceMetadataAlg
        + TransferStateAlg
        + DraftStateAlg
        + ManagerStatusAlg
        + SourceRequestAlg
        + SubmissionAlg
        + MediaOptionsAlg
        + OutputChoiceAlg,
    This::Entry: QueueEntryAlg + RequestOptionsAlg,
    This::Id: Copy + Eq + Debug,
{
    /// Checks that one selector states the preferred roles and every discovered identity.
    ///
    /// The laws checked are:
    ///
    /// 1. the predicate one role denotes accepts exactly the formats carrying those streams;
    /// 2. a discovery observation reaches exactly the identity it addresses;
    /// 3. every discovered format is selectable, in discovery order;
    /// 4. selection walks the offered roles first and the discovered identities after;
    /// 5. preferring a role releases any identity, and choosing an identity fixes exactly it;
    /// 6. a request that is no longer editable keeps the choice it was started with.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn options_laws(&mut self) -> Result<()> {
        let described = [
            ("progressive", true, true),
            ("audio", true, false),
            ("video", false, true),
        ]
        .map(|(id, audio, video)| self.law_format(id, audio, video));
        for (role, expected) in [
            (MediaStreams::AudioVideo, "progressive"),
            (MediaStreams::Audio, "audio"),
            (MediaStreams::Video, "video"),
        ] {
            let predicate = self.stream_role_format(role);
            let accepted: Vec<&str> = ["progressive", "audio", "video"]
                .into_iter()
                .zip(&described)
                .filter(|(_, format)| self.format_matches(&predicate, format))
                .map(|(id, _)| id)
                .collect();
            if accepted != [expected] {
                bail!("the predicate {role:?} denotes accepts {accepted:?} rather than {expected}");
            }
        }

        let ids = self.author_law_queue(2);
        self.apply_format_catalog_event(
            ids[0],
            FormatDiscoveryOp::Available {
                formats: described.into_iter().collect(),
            },
        );
        if self.law_selectable(ids[1])?.next().is_some() {
            bail!("a discovery observation reached an identity it does not address");
        }
        let discovered = ["progressive", "audio", "video"];
        let selectable: Vec<String> = self.law_selectable(ids[0])?.map(str::to_owned).collect();
        if selectable != discovered {
            bail!("the selectable formats are not the discovered ones: {selectable:?}");
        }

        // Selection walks one list: every offered role, then every discovered identity.
        let mut walked = Vec::new();
        for _ in 0..MediaStreams::OFFERED.len() + discovered.len() {
            let entry = self.law_entry(ids[0])?;
            walked.push(entry.chosen_choice().map(str::to_owned));
            self.cycle_queue_format(ids[0], true);
        }
        let roles = walked
            .iter()
            .take(MediaStreams::OFFERED.len())
            .filter(|chosen| chosen.is_none())
            .count();
        if roles != MediaStreams::OFFERED.len() {
            bail!("selection does not begin with the offered roles: {walked:?}");
        }
        let identities: Vec<String> = walked.into_iter().flatten().collect();
        if identities != discovered {
            bail!("selection does not continue through the discovered identities: {identities:?}");
        }

        // Stepping back onto a role releases whatever identity was fixed.
        self.cycle_queue_format(ids[0], true);
        if self.law_entry(ids[0])?.chosen_choice().is_some() {
            bail!("preferring a role kept the identity it was meant to release");
        }

        self.apply_source_metadata(ids[0], "law".to_owned(), Some("Law".to_owned()));
        self.set_waiting(ids[0]);
        let fixed = self.law_entry(ids[0])?.media_streams();
        self.cycle_queue_format(ids[0], true);
        if self.law_entry(ids[0])?.media_streams() != fixed {
            bail!("a started request changed the choice it was started with");
        }
        Ok(())
    }

    /// Defines one described format carrying the stated streams.
    fn law_format(&self, id: &str, audio: bool, video: bool) -> Format {
        self.format(
            id,
            self.metadata([
                (FORMAT_HAS_AUDIO.to_owned(), self.boolean_metadata(audio)),
                (FORMAT_HAS_VIDEO.to_owned(), self.boolean_metadata(video)),
                (FORMAT_EXTENSION.to_owned(), self.string_metadata("mp4")),
            ]),
        )
    }

    /// Observes one entry by stable identity.
    ///
    /// # Errors
    ///
    /// Returns a violation when the collection holds no such identity.
    fn law_entry(&self, id: This::Id) -> Result<&This::Entry> {
        match self.queue_entry(id) {
            Some(entry) => Ok(entry),
            None => bail!("the collection holds no entry for {id:?}"),
        }
    }

    /// Observes the identities one entry currently offers.
    ///
    /// # Errors
    ///
    /// Returns a violation when the collection holds no such identity.
    fn law_selectable(&self, id: This::Id) -> Result<impl Iterator<Item = &str>> {
        Ok(self.law_entry(id)?.selectable_choices())
    }
}

/// Authors the intent-stream laws.
#[ext(name = IntentLaws)]
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
    This::Id: Copy + Eq + Debug,
{
    /// Checks that each intention denotes exactly the meaning derived for it.
    ///
    /// The laws checked are:
    ///
    /// 1. the selection intentions denote derived collection navigation;
    /// 2. expanding and returning is the identity on page and selection;
    /// 3. the editing intentions denote the derived cursor operations;
    /// 4. an intention addressed to one identity leaves the others unobserved.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn intent_laws(&mut self) -> Result<()> {
        let ids = self.author_law_queue(3);
        self.apply_manager_event(ManagerIntentOp::SelectNext {});
        let intended = self.selected_queue_id();
        self.select_relative(false);
        self.select_relative(true);
        if self.selected_queue_id() != intended {
            bail!("the select-next intention does not denote derived collection navigation");
        }

        let selected = self.selected_queue_id();
        self.apply_manager_event(ManagerIntentOp::OpenSelected {});
        if self.page() != ManagerPage::Details(ids[1]) {
            bail!("opening details did not expand the selected identity");
        }
        self.apply_manager_event(ManagerIntentOp::Back {});
        if self.page() != ManagerPage::Collection || self.selected_queue_id() != selected {
            bail!("expanding and returning is not the identity on page and selection");
        }

        self.apply_manager_event(ManagerIntentOp::OpenAddSources {});
        self.apply_manager_event(ManagerIntentOp::InsertText {
            text: "héllo".to_owned(),
        });
        self.apply_manager_event(ManagerIntentOp::MoveCursorLeft {});
        self.apply_manager_event(ManagerIntentOp::DeleteBeforeCursor {});
        if self.draft() != "hélo" {
            bail!(
                "the editing intentions do not denote the derived cursor operations: {}",
                self.draft()
            );
        }

        self.apply_manager_event(ManagerIntentOp::Back {});
        let unaddressed = self.law_entry_shape(ids[2]);
        self.apply_manager_event(ManagerIntentOp::Transfer {
            id: ids[0],
            event: TransferObservationOp::Progress {
                destination: PathBuf::from("law.bin"),
                downloaded: 5,
                total: Some(10),
            },
        });
        if self.law_entry_shape(ids[2]) != unaddressed {
            bail!("an intention addressed to one identity reached another");
        }
        Ok(())
    }

    /// Observes one entry through exactly what the specification exposes about it.
    fn law_entry_shape(&self, id: This::Id) -> Option<(Vec<DetailControl>, Option<SpaceAction>)> {
        self.queue_entry(id)
            .map(|entry| (entry.detail_controls(), entry.space_action()))
    }
}
