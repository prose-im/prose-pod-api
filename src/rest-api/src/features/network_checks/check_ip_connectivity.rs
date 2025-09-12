// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::{model::*, prelude::*, util::*};

pub async fn check_ip_route(
    pod_network_config: PodNetworkConfig,
    network_checker: NetworkChecker,
    app_config: State<Arc<AppConfig>>,
    query: Query<forms::Interval>,
    headers: HeaderMap,
) -> Either<
    Result<Json<Vec<NetworkCheckResult>>, Error>,
    Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Error>,
> {
    match headers.get(ACCEPT) {
        Some(ct) if ct.starts_with(TEXT_EVENT_STREAM.essence_str()) => Either::E2(
            check_ip_stream_route_(pod_network_config, network_checker, app_config, query).await,
        ),
        _ => Either::E1(check_ip_route_(pod_network_config, network_checker).await),
    }
}

async fn check_ip_route_(
    pod_network_config: PodNetworkConfig,
    network_checker: NetworkChecker,
) -> Result<Json<Vec<NetworkCheckResult>>, Error> {
    let res = run_checks(
        pod_network_config.ip_connectivity_checks().into_iter(),
        &network_checker,
        NetworkCheckResult::from,
    )
    .await;
    Ok(Json(res))
}

async fn check_ip_stream_route_(
    pod_network_config: PodNetworkConfig,
    network_checker: NetworkChecker,
    State(ref app_config): State<Arc<AppConfig>>,
    Query(forms::Interval { interval }): Query<forms::Interval>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Error> {
    run_checks_stream(
        pod_network_config.ip_connectivity_checks().into_iter(),
        network_checker,
        ip_connectivity_check_result,
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
pub enum IpConnectivityStatus {
    Queued,
    Checking,
    Success,
    Failure,
}

// MARK: - Boilerplate

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
        }
    }
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
