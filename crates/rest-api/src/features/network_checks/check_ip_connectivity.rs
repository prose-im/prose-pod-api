// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::{model::*, prelude::*, util::*};

#[rocket::get("/v1/network/checks/ip", format = "application/json")]
pub async fn check_ip_route<'r>(
    pod_network_config: LazyGuard<PodNetworkConfig>,
    network_checker: &'r StateRocket<NetworkChecker>,
) -> Result<JsonRocket<Vec<NetworkCheckResult>>, Error> {
    let pod_network_config = pod_network_config.inner?;
    let network_checker = network_checker.inner();

    let res = run_checks(
        pod_network_config.ip_connectivity_checks().into_iter(),
        &network_checker,
    )
    .await;
    Ok(res.into())
}

pub async fn check_ip_route_axum(
    pod_network_config: PodNetworkConfig,
    network_checker: NetworkChecker,
) -> Result<Json<Vec<NetworkCheckResult>>, Error> {
    let res = run_checks(
        pod_network_config.ip_connectivity_checks().into_iter(),
        &network_checker,
    )
    .await;
    Ok(Json(res))
}

#[rocket::get(
    "/v1/network/checks/ip?<interval>",
    format = "text/event-stream",
    rank = 2
)]
pub async fn check_ip_stream_route<'r>(
    pod_network_config: LazyGuard<PodNetworkConfig>,
    network_checker: &'r StateRocket<NetworkChecker>,
    interval: Option<forms::Duration>,
    app_config: &'r StateRocket<AppConfig>,
) -> Result<EventStream![EventRocket + 'r], Error> {
    let pod_network_config = pod_network_config.inner?;
    let network_checker = network_checker.inner();

    run_checks_stream_rocket(
        pod_network_config.ip_connectivity_checks().into_iter(),
        &network_checker,
        ip_connectivity_check_result_rocket,
        interval,
        app_config,
    )
}

pub async fn check_ip_stream_route_axum(
    pod_network_config: PodNetworkConfig,
    network_checker: NetworkChecker,
    Query(forms::Interval { interval }): Query<forms::Interval>,
    State(AppState { app_config, .. }): State<AppState>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Error> {
    run_checks_stream(
        pod_network_config.ip_connectivity_checks().into_iter(),
        network_checker,
        ip_connectivity_check_result,
        interval.map(forms::Duration),
        app_config,
    )
}

// MODEL

#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IpConnectivityStatus {
    Queued,
    Checking,
    Success,
    Failure,
    Missing,
}

// BOILERPLATE

impl_network_check_result_from!(
    IpConnectivityCheck,
    IpConnectivityCheckResult,
    IpConnectivityStatus,
    IpConnectivityCheckId
);
impl_network_check_event_from!(IpConnectivityCheck, Self::IpConnectivityCheckResult);

impl WithQueued for IpConnectivityStatus {
    fn queued() -> Self {
        Self::Queued
    }
}
impl WithChecking for IpConnectivityStatus {
    fn checking() -> Self {
        Self::Checking
    }
}

impl From<IpConnectivityCheckResult> for IpConnectivityStatus {
    fn from(value: IpConnectivityCheckResult) -> Self {
        match value {
            IpConnectivityCheckResult::Success => Self::Success,
            IpConnectivityCheckResult::Failure => Self::Failure,
            IpConnectivityCheckResult::Missing => Self::Missing,
        }
    }
}

pub fn ip_connectivity_check_result_rocket(
    check: &IpConnectivityCheck,
    status: IpConnectivityStatus,
) -> EventRocket {
    EventRocket::json(&CheckResultData {
        description: check.description(),
        status,
    })
    .id(IpConnectivityCheckId::from(check).to_string())
    .event(NetworkCheckEvent::IpConnectivityCheckResult.to_string())
}

pub fn ip_connectivity_check_result(
    check: &IpConnectivityCheck,
    status: IpConnectivityStatus,
) -> Event {
    Event::default()
        .event(NetworkCheckEvent::IpConnectivityCheckResult.to_string())
        .id(IpConnectivityCheckId::from(check).to_string())
        .json_data(&CheckResultData {
            description: check.description(),
            status,
        })
        .unwrap()
}
