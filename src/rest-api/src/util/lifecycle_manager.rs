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
    master_cancellation_token: CancellationToken,
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
    fn new_instance(child_token: CancellationToken) -> (CancellationToken, Arc<Barrier>) {
        (child_token, Arc::new(Barrier::new(2)))
    }
}

impl LifecycleManager {
    pub fn new() -> Self {
        let master_cancellation_token = CancellationToken::new();
        let (restart_tx, restart_rx) = watch::channel(false);
        Self {
            current_instance: Self::new_instance(master_cancellation_token.child_token()),
            master_cancellation_token,
            previous_instance: None,
            restart_tx: Some(Arc::new(restart_tx)),
            restart_rx,
        }
    }

    pub fn current_instance(&self) -> (CancellationToken, Arc<Barrier>) {
        self.current_instance.clone()
    }

    pub fn restart_tx(&self) -> Weak<watch::Sender<bool>> {
        (self.restart_tx.as_ref()).map_or(Default::default(), Arc::downgrade)
    }
    pub fn restart_rx_mut(&mut self) -> &mut watch::Receiver<bool> {
        &mut self.restart_rx
    }
    pub async fn should_restart(&mut self) -> bool {
        tokio::select! {
            _ = self.master_cancellation_token.cancelled() => false,
            res = async {
                while self.restart_rx.changed().await.is_ok() {
                    let initiate_restart = *self.restart_rx.borrow();
                    trace!("restart_rx changed: {initiate_restart}");
                    if initiate_restart {
                        return true
                    }
                }
                false
            } => res,
        }
    }
    pub fn will_restart(&self) -> bool {
        let will_stop = (self.restart_tx.as_ref()).map_or(true, |tx| tx.is_closed());
        !will_stop
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
            current_instance: Self::new_instance(self.master_cancellation_token.child_token()),
            master_cancellation_token: self.master_cancellation_token,
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

    pub fn child_cancellation_token(&self) -> CancellationToken {
        self.master_cancellation_token.child_token()
    }
}

impl LifecycleManager {
    /// Source: [`axum/examples/graceful-shutdown/src/main.rs#L55-L77`](https://github.com/tokio-rs/axum/blob/ef0b99b6a01e083101fe2e78e6a9c17e3708bc3c/examples/graceful-shutdown/src/main.rs#L55-L77)
    ///
    /// NOTE: Graceful shutdown will wait for outstanding requests to complete.
    ///   We have SSE routes so we can't add a timeout like suggested in
    ///   [`axum/examples/graceful-shutdown/src/main.rs#L40-L42`](https://github.com/tokio-rs/axum/blob/ef0b99b6a01e083101fe2e78e6a9c17e3708bc3c/examples/graceful-shutdown/src/main.rs#L40-L42).
    ///   We'll find a solution if it ever becomes a problem.
    pub async fn listen_for_graceful_shutdown(&mut self) {
        use tokio::signal;
        use tracing::warn;

        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                warn!("Received Ctrl+C.")
            },
            _ = terminate => {
                warn!("Process terminated.")
            },
        }

        // Cancel all running instances and break infinite main loop.
        self.master_cancellation_token.cancel();
    }
}
