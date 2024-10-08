// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{
    response::stream::{Event, EventStream},
    State,
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use service::{
    model::{
        dns::DnsEntry, xmpp::XmppConnectionType, DnsCheck, IpConnectivityCheck, PodNetworkConfig,
        PortReachabilityCheck,
    },
    services::network_checker::{DnsRecordStatus, IpVersion, NetworkChecker},
};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::{
    sync::mpsc::{self, error::SendError},
    task::JoinSet,
    time::{sleep, Duration},
};

use crate::{error::Error, guards::LazyGuard};

// #[get("/v1/enrich-members?<jids..>", format = "text/event-stream", rank = 2)]
// pub(super) fn enrich_members_stream<'r>(
//     member_controller: LazyGuard<MemberController<'r>>,
//     jids: Strict<JIDs>,
// ) -> Result<EventStream![Event + 'r], Error> {
//     let member_controller = member_controller.inner?;
//     let jids = jids.into_inner();

//     Ok(EventStream! {
//         fn logged(event: Event) -> Event {
//             trace!("Sending {event:?}…");
//             event
//         }

//         for jid in jids.iter() {
//             let res: EnrichedMember = member_controller.enrich_member(jid).await.into();
//             yield logged(Event::json(&res).id(jid.to_string()).event("enriched-member"));
//         }

//         yield logged(Event::empty().event("end").id("end").with_comment("End of stream"));
//     })
// }

#[get("/v1/network/checks", format = "text/event-stream", rank = 2)]
pub(super) fn check_network_configuration<'r>() -> Result<EventStream![Event + 'r], Error> {
    Ok(EventStream! {
        fn logged(event: Event) -> Event {
            trace!("Sending {event:?}…");
            event
        }

        // for jid in jids.iter() {
        //     let res: EnrichedMember = member_controller.enrich_member(jid).await.into();
        //     yield logged(Event::json(&res).id(jid.to_string()).event("enriched-member"));
        // }

        yield logged(end_event());
    })
}

