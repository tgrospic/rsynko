#![doc = include_str!("../README.md")]

mod hold;
mod run;

pub use hold::{HOLDING_IS_POSSIBLE, ProcessHold};
pub use run::{ProcessSyncEnv, ProcessSyncError, SyncRun};
