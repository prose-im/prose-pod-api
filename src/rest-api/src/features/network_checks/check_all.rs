// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::response::sse::KeepAlive;
use futures::{stream::FuturesOrdered, StreamExt};
use service::{onboarding, util::ConcurrentTaskRunner};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{trace, warn, Instrument as _};

use crate::features::network_checks::{
    check_dns_records::dns_record_check_result,
    check_ip_connectivity::ip_connectivity_check_result,
    check_ports_reachability::port_reachability_check_result,
};

use super::{check_dns_records_route_, model::*, prelude::*, util::*, SSE_TIMEOUT};

pub async fn check_network_configuration_route(
    State(AppState { db, .. }): State<AppState>,
    pod_network_config: PodNetworkConfig,
    network_checker: NetworkChecker,
) -> Result<Json<Vec<NetworkCheckResult>>, Error> {
    let mut tasks: FuturesOrdered<_> = FuturesOrdered::default();

    {
        let pod_network_config = pod_network_config.clone();
        let network_checker = network_checker.clone();
        tasks.push_back(tokio::spawn(
            async move { check_dns_records_route_(pod_network_config, network_checker, &db).await }
                .in_current_span(),
        ));
    }
    {
        let pod_network_config = pod_network_config.clone();
        let network_checker = network_checker.clone();
        tasks.push_back(tokio::spawn(
            async move {
                run_checks(
                    pod_network_config.port_reachability_checks().into_iter(),
                    &network_checker,
                    NetworkCheckResult::from,
                )
                .await
            }
            .in_current_span(),
        ));
    }
    {
        let pod_network_config = pod_network_config.clone();
        let network_checker = network_checker.clone();
        tasks.push_back(tokio::spawn(
            async move {
                run_checks(
                    pod_network_config.ip_connectivity_checks().into_iter(),
                    &network_checker,
                    NetworkCheckResult::from,
                )
                .await
            }
            .in_current_span(),
        ));
    }

    let res: Vec<NetworkCheckResult> = tasks
        .filter_map(|res| async { res.ok() })
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect();

    Ok(Json(res))
}

pub async fn check_network_configuration_stream_route(
    pod_network_config: PodNetworkConfig,
    network_checker: NetworkChecker,
    Query(forms::Interval { interval }): Query<forms::Interval>,
    State(AppState { app_config, db, .. }): State<AppState>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Error> {
    let ref app_config = app_config.read().unwrap().clone();
    let retry_interval = interval.map_or_else(
        || Ok(app_config.default_retry_interval.into_std_duration()),
        validate_retry_interval,
    )?;
    let runner = ConcurrentTaskRunner::default(app_config)
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
                        move || {
                            tokio::spawn(async move {
                                trace!("Setting `all_dns_checks_passed_once` to true…");
                                (onboarding::all_dns_checks_passed_once::set(&db, true).await)
                                    .inspect_err(|err| {
                                        warn!("Could not set `all_dns_checks_passed_once` to true: {err}")
                                    })
                                    .ok();
                            });
                        },
                    );
                    network_checker.run_checks(
                        port_reachability_checks.into_iter(),
                        port_reachability_check_result,
                        tx.clone(),
                        &runner,
                        move || {},
                    );
                    network_checker.run_checks(
                        ip_connectivity_checks.into_iter(),
                        ip_connectivity_check_result,
                        tx.clone(),
                        &runner,
                        move || {},
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
