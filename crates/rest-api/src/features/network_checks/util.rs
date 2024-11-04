// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::atomic::{AtomicUsize, Ordering};

use futures::{stream::FuturesOrdered, StreamExt};
use lazy_static::lazy_static;
use rocket::response::stream::{Event, EventStream};
use service::{model::network_checks::*, services::network_checker::*};
use tokio::{
    sync::mpsc::{self, error::SendError},
    task::JoinSet,
    time::Duration,
};

use crate::{
    error::{self, Error},
    forms,
};

use super::{end_event, NetworkCheckResult};

lazy_static! {
    pub static ref DEFAULT_RETRY_INTERVAL: Duration = Duration::from_secs(5);
}

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

pub fn run_checks_stream<'r, Check, Status>(
    checks: impl Iterator<Item = Check> + Clone + 'r,
    network_checker: &'r NetworkChecker,
    map_to_event: impl Fn(&Check, Status) -> Event + Copy + Send + 'static,
    retry_interval: Option<forms::Duration>,
) -> Result<EventStream![Event + 'r], Error>
where
    Check: NetworkCheck + Send + 'static,
    Check::CheckResult: RetryableNetworkCheckResult + Clone + Send,
    Status: From<Check::CheckResult> + Default,
{
    let retry_interval =
        retry_interval.map_or_else(|| Ok(*DEFAULT_RETRY_INTERVAL), validate_retry_interval)?;

    Ok(EventStream! {
        fn logged(event: Event) -> Event {
            trace!("Sending {event:?}…");
            event
        }

        let (tx, mut rx) = mpsc::channel::<Option<Event>>(32);
        let mut join_set = JoinSet::<Result<(), SendError<Option<Event>>>>::new();

        let remaining = AtomicUsize::new(checks.clone().count());
        network_checker.run_checks(
            checks,
            map_to_event,
            retry_interval.into(),
            tx,
            &mut join_set,
        );

        while remaining.load(Ordering::Relaxed) != 0 {
            match rx.recv().await {
                Some(Some(event)) => yield logged(event),
                Some(None) => { remaining.fetch_sub(1, Ordering::Relaxed); },
                None => {},
            }
        }

        yield logged(end_event());
    })
}

/// Check that the retry interval is between 1 second and 1 minute (inclusive).
pub fn validate_retry_interval(interval: forms::Duration) -> Result<Duration, Error> {
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
