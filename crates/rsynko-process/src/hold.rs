use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

/// Holds one running transfer still, and lets it go again.
///
/// A transfer performed by another program cannot be asked to wait: it is a process, and the way
/// a process is held still is a signal. This carries the identity of the running one, so whoever
/// is watching can hold it without owning it.
#[derive(Clone, Debug, Default)]
pub struct ProcessHold {
    running: Arc<AtomicU32>,
}

/// Names the signal holding a running transfer still.
const HOLD_SIGNAL: &str = "STOP";

/// Names the signal letting a held transfer go again.
const RELEASE_SIGNAL: &str = "CONT";

/// Names the signal ending a transfer that is no longer wanted.
const END_SIGNAL: &str = "TERM";

/// Names the program that sends a signal to a process.
const SIGNAL_PROGRAM: &str = "kill";

impl ProcessHold {
    /// States which process is running, or that none is.
    pub(crate) fn running(&self, process: Option<u32>) {
        self.running.store(process.unwrap_or_default(), Ordering::SeqCst);
    }

    /// Observes which process is running the transfer, while one is.
    #[must_use]
    pub fn process(&self) -> Option<u32> {
        let running = self.running.load(Ordering::SeqCst);
        (running != 0).then_some(running)
    }

    /// Holds the running transfer still, and observes whether one was held.
    pub fn hold(&self) -> bool {
        self.signalled(HOLD_SIGNAL)
    }

    /// Lets a held transfer go again, and observes whether one was let go.
    pub fn release(&self) -> bool {
        self.signalled(RELEASE_SIGNAL)
    }

    /// Ends the running transfer, and observes whether one was ended.
    ///
    /// A held transfer is let go first: a process that is not running cannot notice that it was
    /// asked to stop.
    pub fn cancel(&self) -> bool {
        let _released = self.signalled(RELEASE_SIGNAL);
        self.signalled(END_SIGNAL)
    }

    /// Sends one signal to the whole of the running transfer.
    ///
    /// The transfer is a family: a sender, a receiver, and whatever carries between them. The
    /// signal names the family rather than its head, so none of it is left running alone.
    fn signalled(&self, signal: &str) -> bool {
        let running = self.running.load(Ordering::SeqCst);
        if running == 0 || !HOLDING_IS_POSSIBLE {
            return false;
        }
        // The family is named by a negative number, which has to be stated as an argument rather
        // than read as an option of its own.
        Command::new(SIGNAL_PROGRAM)
            .args(["-s", signal, "--", &format!("-{running}")])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }
}

/// States whether a transfer can be held still on this machine at all.
pub const HOLDING_IS_POSSIBLE: bool = cfg!(unix);
