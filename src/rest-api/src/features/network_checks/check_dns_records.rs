// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{onboarding, sea_orm::DatabaseConnection};
use tracing::{trace, warn};

use super::{model::*, prelude::*, util::*};

pub(super) async fn check_dns_records_route_(
    pod_network_config: PodNetworkConfig,
    network_checker: NetworkChecker,
    db: &DatabaseConnection,
) -> Vec<NetworkCheckResult> {
    let check_results = run_checks(
        pod_network_config.dns_record_checks(),
        &network_checker,
        |(check, result)| {
            (
                !result.is_failure(),
                NetworkCheckResult::from((check, result)),
            )
        },
    )
    .await;

    if check_results.iter().all(|(is_success, _)| *is_success) {
        trace!("Setting `all_dns_checks_passed_once` to true…");
        (onboarding::all_dns_checks_passed_once::set(db, true).await)
            .inspect_err(|err| warn!("Could not set `all_dns_checks_passed_once` to true: {err}"))
            .ok();
    }

    (check_results.into_iter())
        .map(|(_, res)| res)
        .collect::<Vec<_>>()
}

pub async fn check_dns_records_route(
    State(AppState { db, .. }): State<AppState>,
    pod_network_config: PodNetworkConfig,
    network_checker: NetworkChecker,
) -> Result<Json<Vec<NetworkCheckResult>>, Error> {
    let res = check_dns_records_route_(pod_network_config, network_checker, &db).await;
    Ok(Json(res))
}

pub async fn check_dns_records_stream_route(
    pod_network_config: PodNetworkConfig,
    network_checker: NetworkChecker,
    Query(forms::Interval { interval }): Query<forms::Interval>,
    State(AppState { app_config, db, .. }): State<AppState>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Error> {
    let ref app_config = app_config.read().unwrap().clone();
    run_checks_stream(
        pod_network_config.dns_record_checks(),
        network_checker,
        dns_record_check_result,
        interval,
        app_config,
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
    )
}

// MODEL

#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Serialize)]
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
    Event::default()
        .event(NetworkCheckEvent::DnsRecordCheckResult.to_string())
        .id(DnsRecordCheckId::from(check).to_string())
        .json_data(CheckResultData {
            description: check.description(),
            status,
        })
        .unwrap()
}
