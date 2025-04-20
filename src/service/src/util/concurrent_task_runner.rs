// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{cmp::min, fmt::Debug, future::Future, sync::Arc, time::Duration};

use futures::{
    stream::{FuturesOrdered, FuturesUnordered},
    StreamExt,
};
use tokio::{
    sync::{mpsc, Barrier},
    task::JoinHandle,
    time::sleep,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, instrument, trace, trace_span, Instrument as _};

use crate::AppConfig;

/// Helper used to run multiple tasks concurrently.
///
/// It supports timeouts and cancellation out of the box,
/// along with retry logic if needed.
#[derive(Debug, Clone)]
pub struct ConcurrentTaskRunner {
    pub timeout: Duration,
    pub ordered: bool,
    pub cancellation_token: CancellationToken,
    pub retry_interval: Duration,
    pub retry_timeout: Duration,
}

impl ConcurrentTaskRunner {
    pub fn default(app_config: &AppConfig) -> Self {
        let default_reponse_timeout = app_config.default_response_timeout.into_std_duration();
        Self {
            timeout: default_reponse_timeout,
            ordered: false,
            cancellation_token: CancellationToken::new(),
            retry_interval: app_config.default_retry_interval.into_std_duration(),
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
    pub fn no_timeout(mut self) -> Self {
        self.timeout = Duration::from_secs(3600);
        self
    }
    pub fn with_retry_interval(mut self, retry_interval: Duration) -> Self {
        self.retry_interval = retry_interval;
        self
    }
    pub fn child(&self) -> Self {
        Self {
            timeout: self.timeout.clone(),
            ordered: self.ordered,
            cancellation_token: self.cancellation_token.child_token(),
            retry_interval: self.retry_interval.clone(),
            retry_timeout: self.retry_timeout.clone(),
        }
    }
}

/// Just a helper.
macro_rules! send {
    ($tx:expr, $msg:expr, $cancellation_token:expr) => {
        if let Err(err) = $tx.send($msg).await {
            if $tx.is_closed() {
                debug!("Cannot send task result: Task aborted.");
            } else {
                error!("Cannot send task result: {err}");
            }
            // We can't send task results so cancel all tasks.
            $cancellation_token.cancel();
        }
    };
}

impl ConcurrentTaskRunner {
    /// Run tasks concurrently without retries.
    #[instrument(level = "trace", skip_all)]
    pub fn run<D, F, R>(
        &self,
        data: Vec<D>,
        make_future: impl Fn(D) -> F + Send + 'static + Sync,
        on_cancel: impl FnOnce() -> () + Send + 'static,
    ) -> mpsc::Receiver<R>
    where
        D: Debug + Clone + Send + 'static,
        F: Future<Output = R> + Send + 'static + Unpin,
        R: Send + 'static,
    {
        let Self {
            timeout, ordered, ..
        } = self.clone();
        let cancellation_token = self.cancellation_token.child_token();
        let make_future = Arc::new(make_future);

        // Create a mpsc channel to receive task results.
        let (tx, rx) = mpsc::channel::<R>(min(data.len(), 32));

        // Map futures to cancellable tasks.
        let mut tasks: Futures<JoinHandle<Option<R>>> = Futures::new(
            data.into_iter().map(|data| {
                let cancellation_token = cancellation_token.clone();
                let make_future = make_future.clone();

                let span = trace_span!("task", data = format!("{data:?}"));
                tokio::spawn(
                    async move {
                        tokio::select! {
                            res = make_future(data) => { Some(res) },
                            _ = cancellation_token.cancelled() => { None },
                        }
                    }
                    .instrument(span)
                    .in_current_span(),
                )
            }),
            ordered,
        );

        // Spawn a task which will send results to the mpsc channel.
        tokio::spawn(
            async move {
                tokio::select! {
                    // NOTE: If `data.len() == 0` then this `tokio::select!` ends instantly.
                    _ = async {
                        while let Some(Ok(Some(msg))) = tasks.next().await {
                            send!(tx, msg, cancellation_token);
                        }
                    } => {}
                    _ = sleep(timeout) => {
                        trace!("⌛️ Timed out. Cancelling all tasks…");
                        on_cancel();
                        cancellation_token.cancel();
                    }
                };
            }
            .instrument(trace_span!("send_results"))
            .in_current_span(),
        );

        rx
    }

    /// Run tasks concurrently with retries.
    ///
    /// NOTE: No matter [`ordered`][Self::ordered], "default results" (e.g. "QUEUED" events)
    ///   will always be ordered and retry results unordered.
    ///
    /// WARN: Make sure to increase [`timeout`][Self::timeout] in order
    ///   for retries to work as expected. You can use [`Self::no_timeout`].
    #[instrument(level = "trace", skip_all)]
    pub fn run_with_retries<D, F, R, DefaultResult, RetryResult>(
        &self,
        data: Vec<D>,
        make_future: impl Fn(D) -> F + Send + 'static + Sync,
        before_all: Option<DefaultResult>,
        before_retry: Option<RetryResult>,
        should_retry: impl Fn(&R) -> bool + Send + 'static + Copy + Sync,
        on_cancel: impl FnOnce() -> () + Send + 'static,
    ) -> mpsc::Receiver<R>
    where
        D: Debug + Clone + Send + 'static,
        DefaultResult: Fn(&D) -> R + Send + 'static + Copy,
        RetryResult: Fn(&D) -> R + Send + 'static + Copy,
        F: Future<Output = R> + Send + 'static + Unpin,
        R: Send + 'static,
    {
        let Self {
            timeout,
            retry_interval,
            retry_timeout,
            ..
        } = self.clone();
        let cancellation_token = self.cancellation_token.child_token();
        let make_future = Arc::new(make_future);

        let default_msgs: Option<Vec<R>> = before_all.map(|f| data.iter().map(f).collect());

        // Create a mpsc channel to receive task results.
        let (tx, rx) = mpsc::channel::<R>(min(data.len(), 32));

        let barrier = before_all.map(|_| Arc::new(Barrier::new(data.len() + 1)));

        // Map futures to cancellable and retryable tasks.
        let mut tasks: FuturesUnordered<JoinHandle<()>> = data
            .into_iter()
            .map(|data| {
                let cancellation_token = cancellation_token.clone();
                let tx = tx.clone();
                let barrier = barrier.clone();
                let make_future = make_future.clone();

                // Spawn a new task so the future runs concurrently with the other ones.
                let span = trace_span!("task", data = format!("{data:?}"));
                tokio::spawn(
                    async move {
                        if let Some(barrier) = barrier {
                            // Wait for default messages to be sent.
                            barrier.wait().await;
                        };

                        let mut should_retry_this = true;
                        let mut try_n = 1;

                        while should_retry_this {
                            // Avoid an unnecessary retry if applicable.
                            if cancellation_token.is_cancelled() || tx.is_closed() {
                                break
                            }

                            // Optionally send messages before retrying (e.g. "CHECKING").
                            if let Some(before_retry) = before_retry {
                                send!(tx, before_retry(&data), cancellation_token);
                            }

                            tokio::select! {
                                res = make_future(data.clone()).instrument(trace_span!("try", n = try_n)) => {
                                    // Test if we should retry.
                                    should_retry_this = should_retry(&res);
                                    // Send the result.
                                    send!(tx, res, cancellation_token);
                                },
                                // Add retry timeout.
                                _ = sleep(retry_timeout) => {
                                    // If we hit the timeout, `should_retry_this` is still `true`.
                                    // TODO: Notify we hit the timeout.
                                },
                                // Allow cancellation of retries.
                                _ = cancellation_token.cancelled() => {
                                    // If the task if cancelled, break out of the loop.
                                    break
                                },
                            };

                            if should_retry_this {
                                // Wait before retry.
                                sleep(retry_interval).await;
                                try_n += 1;
                            }
                        }
                    }
                    .instrument(span)
                    .in_current_span(),
                )
            })
            .collect();

        // Spawn a task which will send results to the mpsc channel.
        tokio::spawn(
            async move {
                tokio::select! {
                    // NOTE: If `data.len() == 0` then this `tokio::select!` ends instantly.
                    _ = async {
                        // Send default messages if needed.
                        // For example, send `QUEUED` event before tasks start.
                        if let Some(default_msgs) = default_msgs {
                            for msg in default_msgs {
                                send!(tx, msg, cancellation_token);
                            }
                        }

                        // Notify that default messages have been sent.
                        if let Some(barrier) = barrier {
                            barrier.wait().await;
                        }

                        // Wait for all tasks to finish.
                        while let Some(Ok(())) = tasks.next().await {}
                    } => {}
                    // Add global timeout.
                    _ = sleep(timeout) => {
                        trace!("⌛️ Timed out. Cancelling all tasks…");
                        on_cancel();
                        cancellation_token.cancel();
                    }
                };
            }
            .instrument(trace_span!("send_results"))
            .in_current_span(),
        );

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
