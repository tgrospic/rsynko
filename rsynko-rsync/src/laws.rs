//! Law scenarios for folder transfers, stated once over the capabilities.
//!
//! A scenario authors the endpoints and the lines it reasons about through the vocabulary it is
//! bound to, so it constrains any interpreter of the transfer sorts rather than the one this
//! workspace happens to ship.

use crate::*;
use alux_ext::ext;
use anyhow::{Result, bail};
use rsynko_manager::{ChangeKind, PlannedChangeAlg};

/// Supplies the running transfer a scenario cannot author for itself.
pub trait SyncLawFixture: RsyncSorts {
    /// Makes the next transfer state exactly these lines, whatever command it is given.
    fn law_transcript(&mut self, lines: Vec<String>);

    /// Observes whether one command was the last the interpreter was asked to run.
    fn law_ran(&self, command: &Self::Command) -> bool;
}

/// Authors the folder-transfer laws.
#[ext(name = SyncLaws)]
pub impl<This> This
where
    This: RsyncEndpointAlg
        + RsyncEndpointViewAlg
        + SyncCommandAlg
        + SyncCommandViewAlg
        + SyncChangeAlg
        + SyncObservationAlg
        + SyncObservationViewAlg
        + SyncRunAlg
        + SyncWatchAlg
        + SyncLawFixture,
    This::Change: PlannedChangeAlg + Clone,
{
    /// Checks that an endpoint, a command, and a stated line each mean one thing.
    ///
    /// The laws checked are:
    ///
    /// 1. an endpoint states back exactly the text it was read from;
    /// 2. a path on another machine is remote, and neither a path here nor a text naming another
    ///    scheme is;
    /// 3. a line naming two ends states both, however it is spaced and whatever else it names;
    /// 3. a rehearsal command states that it changes nothing, and a transfer does not;
    /// 4. a mirroring command states that it removes, and one that only adds does not;
    /// 5. every itemized shape states the change it denotes, and progress is not a change;
    /// 6. each end is transferred exactly as it was written;
    /// 7. running a transfer states every change its lines named, in the order they arrived;
    /// 8. every way of transferring is named by its own word, and says what it does.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn sync_laws(&mut self) -> Result<()> {
        self.check_endpoint_laws()?;
        self.check_command_laws()?;
        self.check_reading_laws()?;
        self.check_run_laws()
    }

    /// Checks what an endpoint states about the path and the machine it names.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn check_endpoint_laws(&self) -> Result<()> {
        for text in [
            "backup@nas.local:/volume1/photos",
            "nas.local:/volume1/photos",
            "/home/dev/photos",
        ] {
            let endpoint = self.read_endpoint(text);
            let stated = self.endpoint_text(&endpoint);
            if stated != text {
                bail!("{text} states itself back as {stated}");
            }
            if self.endpoint_remote(&endpoint) != text.contains(':') {
                bail!("{text} disagrees about which machine it rests on");
            }
        }
        for elsewhere in [
            "https://www.youtube.com/watch?v=VIDEO_ID",
            "fixture://single-video",
        ] {
            if self.endpoint_remote(&self.read_endpoint(elsewhere)) {
                bail!("{elsewhere} names a resource, and was read as a path on a machine");
            }
        }
        let daemon = self.read_endpoint("rsync://nas.local/volume1/photos");
        if !self.endpoint_remote(&daemon) || self.endpoint_path(&daemon) != "/volume1/photos" {
            bail!("a daemon endpoint does not name the path it states");
        }
        if self.endpoint_name(&self.read_endpoint("/home/dev/photos/2026/")) != "2026" {
            bail!("an endpoint does not name what its path ends with");
        }
        if self.endpoint_name(&self.read_endpoint("nas.local:/srv/report.pdf")) != "report.pdf" {
            bail!("an endpoint naming one file does not name that file");
        }
        self.check_transfer_laws()
    }

    /// Checks what a line naming two ends states about the transfer between them.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn check_transfer_laws(&self) -> Result<()> {
        let from = "backup@nas.local:/volume1/photos";
        let into = "/home/dev/photos";
        // A transfer is written the way it is run, and a whole command is written the same way.
        for line in [
            format!("{from} {into}"),
            format!("  {from}   {into}  "),
            format!("rsync -a --dry-run {from} {into}"),
        ] {
            let Some((source, destination)) = self.read_transfer(&line) else {
                bail!("{line} names two ends, and was read as naming none");
            };
            if self.endpoint_text(&source) != from || self.endpoint_text(&destination) != into {
                bail!("{line} states its ends back as something else");
            }
        }
        for line in [from, "", "one two three"] {
            if self.read_transfer(line).is_some() {
                bail!("{line} does not name two ends, and was read as naming two");
            }
        }
        Ok(())
    }

    /// Checks what a command states about what the transfer is allowed to do.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn check_command_laws(&self) -> Result<()> {
        let source = self.read_endpoint("nas.local:/volume1/photos");
        let destination = self.read_endpoint("/home/dev/photos");
        let rehearsal = self.transfer_command(&source, &destination, SyncMode::rehearsal());
        let transfer = self.transfer_command(&source, &destination, SyncMode::transfer());
        if !self.command_rehearses(&rehearsal) {
            bail!("a rehearsal does not state that it changes nothing");
        }
        if self.command_rehearses(&transfer) {
            bail!("a transfer states that it changes nothing");
        }
        let mirror = self.transfer_command(
            &source,
            &destination,
            SyncMode::transfer().profiled(SyncProfile::Mirror),
        );
        if !self.command_mirrors(&mirror) {
            bail!("a mirroring transfer does not state that it removes");
        }
        if self.command_mirrors(&transfer) {
            bail!("a transfer that only adds states that it removes");
        }
        // Every way of transferring is one job, stated as exactly the arguments that make it one.
        for (place, profile) in sync_profile::ALL.iter().copied().enumerate() {
            let named = sync_profile::to(profile);
            let stated = self.transfer_command(
                &source,
                &destination,
                SyncMode::transfer().profiled(profile),
            );
            for argument in profile.profile_arguments() {
                if !self
                    .command_arguments(&stated)
                    .any(|stated| stated == *argument)
                {
                    bail!("{named} does not ask for {argument}");
                }
            }
            if sync_profile::from(named) != Some(profile) {
                bail!("{named} is not read back as itself");
            }
            // What each way does is stated beside the variant, and the compiler states that every
            // variant states it. That each is named by its own word it cannot state, and a shared
            // word would leave one of them unreachable.
            if sync_profile::ALL[..place]
                .iter()
                .any(|earlier| sync_profile::to(*earlier) == named)
            {
                bail!("two ways of transferring are named {named}");
            }
            if profile.summary().is_empty() {
                bail!("{named} does not say what it does");
            }
        }
        if sync_profile::from("nothing anybody would call a transfer").is_some() {
            bail!("a word naming no way of transferring names one");
        }
        if self.command_arguments(&transfer).last()
            != Some(self.endpoint_text(&destination).as_str())
        {
            bail!("a transfer does not end by naming where it writes");
        }
        // A trailing separator says "the contents of", and its absence says "this, into": both
        // are wanted, so each end is stated exactly as it was written.
        for written in ["/home/dev/photos", "/home/dev/photos/"] {
            let end = self.read_endpoint(written);
            let stated = self.transfer_command(&end, &destination, SyncMode::transfer());
            if !self
                .command_arguments(&stated)
                .any(|argument| argument == written)
            {
                bail!("{written} is not transferred as it was written");
            }
        }

        Ok(())
    }

    /// Checks what one line a running transfer wrote states.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn check_reading_laws(&self) -> Result<()> {
        for (line, expected) in [
            (">f+++++++++|3284921|IMG_0431.jpg", Some(ChangeKind::Create)),
            (">f.st......|1204|album.json", Some(ChangeKind::Update)),
            ("cd+++++++++|0|old/", Some(ChangeKind::Create)),
            (
                ".f         |2400000|IMG_0001.jpg",
                Some(ChangeKind::Unchanged),
            ),
            ("*deleting  |0|old/IMG_0090.jpg", Some(ChangeKind::Delete)),
        ] {
            let observation = self.read_sync_line(line);
            let stated = self
                .observation_change(&observation)
                .map(PlannedChangeAlg::change_kind);
            if stated != expected {
                bail!("{line} states {stated:?} rather than {expected:?}");
            }
        }
        let advanced = self.read_sync_line("     19,084,083  28%    2.02MB/s    0:00:22");
        if self.observation_progress(&advanced) != Some((19_084_083, 28)) {
            bail!("a progress line does not state how far the transfer has come");
        }
        let unread = self.read_sync_line("sending incremental file list");
        if self.observation_change(&unread).is_some()
            || self.observation_progress(&unread).is_some()
        {
            bail!("a line naming nothing is read as something");
        }
        Ok(())
    }

    /// Checks that running a transfer states the changes its lines named.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn check_run_laws(&mut self) -> Result<()> {
        let rehearsal = self.transfer_command(
            &self.read_endpoint("nas.local:/volume1/photos"),
            &self.read_endpoint("/home/dev/photos"),
            SyncMode::rehearsal(),
        );

        self.law_transcript(vec![
            "sending incremental file list".to_owned(),
            ">f+++++++++|4|new.txt".to_owned(),
            "     19,084,083  28%    2.02MB/s    0:00:22".to_owned(),
            "*deleting  |0|gone.txt".to_owned(),
        ]);
        let Ok(changes) = self.run_sync(&rehearsal) else {
            bail!("the rehearsal refused to run");
        };
        let named = changes
            .iter()
            .map(|change| (change.change_path().to_owned(), change.change_kind()))
            .collect::<Vec<_>>();
        if named
            != vec![
                ("new.txt".to_owned(), ChangeKind::Create),
                ("gone.txt".to_owned(), ChangeKind::Delete),
            ]
        {
            bail!("a run states {named:?} rather than the changes its lines named");
        }
        if !self.law_ran(&rehearsal) {
            bail!("a run does not run the command it was given");
        }
        Ok(())
    }
}
