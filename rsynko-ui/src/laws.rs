//! Law scenarios for the presentation, stated once over the capabilities.
//!
//! A scenario authors its own collection through the manager capabilities it is bound to and
//! reads screens back through the renderer a fixture supplies, so it constrains every renderer
//! rather than the one this workspace happens to ship.

use crate::*;
use alux_ext::ext;
use anyhow::{Result, bail};
use rsynko_manager::*;

/// States one line with every run of spaces reduced to one, so what it says can be compared.
fn squeezed(line: &str) -> String {
    line.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Names the application a presentation scenario states its screens for.
const LAW_APPLICATION: Application<'static> = Application { name: "law", version: "0.0.0" };

/// Supplies the renderer a presentation scenario cannot author for itself.
pub trait ScreenLawFixture {
    /// Represents the renderer whose screens the scenario reads.
    type Syntax;

    /// Supplies the renderer the scenario states its screens through.
    fn law_screen_syntax(&self) -> Self::Syntax;
}

/// Authors the key-binding laws.
#[ext(name = KeyBindingLaws)]
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
    /// Checks that every page binds its keys unambiguously and admits nothing it did not bind.
    ///
    /// The laws checked are:
    ///
    /// 1. no two bindings on one page are reached by the same keystroke;
    /// 2. every keystroke a page binds denotes that binding;
    /// 3. exit is bound on every page, and no other modified keystroke denotes anything;
    /// 4. typed scalars denote insertion exactly on the pages holding a draft;
    /// 5. a gated binding whose action has no meaning leaves the manager as it was.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn key_binding_laws(&mut self) -> Result<()> {
        let ids = self.author_law_queue(2);
        let pages = [
            ManagerPage::Collection,
            ManagerPage::AddSources,
            ManagerPage::Details(ids[0]),
            ManagerPage::Formats(ids[0]),
            ManagerPage::Output(ids[0]),
            ManagerPage::Log(ids[0]),
        ];
        for page in pages {
            self.set_page(page);
            let mut bound = Vec::new();
            for binding in self.page_bindings() {
                for key in binding.keys {
                    if bound.contains(&key) {
                        bail!("{key:?} is bound twice on one page");
                    }
                    bound.push(key);
                    if self.keystroke_meaning(key).is_none() {
                        bail!("a bound keystroke {key:?} denotes nothing");
                    }
                }
            }
            let exit = self.keystroke_meaning(EXIT_KEYSTROKE);
            if exit.and_then(|binding| binding.action) != Some(ManagerAction::Exit) {
                bail!("exit is not bound on every page");
            }
            let modified = Keystroke::control(Key::Character('x'));
            if self.keystroke_meaning(modified).is_some() {
                bail!("a modified keystroke denotes what its unmodified key denotes");
            }
            let typed = self.keystroke_meaning(Keystroke::plain(Key::Character('«')));
            if typed.is_some() != matches!(page, ManagerPage::AddSources | ManagerPage::Output(_)) {
                bail!("a typed scalar denotes insertion off a page holding a draft");
            }
        }
        self.set_page(ManagerPage::Collection);
        self.set_selected_queue_id(None);
        let page = self.page();
        // Removal names a selected entry, so with no selection its key must refuse the intention.
        if self.apply_keystroke(Keystroke::plain(Key::Delete)) {
            bail!("a key applied an action the menu states as unavailable");
        }
        if self.page() != page {
            bail!("a refused keystroke changed the current page");
        }
        Ok(())
    }
}

/// Authors the menu-presentation laws.
#[ext(name = MenuPresentationLaws)]
pub impl<This> This
where
    This: NavigationStateAlg
        + QueueCatalogAlg
        + QueueAppendAlg
        + DraftStateAlg
        + OutputDraftAlg
        + InputDraftAlg
        + TextEditorStateAlg
        + DetailSelectionAlg
        + ManagerStatusAlg
        + SourceRequestAlg
        + SubmissionAlg
        + MediaOptionsAlg
        + OutputChoiceAlg,
    This::Entry: QueueEntryAlg + RequestOptionsAlg,
    This::Id: Copy + Eq,
{
    /// Checks that the menu states what the keys actually do.
    ///
    /// The laws checked are:
    ///
    /// 1. every menu entry states at least one key, and states it in brackets;
    /// 2. every key a menu entry states denotes an action on that page;
    /// 3. every page offers exit, and offers it last;
    /// 4. a menu entry is stated as unavailable exactly when its action has no meaning.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn menu_presentation_laws(&mut self) -> Result<()> {
        let ids = self.author_law_queue(1);
        let pages = [
            ManagerPage::Collection,
            ManagerPage::AddSources,
            ManagerPage::Details(ids[0]),
            ManagerPage::Formats(ids[0]),
            ManagerPage::Output(ids[0]),
            ManagerPage::Log(ids[0]),
        ];
        for page in pages {
            self.set_page(page);
            let items = self.menu_items().collect::<Vec<_>>();
            let Some(last) = items.last() else {
                bail!("a page offers no menu at all");
            };
            if last.action != ManagerAction::Exit {
                bail!("a page does not offer exit last");
            }
            for item in &items {
                let label = item.label();
                if item.keys.is_empty() {
                    bail!("the menu states an entry no key reaches");
                }
                if !label.starts_with('[') || !label.contains("] ") {
                    bail!("a menu entry does not state its keys in brackets: {label}");
                }
                if item.availability != self.action_availability(item.action) {
                    bail!("a menu entry disagrees with what its action means: {label}");
                }
                for key in &item.keys {
                    let bound = self.keystroke_meaning(*key).and_then(|binding| binding.action);
                    let cursor = matches!(bound, Some(ManagerAction::Previous | ManagerAction::Next))
                        && item.action == ManagerAction::Next;
                    if bound != Some(item.action) && !cursor {
                        bail!("the menu states {key:?} for {label}, which denotes {bound:?}");
                    }
                }
            }
        }
        Ok(())
    }
}

