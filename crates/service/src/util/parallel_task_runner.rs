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
}

impl ParallelTaskRunner {
    pub fn default(app_config: &AppConfig) -> Self {
        Self {
            timeout: app_config.default_response_timeout.into_std_duration(),
            ordered: false,
        }
    }
    pub fn ordered(self) -> Self {
        Self {
            timeout: self.timeout,
            ordered: true,
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

        let (tx, rx) = mpsc::channel::<R>(min(futures.len(), 32));
        tokio::spawn(async move {
            let cancellation_token = CancellationToken::new();
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
                        if let Err(err) = tx.send(msg).await {
                            if tx.is_closed() {
                                debug!("Cannot send task result: Task aborted.");
                            } else {
                                error!("Cannot send task result: {err}");
                            }
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
