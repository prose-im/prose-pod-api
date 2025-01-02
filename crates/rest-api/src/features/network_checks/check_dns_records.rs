// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::{model::*, prelude::*, util::*};

#[rocket::get("/v1/network/checks/dns", format = "application/json")]
pub async fn check_dns_records_route<'r>(
    pod_network_config: LazyGuard<PodNetworkConfig>,
    network_checker: &'r StateRocket<NetworkChecker>,
) -> Result<JsonRocket<Vec<NetworkCheckResult>>, Error> {
    let pod_network_config = pod_network_config.inner?;
    let network_checker = network_checker.inner();

    let res = run_checks(pod_network_config.dns_record_checks(), &network_checker).await;
    Ok(res.into())
}

pub async fn check_dns_records_route_axum(
    pod_network_config: PodNetworkConfig,
    network_checker: NetworkChecker,
) -> Result<Json<Vec<NetworkCheckResult>>, Error> {
    let res = run_checks(pod_network_config.dns_record_checks(), &network_checker).await;
    Ok(Json(res))
}

#[rocket::get(
    "/v1/network/checks/dns?<interval>",
    format = "text/event-stream",
    rank = 2
)]
pub fn check_dns_records_stream_route<'r>(
    pod_network_config: LazyGuard<PodNetworkConfig>,
    network_checker: &'r StateRocket<NetworkChecker>,
    interval: Option<forms::Duration>,
    app_config: &'r StateRocket<AppConfig>,
) -> Result<EventStream![EventRocket + 'r], Error> {
    let pod_network_config = pod_network_config.inner?;
    let network_checker = network_checker.inner();

    run_checks_stream_rocket(
        pod_network_config.dns_record_checks(),
        &network_checker,
        dns_record_check_result_rocket,
        interval,
        app_config,
    )
}

pub async fn check_dns_records_stream_route_axum(
    pod_network_config: PodNetworkConfig,
    network_checker: NetworkChecker,
    Query(forms::Interval { interval }): Query<forms::Interval>,
    State(AppState { app_config, .. }): State<AppState>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Error> {
    run_checks_stream(
        pod_network_config.dns_record_checks(),
        network_checker,
        dns_record_check_result,
        interval.map(forms::Duration),
        app_config,
    )
}

// MODEL

#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DnsRecordStatus {
    Queued,
    Checking,
    Valid,
    PartiallyValid,
    Invalid,
}

// BOILERPLATE

impl_network_check_result_from!(
    DnsRecordCheck,
    DnsRecordCheckResult,
    DnsRecordStatus,
    DnsRecordCheckId
);
impl_network_check_event_from!(DnsRecordCheck, Self::DnsRecordCheckResult);

impl WithQueued for DnsRecordStatus {
    fn queued() -> Self {
        Self::Queued
    }
}
impl WithChecking for DnsRecordStatus {
    fn checking() -> Self {
        Self::Checking
    }
}

impl From<DnsRecordCheckResult> for DnsRecordStatus {
    fn from(status: DnsRecordCheckResult) -> Self {
        match status {
            DnsRecordCheckResult::Valid => Self::Valid,
            DnsRecordCheckResult::PartiallyValid { .. } => Self::PartiallyValid,
            DnsRecordCheckResult::Invalid | DnsRecordCheckResult::Error(_) => Self::Invalid,
        }
    }
}

pub fn dns_record_check_result_rocket(
    check: &DnsRecordCheck,
    status: DnsRecordStatus,
) -> EventRocket {
    EventRocket::json(&CheckResultData {
        description: check.description(),
        status,
    })
    .id(DnsRecordCheckId::from(check).to_string())
    .event(NetworkCheckEvent::DnsRecordCheckResult.to_string())
}

pub fn dns_record_check_result(check: &DnsRecordCheck, status: DnsRecordStatus) -> Event {
    Event::default()
        .event(NetworkCheckEvent::DnsRecordCheckResult.to_string())
        .id(DnsRecordCheckId::from(check).to_string())
        .json_data(CheckResultData {
            description: check.description(),
            status,
        })
        .unwrap()
}
