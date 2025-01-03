// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::{model::*, prelude::*, util::*};

pub async fn check_ports_route(
    pod_network_config: PodNetworkConfig,
    network_checker: NetworkChecker,
) -> Result<Json<Vec<NetworkCheckResult>>, Error> {
    let res = run_checks(
        pod_network_config.port_reachability_checks().into_iter(),
        &network_checker,
    )
    .await;
    Ok(Json(res))
}

pub async fn check_ports_stream_route(
    pod_network_config: PodNetworkConfig,
    network_checker: NetworkChecker,
    Query(forms::Interval { interval }): Query<forms::Interval>,
    State(AppState { app_config, .. }): State<AppState>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Error> {
    run_checks_stream(
        pod_network_config.port_reachability_checks().into_iter(),
        network_checker,
        port_reachability_check_result,
        interval,
        app_config,
    )
}

// MODEL

#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PortReachabilityStatus {
    Queued,
    Checking,
    Open,
    Closed,
}

// BOILERPLATE

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
