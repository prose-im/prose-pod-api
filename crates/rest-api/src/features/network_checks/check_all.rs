// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use futures::{stream::FuturesOrdered, StreamExt};
use service::{util::ConcurrentTaskRunner, AppConfig};
use tokio::sync::mpsc;

use crate::features::network_checks::{
    check_dns_records::dns_record_check_result,
    check_ip_connectivity::ip_connectivity_check_result,
    check_ports_reachability::port_reachability_check_result,
};

use super::{model::*, prelude::*, util::*};

#[get("/v1/network/checks", format = "application/json")]
pub async fn check_network_configuration_route<'r>(
    pod_network_config: LazyGuard<PodNetworkConfig>,
    network_checker: &'r State<NetworkChecker>,
) -> Result<Json<Vec<NetworkCheckResult>>, Error> {
    let pod_network_config = pod_network_config.inner?;
    let network_checker = network_checker.inner().to_owned();

    let mut tasks: FuturesOrdered<tokio::task::JoinHandle<_>> = FuturesOrdered::default();

    for check in pod_network_config.dns_record_checks() {
        let network_checker = network_checker.clone();
        tasks.push_back(tokio::spawn(async move {
            let result = check.run(&network_checker).await;
            NetworkCheckResult::from((check, result))
        }));
    }
    for check in pod_network_config.port_reachability_checks() {
        let network_checker = network_checker.clone();
        tasks.push_back(tokio::spawn(async move {
            let result = check.run(&network_checker).await;
            NetworkCheckResult::from((check, result))
        }));
    }
    for check in pod_network_config.ip_connectivity_checks() {
        let network_checker = network_checker.clone();
        tasks.push_back(tokio::spawn(async move {
            let result = check.run(&network_checker).await;
            NetworkCheckResult::from((check, result))
        }));
    }

    let res: Vec<NetworkCheckResult> = tasks.filter_map(|res| async { res.ok() }).collect().await;

    Ok(Json(res))
}

#[get(
    "/v1/network/checks?<interval>",
    format = "text/event-stream",
    rank = 2
)]
pub fn check_network_configuration_stream_route<'r>(
    pod_network_config: LazyGuard<PodNetworkConfig>,
    network_checker: &'r State<NetworkChecker>,
    interval: Option<forms::Duration>,
    app_config: &'r State<AppConfig>,
) -> Result<EventStream![Event + 'r], Error> {
    let pod_network_config = pod_network_config.inner?;
    let network_checker = network_checker.inner();

    let retry_interval =
        interval.map_or_else(|| Ok(*DEFAULT_RETRY_INTERVAL), validate_retry_interval)?;

    Ok(EventStream! {
        fn logged(event: Event) -> Event {
            trace!("Sending {event:?}…");
            event
        }

        let runner = ConcurrentTaskRunner::default(&app_config)
            .no_timeout()
            .with_retry_interval(retry_interval);
        let (tx, mut rx) = mpsc::channel::<Event>(32);

        let dns_record_checks = pod_network_config.dns_record_checks();
        let port_reachability_checks = pod_network_config.port_reachability_checks();
        let ip_connectivity_checks = pod_network_config.ip_connectivity_checks();

        network_checker.run_checks(
            dns_record_checks,
            dns_record_check_result,
            tx.clone(),
            &runner,
        );
        network_checker.run_checks(
            port_reachability_checks.into_iter(),
            port_reachability_check_result,
            tx.clone(),
            &runner,
        );
        network_checker.run_checks(
            ip_connectivity_checks.into_iter(),
            ip_connectivity_check_result,
            tx.clone(),
            &runner,
        );

        while let Some(event) = rx.recv().await {
            yield logged(event);
        }

        yield logged(end_event());
    })
}
