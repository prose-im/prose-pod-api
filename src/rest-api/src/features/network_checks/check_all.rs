// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::response::sse::KeepAlive;
use futures::{stream::FuturesOrdered, StreamExt};
use service::util::ConcurrentTaskRunner;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{trace, Instrument as _};

use crate::features::network_checks::{
    check_dns_records::dns_record_check_result,
    check_ip_connectivity::ip_connectivity_check_result,
    check_ports_reachability::port_reachability_check_result,
};

use super::{model::*, prelude::*, util::*, SSE_TIMEOUT};

pub async fn check_network_configuration_route(
    pod_network_config: PodNetworkConfig,
    network_checker: NetworkChecker,
) -> Result<Json<Vec<NetworkCheckResult>>, Error> {
    let mut tasks: FuturesOrdered<tokio::task::JoinHandle<_>> = FuturesOrdered::default();

    for check in pod_network_config.dns_record_checks() {
        let network_checker = network_checker.clone();
        tasks.push_back(tokio::spawn(
            async move {
                let result = check.run(&network_checker).await;
                NetworkCheckResult::from((check, result))
            }
            .in_current_span(),
        ));
    }
    for check in pod_network_config.port_reachability_checks() {
        let network_checker = network_checker.clone();
        tasks.push_back(tokio::spawn(
            async move {
                let result = check.run(&network_checker).await;
                NetworkCheckResult::from((check, result))
            }
            .in_current_span(),
        ));
    }
    for check in pod_network_config.ip_connectivity_checks() {
        let network_checker = network_checker.clone();
        tasks.push_back(tokio::spawn(
            async move {
                let result = check.run(&network_checker).await;
                NetworkCheckResult::from((check, result))
            }
            .in_current_span(),
        ));
    }

    let res: Vec<NetworkCheckResult> = tasks.filter_map(|res| async { res.ok() }).collect().await;

    Ok(Json(res))
}

pub async fn check_network_configuration_stream_route(
    pod_network_config: PodNetworkConfig,
    network_checker: NetworkChecker,
    Query(forms::Interval { interval }): Query<forms::Interval>,
    State(AppState { app_config, .. }): State<AppState>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Error> {
    let retry_interval = interval.map_or_else(
        || Ok(app_config.default_retry_interval.into_std_duration()),
        validate_retry_interval,
    )?;
    let runner = ConcurrentTaskRunner::default(&app_config)
        .with_timeout(*SSE_TIMEOUT)
        .with_retry_interval(retry_interval);
    let cancellation_token = runner.cancellation_token.clone();

    let (sse_tx, sse_rx) = mpsc::channel::<Result<Event, Infallible>>(32);
    tokio::spawn(async move {
        tokio::select! {
                _ = async {
                    fn logged(event: Event) -> Event {
                        trace!("Sending {event:?}…");
                        event
                    }

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
                        if sse_tx.send(Ok(logged(event))).await.ok().is_none() {
                            return
                        }
                    }

                    if sse_tx.send(Ok(logged(end_event()))).await.ok().is_none() {
                        return
                    }
                } => {}
            _ = cancellation_token.cancelled() => {
                trace!("Token cancelled.");
            }
        };
    }
    .in_current_span());

    Ok(Sse::new(ReceiverStream::new(sse_rx)).keep_alive(KeepAlive::default()))
}
