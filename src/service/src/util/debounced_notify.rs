// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Instant};

// NOTE: See `playgrounds/tokio-debounce` for a testing playground.
/// Allows debouncing events (i.e. propagating a signal only after some time has
/// passed).
#[derive(Debug, Clone)]
pub struct DebouncedNotify {
    notify: Arc<Notify>,
}

impl DebouncedNotify {
    pub fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn notify(&self) {
        self.notify.notify_waiters();
    }

    pub fn listen_debounced(
        &self,
        delay: Duration,
        callback: impl Fn(Instant) + Send + 'static,
    ) -> JoinHandle<()> {
        let notify = self.notify.clone();
        tokio::spawn(async move {
            let mut last_signal: Option<Instant> = None;
            loop {
                tokio::select! {
                    _ = notify.notified() => {
                        last_signal = Some(Instant::now());
                    }
                    _ = sleep(delay), if last_signal.is_some_and(|i| i.elapsed() < delay) => {
                        callback(last_signal.unwrap());
                    }
                }
            }
        })
    }
}
