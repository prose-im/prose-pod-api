// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    cmp::{max, min},
    fmt::Debug,
    future::Future,
    sync::Arc,
    time::Duration,
};

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

use crate::app_config::NetworkChecksConfig;

/// Helper used to run multiple tasks concurrently.
///
/// It supports timeouts and cancellation out of the box,
/// along with retry logic if needed.
#[derive(Debug, Clone)]
pub struct ConcurrentTaskRunner {
    pub ordered: bool,
    pub cancellation_token: CancellationToken,
    pub timings: TaskRunnerTimings,
}

impl Default for ConcurrentTaskRunner {
    fn default() -> Self {
        Self {
            ordered: false,
            cancellation_token: CancellationToken::new(),
            timings: Default::default(),
        }
    }
}

impl ConcurrentTaskRunner {
    pub fn ordered(self) -> Self {
        Self {
            ordered: true,
            ..self
        }
    }

    pub fn with_timings(mut self, timings: TaskRunnerTimings) -> Self {
        self.timings = timings;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timings.timeout = timeout;
        self
    }

    /// “No timeout” means 1 hour (which should never happen).
    pub fn no_timeout(mut self) -> Self {
        self.timings.timeout = Duration::from_secs(3600);
        self
    }

    pub fn with_retry_interval(mut self, retry_interval: Duration) -> Self {
        self.timings.retry_interval = retry_interval;
        self
    }

    pub fn child(&self) -> Self {
        Self {
            ordered: self.ordered,
            cancellation_token: self.cancellation_token.child_token(),
            timings: self.timings,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TaskRunnerTimings {
    pub timeout: Duration,
    pub retry_interval: Duration,
    pub retry_timeout: Duration,
}

impl Default for TaskRunnerTimings {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            retry_interval: Duration::from_secs(2),
            retry_timeout: Duration::from_secs(1),
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
        F: Future<Output = R> + Send + 'static,
        R: Send + 'static,
    {
        let Self {
            timings, ordered, ..
        } = self.clone();
        let cancellation_token = self.cancellation_token.child_token();
        let make_future = Arc::new(make_future);

        // Create a mpsc channel to receive task results.
        // NOTE: `mpsc::channel` panics if passed `0`, hence the `max(_, 1)`.
        //   Fixes https://github.com/prose-im/prose-pod-api/issues/238.
        let (tx, rx) = mpsc::channel::<R>(min(max(data.len(), 1), 32));

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
                    _ = sleep(timings.timeout) => {
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
        on_finish: impl FnOnce() -> () + Send + 'static,
        on_cancel: impl FnOnce() -> () + Send + 'static,
    ) -> mpsc::Receiver<R>
    where
        D: Debug + Clone + Send + 'static,
        DefaultResult: Fn(&D) -> R + Send + 'static + Copy,
        RetryResult: Fn(&D) -> R + Send + 'static + Copy,
        F: Future<Output = R> + Send + 'static + Unpin,
        R: Send + 'static,
    {
        let TaskRunnerTimings {
            timeout,
            retry_interval,
            retry_timeout,
        } = self.timings;
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

                        on_finish()
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

#[derive(Debug)]
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

// MARK: - Boilerplate

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

impl From<NetworkChecksConfig> for TaskRunnerTimings {
    fn from(value: NetworkChecksConfig) -> Self {
        Self {
            timeout: value.timeout.into_std_duration(),
            retry_interval: value.retry_interval.into_std_duration(),
            retry_timeout: value.retry_timeout.into_std_duration(),
        }
    }
}
