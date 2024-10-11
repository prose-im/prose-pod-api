// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use futures::{stream::FuturesOrdered, StreamExt};
use lazy_static::lazy_static;
use rocket::{
    response::stream::{Event, EventStream},
    serde::json::Json,
    State,
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use service::{
    model::{dns::DnsEntry, network_checks::*, xmpp::XmppConnectionType, PodNetworkConfig},
    services::network_checker::*,
};
use std::{
    fmt::Display,
    sync::atomic::{AtomicUsize, Ordering},
};
use tokio::{
    sync::mpsc::{self, error::SendError},
    task::JoinSet,
    time::Duration,
};

use crate::{
    error::{self, Error},
    forms,
    guards::LazyGuard,
};

lazy_static! {
    static ref DEFAULT_RETRY_INTERVAL: Duration = Duration::from_secs(5);
}

async fn run_checks<'r, Check>(
    checks: impl Iterator<Item = Check> + 'r,
    network_checker: &'r NetworkChecker,
) -> Vec<NetworkCheckResult>
where
    Check: NetworkCheck + Send + 'static,
    Check::CheckResult: Clone + Send,
    NetworkCheckResult: From<(Check, Check::CheckResult)>,
{
    checks
        .map(|check| async move {
            let result = check.run(network_checker).await;
            NetworkCheckResult::from((check, result))
        })
        .collect::<FuturesOrdered<_>>()
        .collect()
        .await
}

fn run_checks_stream<'r, Check, Status>(
    checks: impl Iterator<Item = Check> + Clone + 'r,
    network_checker: &'r NetworkChecker,
    map_to_event: impl Fn(&Check, Status) -> Event + Copy + Send + 'static,
    retry_interval: Option<forms::Duration>,
) -> Result<EventStream![Event + 'r], Error>
where
    Check: NetworkCheck + Send + 'static,
    Check::CheckResult: RetryableNetworkCheckResult + Clone + Send,
    Status: From<Check::CheckResult> + Default,
{
    let retry_interval =
        retry_interval.map_or_else(|| Ok(*DEFAULT_RETRY_INTERVAL), validate_retry_interval)?;

    Ok(EventStream! {
        fn logged(event: Event) -> Event {
            trace!("Sending {event:?}…");
            event
        }

        let (tx, mut rx) = mpsc::channel::<Option<Event>>(32);
        let mut join_set = JoinSet::<Result<(), SendError<Option<Event>>>>::new();

        let remaining = AtomicUsize::new(checks.clone().count());
        network_checker.run_checks(
            checks,
            map_to_event,
            retry_interval.into(),
            tx,
            &mut join_set,
        );

        while remaining.load(Ordering::Relaxed) != 0 {
            match rx.recv().await {
                Some(Some(event)) => yield logged(event),
                Some(None) => { remaining.fetch_sub(1, Ordering::Relaxed); },
                None => {},
            }
        }

        yield logged(end_event());
    })
}

#[get(
    "/v1/network/checks?<interval>",
    format = "text/event-stream",
    rank = 2
)]
pub(super) fn check_network_configuration<'r>(
    pod_network_config: LazyGuard<PodNetworkConfig>,
    network_checker: &'r State<NetworkChecker>,
    interval: Option<forms::Duration>,
) -> Result<EventStream![Event + 'r], Error> {
    let pod_network_config = pod_network_config.inner?;
    let network_checker = network_checker.inner();

    let retry_interval =
        interval.map_or_else(|| Ok(*DEFAULT_RETRY_INTERVAL), validate_retry_interval)?;

    Ok(EventStream! {
        fn logged(event: Event) -> Event {
            trace!("Sending {event:?}…");
            event
        }

        let (tx, mut rx) = mpsc::channel::<Option<Event>>(32);
        let mut join_set = JoinSet::<Result<(), SendError<Option<Event>>>>::new();

        let dns_record_checks = pod_network_config.dns_record_checks();
        let port_reachability_checks = pod_network_config.port_reachability_checks();
        let ip_connectivity_checks = pod_network_config.ip_connectivity_checks();

        let remaining = AtomicUsize::new(
            dns_record_checks.clone().count()
            + port_reachability_checks.len()
            + ip_connectivity_checks.len()
        );
        network_checker.run_checks(
            dns_record_checks,
            dns_record_check_result,
            retry_interval,
            tx.clone(),
            &mut join_set,
        );
        network_checker.run_checks(
            port_reachability_checks.into_iter(),
            port_reachability_check_result,
            retry_interval,
            tx.clone(),
            &mut join_set,
        );
        network_checker.run_checks(
            ip_connectivity_checks.into_iter(),
            ip_connectivity_check_result,
            retry_interval,
            tx.clone(),
            &mut join_set,
        );

        while remaining.load(Ordering::Relaxed) != 0 {
            match rx.recv().await {
                Some(Some(event)) => yield logged(event),
                Some(None) => { remaining.fetch_sub(1, Ordering::Relaxed); },
                None => break,
            }
        }

        yield logged(end_event());
    })
}

