// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::{Arc, Weak};

use tokio::sync::{watch, Barrier};
use tokio_util::sync::CancellationToken;
use tracing::trace;

#[derive(Debug, Clone)]
pub struct LifecycleManager {
    current_instance: (CancellationToken, Arc<Barrier>),
    previous_instance: Option<(CancellationToken, Arc<Barrier>)>,
    /// NOTE: Set to [`None`] to close the channel, effectively
    ///   stopping restarts after the API gracefully shuts down.
    restart_tx: Option<Arc<watch::Sender<bool>>>,
    /// NOTE: Receives `true` every time the API should restart.
    ///   Receives `false` once the API has restarted.
    restart_rx: watch::Receiver<bool>,
}

impl LifecycleManager {
    fn new_instance() -> (CancellationToken, Arc<Barrier>) {
        (CancellationToken::new(), Arc::new(Barrier::new(2)))
    }
}

impl LifecycleManager {
    pub fn new() -> Self {
        let (restart_tx, restart_rx) = watch::channel(false);
        Self {
            current_instance: Self::new_instance(),
            previous_instance: None,
            restart_tx: Some(Arc::new(restart_tx)),
            restart_rx,
        }
    }

    pub fn current_instance(&self) -> (CancellationToken, Arc<Barrier>) {
        self.current_instance.clone()
    }

    pub fn restart_tx(&self) -> Weak<watch::Sender<bool>> {
        self.restart_tx
            .as_ref()
            .map(Arc::downgrade)
            .unwrap_or_default()
    }
    pub fn restart_rx_mut(&mut self) -> &mut watch::Receiver<bool> {
        &mut self.restart_rx
    }

    pub fn set_restarting(&self) {
        if let Some(ref restart_tx) = self.restart_tx {
            restart_tx.send_modify(|restarting| *restarting = true);
        }
    }
    pub fn is_restarting(&self) -> bool {
        *self.restart_rx.borrow()
    }
    pub fn set_restart_finished(&self) {
        if let Some(ref restart_tx) = self.restart_tx {
            restart_tx.send_modify(|restarting| *restarting = false);
        }
    }

    pub fn rotate_instance(self) -> Self {
        Self {
            current_instance: Self::new_instance(),
            previous_instance: Some(self.current_instance),
            restart_tx: self.restart_tx,
            restart_rx: self.restart_rx,
        }
    }

    /// Stop the previous instance, if applicable.
    pub async fn stop_previous_instance(&self) {
        if let Some((running, stopped)) = self.previous_instance.as_ref() {
            // Stop the previous instance.
            trace!("Stopping the previous instance…");
            running.cancel();
            // Wait for previous instance to be stopped (for port to be available).
            trace!("Waiting for previous instance to be stopped…");
            stopped.wait().await;
        }
    }
}
