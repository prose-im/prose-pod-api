// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::{model::*, prelude::*, util::*};

#[rocket::get("/v1/network/checks/ports", format = "application/json")]
pub async fn check_ports_route<'r>(
    pod_network_config: LazyGuard<PodNetworkConfig>,
    network_checker: &'r State<NetworkChecker>,
) -> Result<Json<Vec<NetworkCheckResult>>, Error> {
    let pod_network_config = pod_network_config.inner?;
    let network_checker = network_checker.inner();

    let res = run_checks(
        pod_network_config.port_reachability_checks().into_iter(),
        &network_checker,
    )
    .await;
    Ok(res.into())
}

pub async fn check_ports_route_axum() {
    todo!()
}

#[rocket::get(
    "/v1/network/checks/ports?<interval>",
    format = "text/event-stream",
    rank = 2
)]
pub fn check_ports_stream_route<'r>(
    pod_network_config: LazyGuard<PodNetworkConfig>,
    network_checker: &'r State<NetworkChecker>,
    interval: Option<forms::Duration>,
    app_config: &'r State<AppConfig>,
) -> Result<EventStream![Event + 'r], Error> {
    let pod_network_config = pod_network_config.inner?;
    let network_checker = network_checker.inner();

    run_checks_stream(
        pod_network_config.port_reachability_checks().into_iter(),
        &network_checker,
        port_reachability_check_result,
        interval,
        app_config,
    )
}

pub async fn check_ports_stream_route_axum() {
    todo!()
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
    Event::json(&CheckResultData {
        description: check.description(),
        status,
    })
    .id(PortReachabilityCheckId::from(check).to_string())
    .event(NetworkCheckEvent::PortReachabilityCheckResult.to_string())
}