#[get("/v1/network/checks/dns", format = "application/json")]
pub(super) async fn check_dns_records<'r>(
    pod_network_config: LazyGuard<PodNetworkConfig>,
    network_checker: &'r State<NetworkChecker>,
) -> Result<Json<Vec<NetworkCheckResult>>, Error> {
    let pod_network_config = pod_network_config.inner?;
    let network_checker = network_checker.inner();

    let res = run_checks(pod_network_config.dns_record_checks(), &network_checker).await;
    Ok(res.into())
}

#[get(
    "/v1/network/checks/dns?<interval>",
    format = "text/event-stream",
    rank = 2
)]
pub(super) fn check_dns_records_stream<'r>(
    pod_network_config: LazyGuard<PodNetworkConfig>,
    network_checker: &'r State<NetworkChecker>,
    interval: Option<forms::Duration>,
) -> Result<EventStream![Event + 'r], Error> {
    let pod_network_config = pod_network_config.inner?;
    let network_checker = network_checker.inner();

    run_checks_stream(
        pod_network_config.dns_record_checks(),
        &network_checker,
        dns_record_check_result,
        interval,
    )
}

#[get("/v1/network/checks/ports", format = "application/json")]
pub(super) async fn check_ports<'r>(
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

#[get(
    "/v1/network/checks/ports?<interval>",
    format = "text/event-stream",
    rank = 2
)]
pub(super) fn check_ports_stream<'r>(
    pod_network_config: LazyGuard<PodNetworkConfig>,
    network_checker: &'r State<NetworkChecker>,
    interval: Option<forms::Duration>,
) -> Result<EventStream![Event + 'r], Error> {
    let pod_network_config = pod_network_config.inner?;
    let network_checker = network_checker.inner();

    run_checks_stream(
        pod_network_config.port_reachability_checks().into_iter(),
        &network_checker,
        port_reachability_check_result,
        interval,
    )
}

#[get("/v1/network/checks/ip", format = "application/json")]
pub(super) async fn check_ip<'r>(
    pod_network_config: LazyGuard<PodNetworkConfig>,
    network_checker: &'r State<NetworkChecker>,
) -> Result<Json<Vec<NetworkCheckResult>>, Error> {
    let pod_network_config = pod_network_config.inner?;
    let network_checker = network_checker.inner();

    let res = run_checks(
        pod_network_config.ip_connectivity_checks().into_iter(),
        &network_checker,
    )
    .await;
    Ok(res.into())
}

#[get(
    "/v1/network/checks/ip?<interval>",
    format = "text/event-stream",
    rank = 2
)]
pub(super) async fn check_ip_stream<'r>(
    pod_network_config: LazyGuard<PodNetworkConfig>,
    network_checker: &'r State<NetworkChecker>,
    interval: Option<forms::Duration>,
) -> Result<EventStream![Event + 'r], Error> {
    let pod_network_config = pod_network_config.inner?;
    let network_checker = network_checker.inner();

    run_checks_stream(
        pod_network_config.ip_connectivity_checks().into_iter(),
        &network_checker,
        ip_connectivity_check_result,
        interval,
    )
}