/// Authors the progress-gauge laws.
///
/// A gauge is derived from a share and a width alone, so this bundle names no capability: the
/// bounds of a law are the meanings it is derived from, and here there are none.
#[ext(name = GaugeLaws)]
pub impl<This> This {
    /// Checks that a gauge states a share without claiming more resolution than it has.
    ///
    /// The laws checked are:
    ///
    /// 1. a gauge occupies its stated width whatever share it states;
    /// 2. no share is stated as filled before it is reached, and none is stated twice;
    /// 3. nothing stated is nothing filled, and everything stated fills the whole track;
    /// 4. the stated text occupies exactly one cell per counted cell.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn gauge_laws(&self) -> Result<()> {
        let mut previous = 0;
        for percent in 0..=100_u16 {
            let gauge = Gauge::of(percent, GAUGE_WIDTH);
            if gauge.width() != GAUGE_WIDTH {
                bail!("a gauge of {percent}% occupies {} cells", gauge.width());
            }
            let eighths = gauge.filled * Gauge::LEADING.len() + gauge.leading;
            if eighths < previous {
                bail!("a gauge of {percent}% states less than a smaller share");
            }
            previous = eighths;
            if gauge.text().chars().count() != GAUGE_WIDTH {
                bail!("a gauge of {percent}% states {} cells of text", gauge.text());
            }
        }
        let empty = Gauge::of(0, GAUGE_WIDTH);
        if empty.filled != 0 || empty.leading != 0 || empty.track != GAUGE_WIDTH {
            bail!("an unstarted transfer states a filled gauge");
        }
        let full = Gauge::of(100, GAUGE_WIDTH);
        if full.filled != GAUGE_WIDTH || full.track != 0 {
            bail!("a completed transfer states an unfilled gauge");
        }
        Ok(())
    }
}

