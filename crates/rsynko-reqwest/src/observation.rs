use rsynko_memory::DownloadObservationOp;
use std::sync::mpsc::{Receiver, Sender, channel};

/// Carries one pure download observation.
pub type RuntimeObservation = DownloadObservationOp;

/// Sends download observations from the thread that retrieves.
pub type RuntimeObservationSender = Sender<RuntimeObservation>;

/// Receives what one running download states, for whoever attends to it.
pub type RuntimeObservationReceiver = Receiver<RuntimeObservation>;

/// Constructs one observation channel without consuming its receiving end.
///
/// A download states what it does whether or not anybody reads it, so nothing here is asked and
/// nothing is answered: the retrieving thread never waits on whoever is watching.
pub fn runtime_observation_channel() -> (RuntimeObservationSender, RuntimeObservationReceiver) {
    channel()
}
