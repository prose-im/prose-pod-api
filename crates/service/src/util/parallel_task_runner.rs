// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{cmp::min, future::Future, time::Duration};

use futures::{
    stream::{FuturesOrdered, FuturesUnordered},
    StreamExt,
};
use tokio::{sync::mpsc, task::JoinHandle, time::sleep};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

use crate::AppConfig;

#[derive(Debug, Clone)]
pub struct ParallelTaskRunner {
    pub timeout: Duration,
    pub ordered: bool,
    pub cancellation_token: CancellationToken,
    pub retry_interval: Duration,
    pub retry_timeout: Duration,
}

/// Just a helper.
macro_rules! send {
    ($tx:expr, $msg:expr) => {
        if let Err(err) = $tx.send($msg).await {
            if $tx.is_closed() {
                debug!("Cannot send task result: Task aborted.");
            } else {
                error!("Cannot send task result: {err}");
            }
        }
    };
}

impl ParallelTaskRunner {
    pub fn default(app_config: &AppConfig) -> Self {
        let default_reponse_timeout = app_config.default_response_timeout.into_std_duration();
        Self {
            timeout: default_reponse_timeout,
            ordered: false,
            cancellation_token: CancellationToken::new(),
            retry_interval: Duration::from_secs(5),
            retry_timeout: default_reponse_timeout,
        }
    }
    pub fn ordered(self) -> Self {
        Self {
            ordered: true,
            ..self
        }
    }
    /// "No timeout" means 1 hour (which should never happen).
    pub fn no_timeout(self) -> Self {
        Self {
            timeout: Duration::from_secs(3600),
            ..self
        }
    }

    pub fn run<F, R>(
        &self,
        futures: Vec<F>,
        on_cancel: impl FnOnce() -> () + Send + 'static,
    ) -> mpsc::Receiver<R>
    where
        F: Future<Output = R> + Send + Unpin + 'static,
        R: Send + 'static,
    {
        let Self {
            timeout, ordered, ..
        } = self.clone();
        let cancellation_token = self.cancellation_token.child_token();

        let (tx, rx) = mpsc::channel::<R>(min(futures.len(), 32));
        tokio::spawn(async move {
            let mut tasks: Futures<JoinHandle<Option<R>>> = Futures::new(
                futures.into_iter().map(|future| {
                    let cancellation_token = cancellation_token.clone();
                    tokio::spawn(async move {
                        tokio::select! {
                            res = future => { Some(res) },
                            _ = cancellation_token.cancelled() => { None },
                        }
                    })
                }),
                ordered,
            );

            tokio::select! {
                _ = async {
                    // NOTE: If `futures.len() == 0` then this `tokio::select!` ends instantly.
                    while let Some(Ok(Some(msg))) = tasks.next().await {
                        send!(tx, msg);
                    }
                } => {}
                _ = sleep(timeout) => {
                    debug!("Timed out. Cancelling all tasks…");

                    cancellation_token.cancel();
                    on_cancel();
                }
            };
        });

        rx
    }

    /// NOTE: Make sure to increase [`timeout`][ParallelTaskRunner::timeout] in order
    ///   for retries to work as expected. You can use [`ParallelTaskRunner::no_timeout`].
    pub fn run_with_retries<F, R>(
        &self,
        futures: Vec<F>,
        on_cancel: impl FnOnce() -> () + Send + 'static,
        should_retry: impl Fn(&R) -> bool + Send + 'static + Copy + Sync,
        default_result: impl Fn(&R) -> Option<R> + Send + 'static + Copy,
    ) -> mpsc::Receiver<R>
    where
        F: Future<Output = R> + Send + Unpin + 'static + Copy + Sync,
        R: Send + 'static,
    {
        let Self {
            timeout,
            ordered,
            retry_interval,
            retry_timeout,
            ..
        } = self.clone();
        let cancellation_token = self.cancellation_token.child_token();

        // Create a mpsc channel to receive task results.
        let (tx, rx) = mpsc::channel::<R>(min(futures.len(), 32));

        // Spawn a task which will send results to the mpsc channel.
        tokio::spawn(async move {
            // Map futures to cancellable and retryable tasks.
            let mut tasks: Futures<JoinHandle<Option<R>>> = Futures::new(
                futures.into_iter().map(|future| {
                    let cancellation_token = cancellation_token.clone();
                    let tx = tx.clone();

                    // Spawn a new task so the future runs concurrently with the other ones.
                    tokio::spawn(async move {
                        tokio::select! {
                            // Try a first time.
                            res = future => {
                                // Test if we should retry.
                                let mut should_retry_this = should_retry(&res);
                                let default_result = default_result(&res);

                                // Send the result.
                                send!(tx, res);

                                if should_retry_this {
                                    tokio::spawn(async move {
                                        tokio::select! {
                                            _ = async {
                                                while should_retry_this {
                                                    // Wait before retry.
                                                    sleep(retry_interval).await;
                                                    // Retry.
                                                    tokio::select! {
                                                        res = future => {
                                                            // Test if we should retry.
                                                            should_retry_this = should_retry(&res);
                                                            // Send the result.
                                                            send!(tx, res);
                                                        },
                                                        // Add retry timeout.
                                                        _ = sleep(retry_timeout) => {
                                                            // If we hit the timeout, `should_retry_this` is still `true`.
                                                            // TODO: Notify we hit the timeout.
                                                        },
                                                        // Allow cancellation.
                                                        _ = cancellation_token.cancelled() => {},
                                                    };
                                                }
                                            } => {}
                                        };
                                    });
                                }

                                default_result
                            },
                            // Allow cancellation.
                            _ = cancellation_token.cancelled() => { None },
                        }
                    })
                }),
                ordered,
            );

            tokio::select! {
                _ = async {
                    // NOTE: If `futures.len() == 0` then this `tokio::select!` ends instantly.
                    while let Some(Ok(res)) = tasks.next().await {
                        if let Some(msg) = res {
                            send!(tx, msg);
                        }
                    }
                } => {}
                _ = sleep(timeout) => {
                    debug!("Timed out. Cancelling all tasks…");

                    cancellation_token.cancel();
                    on_cancel();
                }
            };
        });

        rx
    }

    pub fn cancel_all_tasks(&self) {
        self.cancellation_token.cancel();
    }
}

enum Futures<F: Future> {
    Ordered(FuturesOrdered<F>),
    Unordered(FuturesUnordered<F>),
}

impl<F: Future> Futures<F> {
    fn new(iter: impl Iterator<Item = F>, ordered: bool) -> Self {
        if ordered {
            Self::Ordered(iter.collect())
        } else {
            Self::Unordered(iter.collect())
        }
    }
    async fn next(&mut self) -> Option<F::Output> {
        match self {
            Futures::Ordered(futures) => futures.next().await,
            Futures::Unordered(futures) => futures.next().await,
        }
    }
}

impl<F: Future> From<FuturesOrdered<F>> for Futures<F> {
    fn from(futures: FuturesOrdered<F>) -> Self {
        Self::Ordered(futures)
    }
}
impl<F: Future> From<FuturesUnordered<F>> for Futures<F> {
    fn from(futures: FuturesUnordered<F>) -> Self {
        Self::Unordered(futures)
    }
}