// ===== JSON RESPONSES =====

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkCheckResult {
    id: String,
    event: NetworkCheckEvent,
    data: serde_json::Value,
}

impl NetworkCheckResult {
    pub fn new<'a, Id, Check, Status>(check: &'a Check, status: Status) -> Self
    where
        Id: From<&'a Check> + Display,
        Check: NetworkCheck,
        Status: Serialize,
        NetworkCheckEvent: From<&'a Check>,
    {
        let data = CheckResultData {
            description: check.description(),
            status,
        };
        Self {
            event: NetworkCheckEvent::from(&check),
            id: Id::from(check.to_owned()).to_string(),
            data: serde_json::to_value(&data).unwrap_or_default(),
        }
    }
}

macro_rules! impl_network_check_result_from {
    ($check:ty, $result:ty, $status:ty, $id:ty) => {
        impl From<($check, $result)> for NetworkCheckResult {
            fn from((check, result): ($check, $result)) -> Self {
                Self::new::<$id, $check, $status>(&check, <$status>::from(result))
            }
        }
    };
}

impl_network_check_result_from!(
    DnsRecordCheck,
    DnsRecordCheckResult,
    DnsRecordStatus,
    DnsRecordCheckId
);
impl_network_check_result_from!(
    PortReachabilityCheck,
    PortReachabilityCheckResult,
    PortReachabilityStatus,
    PortReachabilityCheckId
);
impl_network_check_result_from!(
    IpConnectivityCheck,
    IpConnectivityCheckResult,
    IpConnectivityStatus,
    IpConnectivityCheckId
);

// ===== EVENTS =====

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(strum::Display)]
#[strum(serialize_all = "kebab-case")]
pub enum NetworkCheckEvent {
    DnsRecordCheckResult,
    PortReachabilityCheckResult,
    IpConnectivityCheckResult,
}

macro_rules! impl_network_check_event_from {
    ($check:ty, $result:expr) => {
        impl From<&$check> for NetworkCheckEvent {
            fn from(_: &$check) -> Self {
                $result
            }
        }
    };
}

impl_network_check_event_from!(DnsRecordCheck, Self::DnsRecordCheckResult);
impl_network_check_event_from!(PortReachabilityCheck, Self::PortReachabilityCheckResult);
impl_network_check_event_from!(IpConnectivityCheck, Self::IpConnectivityCheckResult);

#[derive(Debug, Serialize, Deserialize)]
struct CheckResultData<Status> {
    description: String,
    status: Status,
}

#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum DnsRecordStatus {
    Checking,
    Valid,
    PartiallyValid,
    Invalid,
}

