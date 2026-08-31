use crate::*;
use alux_ext::ext;
use alux_sdk::case_mapping;
use ambassador::delegatable_trait;

/// Provides the carrier and constructor for exactly the command one transfer runs.
///
/// The specification owns the command because the command *is* the request: an interpreter runs
/// it and moves its output, and never decides what a transfer should ask for.
#[delegatable_trait]
pub trait SyncCommandAlg: RsyncSorts {
    /// Defines the command running one program with exactly these arguments, in this order.
    fn sync_command(
        &self,
        program: impl Into<String>,
        arguments: impl IntoIterator<Item = String>,
    ) -> Self::Command;
}

/// Specifies what one command states about itself.
#[delegatable_trait]
pub trait SyncCommandViewAlg: RsyncSorts {
    /// Observes the program the transfer is performed by.
    fn command_program<'a>(&self, command: &'a Self::Command) -> &'a str;
    /// Observes every argument, in the order the program is given them.
    fn command_arguments<'a>(&self, command: &'a Self::Command) -> impl Iterator<Item = &'a str>;
}

/// Names the program a transfer is performed by.
pub const SYNC_PROGRAM: &str = "rsync";

/// States each transferred path as its change, its byte count, and its name.
pub const SYNC_OUT_FORMAT: &str = "--out-format=%i|%l|%n";

/// States that the transfer changes nothing and only says what it would do.
pub const SYNC_REHEARSAL: &str = "--dry-run";

/// States that the destination is made to match the source exactly.
pub const SYNC_MIRROR: &str = "--delete";

/// Derives the command one folder transfer runs from the ends it joins.
#[ext(name = SyncCommandExt)]
pub impl<This> This
where
    This: SyncCommandAlg + SyncCommandViewAlg + RsyncEndpointAlg + RsyncEndpointViewAlg,
{
    /// States the command transferring one endpoint into another.
    ///
    /// What is transferred may be one file or a whole folder, and each end is stated exactly as
    /// it was written. A source ending with a separator names the contents of a folder; one
    /// without names the path itself, which comes to rest inside the destination. Both are
    /// wanted, so neither is invented here.
    fn transfer_command(
        &self,
        source: &This::Endpoint,
        destination: &This::Endpoint,
        mode: SyncMode,
    ) -> This::Command {
        let arguments = [
            // Everything a folder states about itself is part of the folder.
            "--archive",
            // An interrupted transfer resumes rather than starting the file again.
            "--partial",
            // Stated twice, so the transfer also names what it would leave exactly as it is.
            "--itemize-changes",
            "--itemize-changes",
            SYNC_OUT_FORMAT,
            "--info=progress2",
        ]
        .into_iter()
        .map(str::to_owned)
        .chain(
            mode.profile
                .profile_arguments()
                .iter()
                .map(|argument| (*argument).to_owned()),
        )
        .chain(mode.rehearsal.then(|| SYNC_REHEARSAL.to_owned()))
        .chain([self.endpoint_text(source), self.endpoint_text(destination)]);
        self.sync_command(SYNC_PROGRAM, arguments)
    }

    /// Observes whether the command states what it would do instead of doing it.
    fn command_rehearses(&self, command: &This::Command) -> bool {
        self.command_arguments(command)
            .any(|argument| argument == SYNC_REHEARSAL)
    }

    /// Observes whether the command removes what the source no longer holds.
    fn command_mirrors(&self, command: &This::Command) -> bool {
        self.command_arguments(command)
            .any(|argument| argument == SYNC_MIRROR)
    }
}

/// Selects what one transfer is allowed to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyncMode {
    /// Selects the way the path is transferred.
    pub profile: SyncProfile,
    /// States what the transfer would do instead of doing it.
    pub rehearsal: bool,
}

impl SyncMode {
    /// Selects a transfer that states what it would do and changes nothing.
    #[must_use]
    pub const fn rehearsal() -> Self {
        Self {
            profile: SyncProfile::Copy,
            rehearsal: true,
        }
    }

    /// Selects a transfer that adds and replaces, and removes nothing.
    #[must_use]
    pub const fn transfer() -> Self {
        Self {
            profile: SyncProfile::Copy,
            rehearsal: false,
        }
    }

    /// Selects whether the transfer states what it would do instead of doing it.
    #[must_use]
    pub const fn rehearsed(mut self, rehearsal: bool) -> Self {
        self.rehearsal = rehearsal;
        self
    }

