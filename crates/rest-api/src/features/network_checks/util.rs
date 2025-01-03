// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{convert::Infallible, fmt::Debug};

use axum::response::{
    sse::{Event, KeepAlive},
    Sse,
};
use futures::{stream::FuturesOrdered, Stream, StreamExt};
use service::{network_checks::*, util::ConcurrentTaskRunner, AppConfig};
use tokio::{sync::mpsc, time::Duration};
use tokio_stream::wrappers::ReceiverStream;
use tracing::trace;

use crate::error::{self, Error};

use super::{end_event, NetworkCheckResult};

pub async fn run_checks<'r, Check>(
    checks: impl Iterator<Item = Check> + 'r,
    network_checker: &'r NetworkChecker,
) -> Vec<NetworkCheckResult>
where
    Check: NetworkCheck + Send + 'static,
    Check::CheckResult: Clone + Send,
    NetworkCheckResult: From<(Check, Check::CheckResult)>,
{
    checks
        .map(|check| async move {
            let result = check.run(network_checker).await;
            NetworkCheckResult::from((check, result))
        })
        .collect::<FuturesOrdered<_>>()
        .collect()
        .await
}

pub fn run_checks_stream<Check, Status>(
    checks: impl Iterator<Item = Check> + Clone + Send + 'static,
    network_checker: NetworkChecker,
    map_to_event: impl Fn(&Check, Status) -> Event + Copy + Send + Sync + 'static,
    retry_interval: Option<iso8601_duration::Duration>,
    app_config: AppConfig,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Error>
where
    Check: NetworkCheck + Debug + Send + 'static + Clone + Sync,
    Check::CheckResult: RetryableNetworkCheckResult + Clone + Send,
    Status: From<Check::CheckResult> + WithQueued + WithChecking + Send + 'static,
{
    let retry_interval = retry_interval.map_or_else(
        || Ok(app_config.default_retry_interval.into_std_duration()),
        validate_retry_interval,
    )?;

    let (sse_tx, sse_rx) = mpsc::channel::<Result<Event, Infallible>>(32);

    let runner = ConcurrentTaskRunner::default(&app_config)
        .no_timeout()
        .with_retry_interval(retry_interval);
    let cancellation_token = runner.cancellation_token.clone();

    tokio::spawn(async move {
        tokio::select! {
            _ = async {
                fn logged(event: Event) -> Event {
                    trace!("Sending {event:?}…");
                    event
                }

                let (tx, mut rx) = mpsc::channel::<Event>(32);
                network_checker.run_checks(checks, map_to_event, tx, &runner);

                while let Some(event) = rx.recv().await {
                    sse_tx.send(Ok(logged(event))).await.unwrap();
                }

                sse_tx.send(Ok(logged(end_event()))).await.unwrap();
            } => {}
            _ = cancellation_token.cancelled() => {
                trace!("Token cancelled.");
            }
        };
    });

    Ok(Sse::new(ReceiverStream::new(sse_rx)).keep_alive(KeepAlive::default()))
}

/// Check that the retry interval is between 1 second and 1 minute (inclusive).
pub fn validate_retry_interval(interval: iso8601_duration::Duration) -> Result<Duration, Error> {
    let interval_is_max_1_minute = || interval.num_minutes().is_some_and(|m| m <= 1.);
    let interval_is_min_1_second = || interval.num_seconds().is_some_and(|s| s >= 1.);

    let interval_is_valid = interval_is_max_1_minute() && interval_is_min_1_second();

    if interval_is_valid {
        // NOTE: We can force unwrap here because `to_std` only returns `None` if `Duration` contains `year` or `month`,
        //   which is impossible due to previous checks.
        Ok(interval.to_std().unwrap())
    } else {
        Err(error::BadRequest {
            reason: "Invalid retry interval. Authorized values must be between 1 second and 1 minute (inclusive).".to_string()
        }.into())
    }
}