impl Default for DnsRecordStatus {
    fn default() -> Self {
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

#[derive(Debug)]
#[derive(strum::Display)]
enum DnsRecordCheckId {
    #[strum(to_string = "IPv4")]
    Ipv4,
    #[strum(to_string = "IPv6")]
    Ipv6,
    #[strum(to_string = "SRV-c2s")]
    SrvC2S,
    #[strum(to_string = "SRV-s2s")]
    SrvS2S,
}

impl From<&DnsRecordCheck> for DnsRecordCheckId {
    fn from(check: &DnsRecordCheck) -> Self {
        match check.dns_entry {
            DnsEntry::Ipv4 { .. } => Self::Ipv4,
            DnsEntry::Ipv6 { .. } => Self::Ipv6,
            DnsEntry::SrvC2S { .. } => Self::SrvC2S,
            DnsEntry::SrvS2S { .. } => Self::SrvS2S,
        }
    }
}

fn dns_record_check_result(check: &DnsRecordCheck, status: DnsRecordStatus) -> Event {
    Event::json(&CheckResultData {
        description: check.description(),
        status,
    })
    .id(DnsRecordCheckId::from(check).to_string())
    .event(NetworkCheckEvent::DnsRecordCheckResult.to_string())
}

#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum PortReachabilityStatus {
    Checking,
    Open,
    Closed,
}

impl Default for PortReachabilityStatus {
    fn default() -> Self {
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

#[derive(Debug)]
#[derive(strum::Display)]
enum PortReachabilityCheckId {
    #[strum(to_string = "TCP-c2s")]
    TcpC2S,
    #[strum(to_string = "TCP-s2s")]
    TcpS2S,
    #[strum(to_string = "TCP-HTTPS")]
    TcpHttps,
}

impl From<&PortReachabilityCheck> for PortReachabilityCheckId {
    fn from(check: &PortReachabilityCheck) -> Self {
        match check {
            PortReachabilityCheck::Xmpp {
                conn_type: XmppConnectionType::C2S,
                ..
            } => Self::TcpC2S,
            PortReachabilityCheck::Xmpp {
                conn_type: XmppConnectionType::S2S,
                ..
            } => Self::TcpS2S,
            PortReachabilityCheck::Https { .. } => Self::TcpHttps,
        }
    }
}

fn port_reachability_check_result(
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

#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum IpConnectivityStatus {
    Checking,
    Success,
    Failure,
    Missing,
}

impl Default for IpConnectivityStatus {
    fn default() -> Self {
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

#[derive(Debug)]
#[derive(strum::Display)]
enum IpConnectivityCheckId {
    #[strum(to_string = "IPv4-c2s")]
    Ipv4C2S,
    #[strum(to_string = "IPv6-c2s")]
    Ipv6C2S,
    #[strum(to_string = "IPv4-s2s")]
    Ipv4S2S,
    #[strum(to_string = "IPv6-s2s")]
    Ipv6S2S,
}

impl From<&IpConnectivityCheck> for IpConnectivityCheckId {
    fn from(check: &IpConnectivityCheck) -> Self {
        match check {
            IpConnectivityCheck::XmppServer {
                conn_type: XmppConnectionType::C2S,
                ip_version: IpVersion::V4,
                ..
            } => Self::Ipv4C2S,
            IpConnectivityCheck::XmppServer {
                conn_type: XmppConnectionType::C2S,
                ip_version: IpVersion::V6,
                ..
            } => Self::Ipv6C2S,
            IpConnectivityCheck::XmppServer {
                conn_type: XmppConnectionType::S2S,
                ip_version: IpVersion::V4,
                ..
            } => Self::Ipv4S2S,
            IpConnectivityCheck::XmppServer {
                conn_type: XmppConnectionType::S2S,
                ip_version: IpVersion::V6,
                ..
            } => Self::Ipv6S2S,
        }
    }
}

fn ip_connectivity_check_result(
    check: &IpConnectivityCheck,
    status: IpConnectivityStatus,
) -> Event {
    Event::json(&CheckResultData {
        description: check.description(),
        status,
    })
    .id(IpConnectivityCheckId::from(check).to_string())
    .event(NetworkCheckEvent::IpConnectivityCheckResult.to_string())
}

fn end_event() -> Event {
    Event::empty()
        .event("end")
        .id("end")
        .with_comment("End of stream")
}

/// Check that the retry interval is between 1 second and 1 minute (inclusive).
fn validate_retry_interval(interval: forms::Duration) -> Result<Duration, Error> {
    let interval_is_max_1_minute = || interval.num_minutes().is_some_and(|m| m <= 1.);
    let interval_is_min_1_second = || interval.num_seconds().is_some_and(|s| s >= 1.);

    let interval_is_valid = interval_is_max_1_minute() && interval_is_min_1_second();

    if interval_is_valid {
        // NOTE: We can force unwrap here because `to_std` only returns `None` if `Duration` contains `year` or `month`,
        //   which is impossible due to previous checks.
        Ok(interval.to_std().unwrap())
    } else {
        Err(error::BadRequest {
            reason: "Invalid retry interval. Authorized values must be between 1 second and 1 minute (inclusive).".to_string()
        }.into())
    }
}
