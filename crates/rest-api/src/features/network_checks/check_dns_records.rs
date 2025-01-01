// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::{model::*, prelude::*, util::*};

#[rocket::get("/v1/network/checks/dns", format = "application/json")]
pub async fn check_dns_records_route<'r>(
    pod_network_config: LazyGuard<PodNetworkConfig>,
    network_checker: &'r State<NetworkChecker>,
) -> Result<Json<Vec<NetworkCheckResult>>, Error> {
    let pod_network_config = pod_network_config.inner?;
    let network_checker = network_checker.inner();

    let res = run_checks(pod_network_config.dns_record_checks(), &network_checker).await;
    Ok(res.into())
}

pub async fn check_dns_records_route_axum() {
    todo!()
}

#[rocket::get(
    "/v1/network/checks/dns?<interval>",
    format = "text/event-stream",
    rank = 2
)]
pub fn check_dns_records_stream_route<'r>(
    pod_network_config: LazyGuard<PodNetworkConfig>,
    network_checker: &'r State<NetworkChecker>,
    interval: Option<forms::Duration>,
    app_config: &'r State<AppConfig>,
) -> Result<EventStream![Event + 'r], Error> {
    let pod_network_config = pod_network_config.inner?;
    let network_checker = network_checker.inner();

    run_checks_stream(
        pod_network_config.dns_record_checks(),
        &network_checker,
        dns_record_check_result,
        interval,
        app_config,
    )
}

pub async fn check_dns_records_stream_route_axum() {
    todo!()
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

pub fn dns_record_check_result(check: &DnsRecordCheck, status: DnsRecordStatus) -> Event {
    Event::json(&CheckResultData {
        description: check.description(),
        status,
    })
    .id(DnsRecordCheckId::from(check).to_string())
    .event(NetworkCheckEvent::DnsRecordCheckResult.to_string())
}
