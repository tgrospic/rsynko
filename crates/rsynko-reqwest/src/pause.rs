use std::sync::{Arc, Condvar, Mutex};

/// Controls cooperative suspension of one streaming download.
#[derive(Clone, Debug)]
pub struct RuntimePause {
    state: Arc<(Mutex<RuntimeControlState>, Condvar)>,
}

#[derive(Debug)]
struct RuntimeControlState {
    paused: bool,
    cancelled: bool,
}

impl RuntimePause {
    /// Constructs a running transfer control.
    #[must_use]
    pub fn running() -> Self {
        Self {
            state: (
                Mutex::new(RuntimeControlState {
                    paused: false,
                    cancelled: false,
                }),
                Condvar::new(),
            )
                .into(),
        }
    }

    /// Sets whether retrieval waits before reading its next chunk.
    pub fn set_paused(&self, paused: bool) {
        let (state, changed) = &*self.state;
        state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .paused = paused;
        if !paused {
            changed.notify_all();
        }
    }

    /// Cancels retrieval and wakes a suspended reader.
    pub fn cancel(&self) {
        let (state, changed) = &*self.state;
        state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .cancelled = true;
        changed.notify_all();
    }

    pub(crate) fn wait_until_running(&self) -> bool {
        let (state, changed) = &*self.state;
        let mut paused = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        while paused.paused && !paused.cancelled {
            paused = changed
                .wait(paused)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
        !paused.cancelled
    }
}

#[cfg(test)]
mod tests {
    use super::RuntimePause;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn paused_reader_waits_until_resume() {
        let pause = RuntimePause::running();
        pause.set_paused(true);
        let waiting = pause.clone();
        let (sender, receiver) = mpsc::channel();
        let worker = thread::spawn(move || {
            waiting.wait_until_running();
            sender.send(()).expect("observer remains connected");
        });

        assert!(receiver.recv_timeout(Duration::from_millis(20)).is_err());
        pause.set_paused(false);
        receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("reader resumed");
        worker.join().expect("pause worker");
    }

    #[test]
    fn cancellation_wakes_a_paused_reader() {
        let pause = RuntimePause::running();
        pause.set_paused(true);
        let waiting = pause.clone();
        let (sender, receiver) = mpsc::channel();
        let worker = thread::spawn(move || {
            sender
                .send(waiting.wait_until_running())
                .expect("observer remains connected");
        });

        pause.cancel();
        assert!(
            !receiver
                .recv_timeout(Duration::from_secs(1))
                .expect("cancelled")
        );
        worker.join().expect("pause worker");
    }
}