#[get("/v1/network/checks/dns", format = "text/event-stream", rank = 2)]
pub(super) fn check_dns_records_stream<'r>(
    pod_network_config: LazyGuard<PodNetworkConfig>,
    network_checker: &'r State<NetworkChecker>,
) -> Result<EventStream![Event + 'r], Error> {
    let pod_network_config = pod_network_config.inner?;
    let network_checker = network_checker.inner();

    let checks = pod_network_config.dns_checks();

    let remaining = AtomicUsize::new(checks.clone().count());

    Ok(EventStream! {
        fn logged(event: Event) -> Event {
            trace!("Sending {event:?}…");
            event
        }

        let (tx, mut rx) = mpsc::channel::<Option<Event>>(32);
        let mut set = JoinSet::<Result<(), SendError<Option<Event>>>>::new();

        for check in checks {
            let tx_clone = tx.clone();
            let network_checker = network_checker.to_owned();

            set.spawn(async move {
                tx_clone.send(Some(dns_record_check_result(&check, DnsRecordCheckStatus::Checking))).await?;

                loop {
                    let status = &network_checker.check_dns_entry(check.dns_entry.clone());
                    tx_clone.send(Some(dns_record_check_result(&check, status.into()))).await?;

                    if matches!(status, DnsRecordStatus::Invalid | DnsRecordStatus::Error(_)) {
                        sleep(Duration::from_secs(1)).await;
                    } else {
                        return tx_clone.send(None).await;
                    }
                }
            });
        }

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

#[get("/v1/network/checks/ports", format = "text/event-stream", rank = 2)]
pub(super) fn check_ports_stream<'r>(
    pod_network_config: LazyGuard<PodNetworkConfig>,
    network_checker: &'r State<NetworkChecker>,
) -> Result<EventStream![Event + 'r], Error> {
    let pod_network_config = pod_network_config.inner?;
    let network_checker = network_checker.inner();

    let checks = pod_network_config.port_reachabilty_checks();

    let remaining = AtomicUsize::new(checks.clone().len());

    Ok(EventStream! {
        fn logged(event: Event) -> Event {
            trace!("Sending {event:?}…");
            event
        }

        let (tx, mut rx) = mpsc::channel::<Option<Event>>(32);
        let mut set = JoinSet::<Result<(), SendError<Option<Event>>>>::new();

        for check in checks {
            let tx_clone = tx.clone();
            let network_checker = network_checker.to_owned();

            set.spawn(async move {
                tx_clone.send(Some(port_reachability_check_result(&check, PortReachabilityStatus::Checking))).await?;

                loop {
                    let status: PortReachabilityStatus = {
                        let mut status = PortReachabilityStatus::Closed;
                        for hostname in check.hostnames().iter() {
                            if network_checker.is_port_open(&hostname.to_string(), check.port()) {
                                status = PortReachabilityStatus::Open;
                                break;
                            }
                        }
                        status
                    };

                    tx_clone.send(Some(port_reachability_check_result(&check, status))).await?;

                    if matches!(status, PortReachabilityStatus::Closed) {
                        sleep(Duration::from_secs(1)).await;
                    } else {
                        return tx_clone.send(None).await;
                    }
                }
            });
        }

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

#[get("/v1/network/checks/ip", format = "text/event-stream", rank = 2)]
pub(super) async fn check_ip_stream<'r>(
    pod_network_config: LazyGuard<PodNetworkConfig>,
    network_checker: &'r State<NetworkChecker>,
) -> Result<EventStream![Event + 'r], Error> {
    let pod_network_config = pod_network_config.inner?;
    let network_checker = network_checker.inner();

    let checks = pod_network_config.ip_connectivity_checks();

    let remaining = AtomicUsize::new(checks.clone().len());

    Ok(EventStream! {
        fn logged(event: Event) -> Event {
            trace!("Sending {event:?}…");
            event
        }

        let (tx, mut rx) = mpsc::channel::<Option<Event>>(32);
        let mut set = JoinSet::<Result<(), SendError<Option<Event>>>>::new();

        for check in checks {
            let tx_clone = tx.clone();
            let network_checker = network_checker.to_owned();

            set.spawn(async move {
                tx_clone.send(Some(ip_connectivity_check_result(&check, IpConnectivityStatus::Checking))).await?;

                loop {
                    let status: IpConnectivityStatus = {
                        let mut status = IpConnectivityStatus::Failure;
                        for hostname in check.hostnames().iter() {
                            if network_checker.is_ip_available(&hostname.to_string(), check.ip_version()) {
                                status = IpConnectivityStatus::Success;
                                break;
                            }
                        }
                        status
                    };

                    tx_clone.send(Some(ip_connectivity_check_result(&check, status))).await?;

                    if matches!(status, IpConnectivityStatus::Failure) {
                        sleep(Duration::from_secs(1)).await;
                    } else {
                        return tx_clone.send(None).await;
                    }
                }
            });
        }

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

// ===== EVENTS =====

#[derive(Debug, Serialize, Deserialize)]
struct CheckResultData<Status> {
    description: String,
    status: Status,
}

#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum DnsRecordCheckStatus {
    Checking,
    Valid,
    PartiallyValid,
    Invalid,
}

impl From<&DnsRecordStatus> for DnsRecordCheckStatus {
    fn from(status: &DnsRecordStatus) -> Self {
        match status {
            DnsRecordStatus::Valid => Self::Valid,
            DnsRecordStatus::PartiallyValid { .. } => Self::PartiallyValid,
            DnsRecordStatus::Invalid | DnsRecordStatus::Error(_) => Self::Invalid,
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

impl From<&DnsCheck> for DnsRecordCheckId {
    fn from(check: &DnsCheck) -> Self {
        match check.dns_entry {
            DnsEntry::Ipv4 { .. } => Self::Ipv4,
            DnsEntry::Ipv6 { .. } => Self::Ipv6,
            DnsEntry::SrvC2S { .. } => Self::SrvC2S,
            DnsEntry::SrvS2S { .. } => Self::SrvS2S,
        }
    }
}

fn dns_record_check_result(dns_check: &DnsCheck, status: DnsRecordCheckStatus) -> Event {
    Event::json(&CheckResultData {
        description: dns_check.description(),
        status,
    })
    .id(DnsRecordCheckId::from(dns_check).to_string())
    .event("dns-record-check-result")
}

#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum PortReachabilityStatus {
    Checking,
    Open,
    Closed,
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
    .event("port-reachability-check-result")
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
    .event("ip-connectivity-check-result")
}

fn end_event() -> Event {
    Event::empty()
        .event("end")
        .id("end")
        .with_comment("End of stream")
}