    /// Selects the way the path is transferred.
    #[must_use]
    pub const fn profiled(mut self, profile: SyncProfile) -> Self {
        self.profile = profile;
        self
    }
}

/// Selects one commonly wanted way of transferring a path.
///
/// A transfer has a great many arguments and almost nobody wants an arbitrary combination of
/// them: what people want is one of a handful of familiar jobs. Each of these names what it
/// *does* — not what it is for — and states exactly the arguments that make it do that.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncProfile {
    /// Adds and replaces, and removes nothing.
    Copy,
    /// Makes the destination match the source exactly.
    Mirror,
    /// Mirrors, keeping what it replaces and compressing what it sends.
    MirrorKeeping,
    /// Mirrors by sending whole files rather than comparing them piece by piece.
    MirrorWhole,
    /// Mirrors, and makes everything readable where it lands.
    MirrorReadable,
    /// Replaces nothing that is newer at the destination.
    SkipNewer,
    /// Compares what files hold rather than when they changed.
    CompareContent,
    /// Continues large files where an interrupted transfer left them.
    Resume,
    /// Moves, removing from the source whatever arrived safely.
    Move,
    /// Leaves room on the line for everything else.
    LimitRate,
    /// Keeps links, ownership, and every other mark a file carries.
    KeepMarks,
    /// Stays on the disk it started on.
    OneDisk,
    /// Leaves behind what no one meant to keep.
    SkipJunk,
}

case_mapping! {
    SyncProfile, &'static str as &str,
        Copy           <=> "copy",
        Mirror         <=> "mirror",
        MirrorKeeping  <=> "mirror-keeping",
        MirrorWhole    <=> "mirror-whole",
        MirrorReadable <=> "mirror-readable",
        SkipNewer      <=> "skip-newer",
        CompareContent <=> "compare-content",
        Resume         <=> "resume",
        Move           <=> "move",
        LimitRate      <=> "limit-rate",
        KeepMarks      <=> "keep-marks",
        OneDisk        <=> "one-disk",
        SkipJunk       <=> "skip-junk",
}

impl SyncProfile {
    /// States what the transfer does, and then what that is usually wanted for.
    ///
    /// A reader chooses by what a way of transferring does, and confirms the choice by what it is
    /// good for, so both are stated and in that order.
    #[must_use]
    pub const fn summary(self) -> &'static str {
        match self {
            Self::Copy => "adds and replaces, removes nothing, good for topping a folder up",
            Self::Mirror => "makes the destination match exactly, good for keeping a copy current",
            Self::MirrorKeeping => {
                "mirrors, keeps what it replaces, compresses, good for remote backups"
            }
            Self::MirrorWhole => "mirrors by sending whole files, good for local disks and SSDs",
            Self::MirrorReadable => {
                "mirrors and makes everything readable, good for publishing a web root"
            }
            Self::SkipNewer => {
                "replaces nothing newer at the destination, good for merging two copies"
            }
            Self::CompareContent => {
                "compares what files hold, not when they changed, good after clock drift"
            }
            Self::Resume => "continues large files where they left off, good for flaky links",
            Self::Move => "moves, removing what arrived safely, good for clearing a staging area",
            Self::LimitRate => "leaves room on the line, good for a shared connection",
            Self::KeepMarks => {
                "keeps links, ownership, and every file mark, good for system backups"
            }
            Self::OneDisk => "stays on the disk it started on, good for roots with mounts under",
            Self::SkipJunk => "leaves behind what no one meant to keep, good for editor droppings",
        }
    }

    /// States exactly the arguments that make the transfer this job and no other.
    #[must_use]
    pub const fn profile_arguments(self) -> &'static [&'static str] {
        match self {
            Self::Copy => &[],
            Self::Mirror => &["--delete"],
            Self::MirrorKeeping => &["--compress", "--delete", "--backup"],
            Self::MirrorWhole => &["--whole-file", "--delete"],
            Self::MirrorReadable => &["--delete", "--chmod=D755,F644"],
            Self::SkipNewer => &["--update"],
            Self::CompareContent => &["--checksum"],
            Self::Resume => &["--inplace", "--append-verify"],
            Self::Move => &["--remove-source-files"],
            Self::LimitRate => &["--bwlimit=2000"],
            Self::KeepMarks => &["--hard-links", "--acls", "--xattrs"],
            Self::OneDisk => &["--one-file-system"],
            Self::SkipJunk => &[
                "--exclude=.DS_Store",
                "--exclude=Thumbs.db",
                "--exclude=*.tmp",
            ],
        }
    }
}