/// Authors the screen-composition laws.
#[ext(name = ScreenLaws)]
pub impl<This> This
where
    This: NavigationStateAlg
        + QueueCatalogAlg
        + QueueAppendAlg
        + DraftStateAlg
        + OutputDraftAlg
        + InputDraftAlg
        + TextEditorStateAlg
        + DetailSelectionAlg
        + ManagerStatusAlg
        + SourceRequestAlg
        + SubmissionAlg
        + MediaOptionsAlg
        + OutputChoiceAlg
        + QueuePauseAlg
        + TransferStateAlg
        + ScreenLawFixture
        + SubmissionLawFixture,
    This::Entry: QueueEntryAlg
        + RequestOptionsAlg
        + TransferViewAlg
        + FormatChoiceViewAlg<Format: FormatDescriptionAlg>
        + RehearsalViewAlg<Change: PlannedChangeAlg>,
    This::Syntax: ScreenSyntax,
    This::Id: Copy + Eq,
{
    /// Checks that every page states what the manager holds and offers.
    ///
    /// The laws checked are:
    ///
    /// 1. every page names what it belongs to and the pages it rests under, the collection
    ///    excepted, which names itself;
    /// 2. every page states every menu entry it offers;
    /// 3. an empty collection names the key that fills it;
    /// 4. expanded details state every control the entry offers, each exactly once;
    /// 5. a record states every note observed about the request;
    /// 6. the choice a request fixed reads the same where it is chosen and where it is shown;
    /// 7. a request being worked on and one held still do not read alike.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn screen_laws(&mut self) -> Result<()> {
        self.set_page(ManagerPage::Collection);
        let empty = self.law_screen_lines();
        let add = self.action_keys(ManagerAction::AddSources).next().map(Keystroke::label).unwrap_or_default();
        if !empty.iter().any(|line| line.contains(&format!("[{add}]"))) {
            bail!("an empty collection does not name the key that fills it");
        }
        let ids = self.author_law_queue(2);
        for page in [
            ManagerPage::Collection,
            ManagerPage::Details(ids[0]),
            ManagerPage::Formats(ids[0]),
            ManagerPage::Log(ids[0]),
        ] {
            self.set_page(page);
            let lines = self.law_screen_lines();
            // The collection is what every page rests under, and names itself.
            for breadcrumb in self.breadcrumbs().into_iter().skip(1) {
                if !lines.iter().any(|line| line.contains(&breadcrumb.label)) {
                    bail!("a page does not state the path that reached it");
                }
            }
            for item in self.menu_items() {
                if !lines.iter().any(|line| line.contains(&item.label())) {
                    bail!("a page does not state the menu entry it offers: {}", item.label());
                }
            }
        }
        self.set_page(ManagerPage::Details(ids[0]));
        let details = self.law_screen_lines();
        let controls = self.queue_entry(ids[0]).map(QueueEntryAlg::detail_controls).unwrap_or_default();
        for control in controls {
            let label = control.control_label();
            let stated = details.iter().filter(|line| line.contains(label)).count();
            if stated != 1 {
                bail!("expanded details state {label} {stated} times");
            }
        }
        self.set_page(ManagerPage::Log(ids[0]));
        let record = self.law_screen_lines();
        let notes = self
            .queue_entry(ids[0])
            .map(|entry| entry.download_log().map(str::to_owned).collect::<Vec<_>>())
            .unwrap_or_default();
        for note in notes {
            if !record.iter().any(|line| line.contains(&note)) {
                bail!("a record does not state the note {note}");
            }
        }
        self.check_choice_laws()?;
        self.check_phase_laws(ids[0], ids[1])
    }

    /// Checks that one fixed choice reads the same wherever a reader meets it.
    ///
    /// A chooser compares alternatives by reading down a column, and details state one choice as
    /// one phrase, so the two are spaced differently on purpose. What they say is the same.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn check_choice_laws(&mut self) -> Result<()> {
        let (submission, _, _) = self.law_folder_submission();
        self.set_draft(submission);
        self.submit_draft();
        let Some(id) = self.queue_ids().last() else {
            bail!("a submitted transfer was not collected");
        };
        let phrase = {
            let Some(entry) = self.queue_entry(id) else {
                bail!("a submitted transfer was not collected");
            };
            let Some(chosen) = entry.chosen_choice() else {
                bail!("a transfer states no way of transferring");
            };
            let summary = entry.choice_summary(chosen).unwrap_or_default();
            squeezed(&format!("{chosen} {summary}"))
        };
        self.set_page(ManagerPage::Formats(id));
        if !self.law_screen_lines().iter().any(|line| squeezed(line).contains(&phrase)) {
            bail!("a chooser does not state the choice the request fixed: {phrase}");
        }
        self.set_page(ManagerPage::Details(id));
        if !self.law_screen_lines().iter().any(|line| squeezed(line).ends_with(&phrase)) {
            bail!("details state the fixed choice in other words than the chooser does: {phrase}");
        }
        Ok(())
    }

    /// Checks that the collection distinguishes work happening from work held still.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn check_phase_laws(&mut self, running: This::Id, held: This::Id) -> Result<()> {
        self.apply_transfer_event(running, TransferObservationOp::Started {});
        self.apply_transfer_event(held, TransferObservationOp::Started {});
        self.apply_transfer_event(held, TransferObservationOp::PauseCapability { supported: true });
        self.toggle_queue_pause(held);
        self.set_page(ManagerPage::Collection);
        let lines = self.law_screen_lines();
        let marked = |phase: TransferPhase| lines.iter().filter(|line| line.contains(phase.phase_marker())).count();
        if self.queue_entry(held).map(QueueEntryAlg::phase) != Some(TransferPhase::Paused) {
            bail!("a request told to wait was not held");
        }
        if marked(TransferPhase::Downloading) == 0 || marked(TransferPhase::Paused) == 0 {
            bail!("the collection does not mark work happening apart from work held still");
        }
        if TransferPhase::Downloading.phase_marker() == TransferPhase::Paused.phase_marker() {
            bail!("work happening and work held still are marked the same");
        }
        Ok(())
    }

    /// Reads the current page back as the lines it states.
    fn law_screen_lines(&self) -> Vec<String> {
        let syntax = self.law_screen_syntax();
        let screen = self.screen(&syntax, LAW_APPLICATION);
        syntax.screen_text(&screen).collect()
    }
}
