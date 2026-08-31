# rsynko-rsync

`rsynko-rsync` defines the transfer of one path from one place to another. That path may be a single file or a whole folder, and the difference is not this specification's business: a folder is one thing, not a collection of files that happen to travel together. Either way it is named at both ends, it is stated before it is performed, and it states back exactly what it would do.

The specification owns the command, including every argument, because the command *is* the request. An interpreter starts it, hands back what it wrote, and reports how it ended; it never decides what a transfer should ask for.

## Specification surface

[`RsyncSorts`] states the type families every class shares: an endpoint, a command, one thing a running transfer states, and one path it would change. An interpreter ties all four at once and no class re-declares any of them.

An endpoint is defined by what it names and observed through what it states:

- [`RsyncEndpointAlg::endpoint`] defines one from an account, a machine, and a path;
- [`RsyncEndpointViewAlg`] observes those three back;
- `RsyncEndpointExt::read_endpoint` derives one from submitted text, and `endpoint_text` states it back the way the transfer program is given it.

Reading and stating are inverse, which is the law that keeps a submitted source recognizable after it has been through the vocabulary.

Only three shapes name a path on another machine: a daemon URL, and a machine and a path separated by a colon with nothing path-like before it. A text naming any other scheme names a resource somewhere else, which is what keeps a media URL from being read as a path to transfer.

Composing those meanings is an extension, and its bounds are the specification. Nothing here names a record, a program, or a process:

```rust
use alux_ext::ext;
use rsynko_rsync::{
    RsyncEndpointAlg, RsyncEndpointExt, RsyncEndpointViewAlg, SyncCommandAlg, SyncCommandExt,
    SyncCommandViewAlg, SyncMode,
};

#[ext(name = ExampleFolderExt)]
impl<This> This
where
    This: RsyncEndpointAlg + RsyncEndpointViewAlg + SyncCommandAlg + SyncCommandViewAlg,
{
    /// States the rehearsal of one submitted path into the place it would come to rest.
    fn example_rehearsal(&self, submitted: &str, into: &str) -> Option<This::Command> {
        let source = self.read_endpoint(submitted);
        // A path already here is not something somebody asked to be brought here.
        self.endpoint_remote(&source).then(|| {
            self.transfer_command(&source, &self.read_endpoint(into), SyncMode::rehearsal())
        })
    }

    /// Names what a submitted source would arrive as, under a chosen parent.
    fn example_destination(&self, submitted: &str, parent: &str) -> String {
        format!(
            "{parent}/{}",
            self.endpoint_name(&self.read_endpoint(submitted))
        )
    }
}
```

[`SyncCommandAlg::sync_command`] defines the command; [`SyncCommandViewAlg`] observes the program and its arguments. `SyncCommandExt::transfer_command` derives the whole command from the two ends and a [`SyncMode`], and `command_rehearses` and `command_mirrors` observe what the derived command is allowed to do. Each end is stated exactly as it was written: a source ending with a separator names the contents of a folder, and one without names the path itself, which comes to rest inside the destination. Both are wanted, so neither is invented.

A transfer has a great many arguments and almost nobody wants an arbitrary combination of them: what people want is one of a handful of familiar jobs. [`SyncProfile`] names thirteen, each after what it *does* rather than what it is for, and each states exactly the arguments that make it do that — `mirror` asks for `--delete`, `mirror-keeping` for `--compress --delete --backup`, `mirror-whole` for `--whole-file --delete`, `compare-content` for `--checksum`, `resume` for `--inplace --append-verify`. `SyncProfile::key` names one, `summary` says what it does and then what that is usually good for, and `SyncProfile::read` reads a name back, so a chosen way survives being written down.

A running transfer states two kinds of thing, and a great many lines that are neither. [`SyncObservationAlg`] defines all three, [`SyncObservationViewAlg`] observes which one a line turned out to be, and `SyncReadExt::read_sync_line` derives that from what the program wrote. A changed path is defined by [`SyncChangeAlg::sync_change`] and observed through the manager's own `PlannedChangeAlg`, so what a transfer says it would do and what a reader is shown are the same statement.

## Running one

[`SyncRunAlg`] is the whole of what an interpreter supplies: start a command, read the next line, and state how it ended. [`SyncWatchAlg`] states what to do with each observation as it arrives. `SyncProgramExt::run_sync` composes them into one transfer that reports progress while it runs and states every change it named when it ends.

The command carries the rehearsal, so rehearsing and transferring are one program run two ways rather than two programs. What a caller does with the result is another extension over the same bounds:

```rust
use alux_ext::ext;
use rsynko_manager::{ChangeKind, PlannedChangeAlg};
use rsynko_rsync::{
    SyncChangeAlg, SyncObservationAlg, SyncObservationViewAlg, SyncProgramExt, SyncRunAlg,
    SyncWatchAlg,
};

#[ext(name = ExampleReportExt)]
impl<This> This
where
    This: SyncRunAlg + SyncWatchAlg + SyncChangeAlg + SyncObservationAlg + SyncObservationViewAlg,
    This::Change: PlannedChangeAlg + Clone,
{
    /// Counts what one transfer would alter, and what it would leave exactly as it is.
    ///
    /// # Errors
    ///
    /// Returns the failure that ran the transfer.
    fn example_report(&self, command: &This::Command) -> Result<(usize, usize), This::Error> {
        let changes = self.run_sync(command)?;
        let altered = changes
            .iter()
            .filter(|change| change.change_kind().alters())
            .count();
        Ok((altered, changes.len() - altered))
    }

    /// Observes whether one transfer would remove anything at the destination.
    ///
    /// # Errors
    ///
    /// Returns the failure that ran the transfer.
    fn example_removes(&self, command: &This::Command) -> Result<bool, This::Error> {
        Ok(self
            .run_sync(command)?
            .iter()
            .any(|change| change.change_kind() == ChangeKind::Delete))
    }
}
```

A reference interpreter tying the four sorts to inspectable records lives in `rsynko-memory`, and `rsynko-process` supplies the two capabilities that actually run one.

## Laws

`SyncLaws::sync_laws` checks that an endpoint states back the text it was read from and agrees about which machine it rests on; that a rehearsal states it changes nothing while a transfer does not, and a mirroring transfer states it removes while one that only adds does not; that every itemized shape states the change it denotes and progress is not one; and that running a transfer states every change its lines named, in the order they arrived. `SyncLawFixture` supplies what a scenario cannot author for itself: the lines the next transfer will state.
