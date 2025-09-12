// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::{model::*, prelude::*, util::*};

pub async fn check_ports_route(
    pod_network_config: PodNetworkConfig,
    network_checker: NetworkChecker,
    query: Query<forms::Interval>,
    app_config: State<Arc<AppConfig>>,
    headers: HeaderMap,
) -> Either<
    Result<Json<Vec<NetworkCheckResult>>, Error>,
    Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Error>,
> {
    match headers.get(ACCEPT) {
        Some(ct) if ct.starts_with(TEXT_EVENT_STREAM.essence_str()) => Either::E2(
            check_ports_stream_route_(pod_network_config, network_checker, app_config, query).await,
        ),
        _ => Either::E1(check_ports_route_(pod_network_config, network_checker).await),
    }
}

async fn check_ports_route_(
    pod_network_config: PodNetworkConfig,
    network_checker: NetworkChecker,
) -> Result<Json<Vec<NetworkCheckResult>>, Error> {
    let res = run_checks(
        pod_network_config.port_reachability_checks().into_iter(),
        &network_checker,
        NetworkCheckResult::from,
    )
    .await;
    Ok(Json(res))
}

async fn check_ports_stream_route_(
    pod_network_config: PodNetworkConfig,
    network_checker: NetworkChecker,
    State(ref app_config): State<Arc<AppConfig>>,
    Query(forms::Interval { interval }): Query<forms::Interval>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Error> {
    run_checks_stream(
        pod_network_config.port_reachability_checks().into_iter(),
        network_checker,
        port_reachability_check_result,
        interval,
        app_config,
        move || {},
    )
}

// MARK: - Models

#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PortReachabilityStatus {
    Queued,
    Checking,
    Open,
    Closed,
}

// MARK: - Boilerplate

impl_network_check_result_from!(
    PortReachabilityCheck,
    PortReachabilityCheckResult,
    PortReachabilityStatus,
    PortReachabilityCheckId
);
impl_network_check_event_from!(PortReachabilityCheck, Self::PortReachabilityCheckResult);

impl WithQueued for PortReachabilityStatus {
    fn queued() -> Self {
        Self::Queued
    }
}
impl WithChecking for PortReachabilityStatus {
    fn checking() -> Self {
        Self::Checking
    }
}

impl From<PortReachabilityCheckResult> for PortReachabilityStatus {
    fn from(check_result: PortReachabilityCheckResult) -> Self {
        match check_result {
            PortReachabilityCheckResult::Open => Self::Open,
            PortReachabilityCheckResult::Closed => Self::Closed,
        }
    }
}

pub fn port_reachability_check_result(
    check: &PortReachabilityCheck,
    status: PortReachabilityStatus,
) -> Event {
    Event::default()
        .event(NetworkCheckEvent::PortReachabilityCheckResult.to_string())
        .id(PortReachabilityCheckId::from(check).to_string())
        .json_data(&CheckResultData {
            description: check.description(),
            status,
        })
        .unwrap()
}
