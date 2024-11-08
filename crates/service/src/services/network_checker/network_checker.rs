// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    fmt::Debug,
    net::{IpAddr, SocketAddr, ToSocketAddrs as _},
    ops::Deref,
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use hickory_proto::rr::Name as DomainName;
use linked_hash_set::LinkedHashSet;
use tokio::{
    sync::mpsc::{error::SendError, Sender},
    task::JoinSet,
    time::sleep,
};

use crate::model::{dns::*, network_checks::*};

/// A service used to perform network checks (DNS resolution, ports checking…).
#[derive(Debug, Clone)]
pub struct NetworkChecker(Arc<dyn NetworkCheckerImpl>);

impl NetworkChecker {
    pub fn new(implem: Arc<dyn NetworkCheckerImpl>) -> Self {
        Self(implem)
    }
}

impl Deref for NetworkChecker {
    type Target = Arc<dyn NetworkCheckerImpl>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
pub trait NetworkCheckerImpl: Debug + Sync + Send {
    async fn ipv4_lookup(&self, host: &str) -> Result<Vec<DnsRecord>, DnsLookupError>;
    async fn ipv6_lookup(&self, host: &str) -> Result<Vec<DnsRecord>, DnsLookupError>;
    async fn srv_lookup(&self, host: &str) -> Result<SrvLookupResponse, DnsLookupError>;

    fn is_reachable(&self, addr: SocketAddr) -> bool;
    fn is_port_open(&self, host: &str, port: u16) -> bool {
        (host, port)
            .to_socket_addrs()
            .is_ok_and(|mut addrs| addrs.any(|a| self.is_reachable(a)))
    }

    async fn is_ipv4_available(&self, host: &str) -> bool;
    async fn is_ipv6_available(&self, host: &str) -> bool;
    async fn is_ip_available(&self, host: String, ip_version: IpVersion) -> bool {
        match ip_version {
            IpVersion::V4 => self.is_ipv4_available(&host).await,
            IpVersion::V6 => self.is_ipv6_available(&host).await,
        }
    }
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("DNS lookup error: {0}")]
pub struct DnsLookupError(pub String);

#[derive(Debug, Clone)]
pub struct SrvLookupResponse {
    pub records: Vec<DnsRecord>,
    pub recursively_resolved_ips: LinkedHashSet<IpAddr>,
    pub srv_targets: LinkedHashSet<DomainName>,
}

#[derive(Debug, Clone)]
pub enum IpVersion {
    V4,
    V6,
}

impl NetworkChecker {
    pub fn run_checks<'a, Check, Status, Event>(
        &self,
        checks: impl Iterator<Item = Check>,
        map_to_event: impl Fn(&Check, Status) -> Event + Copy + Send + 'static,
        retry_interval: Duration,
        sender: Sender<Option<Event>>,
        join_set: &mut JoinSet<Result<(), SendError<Option<Event>>>>,
    ) where
        Check: NetworkCheck + Send + 'static,
        Check::CheckResult: RetryableNetworkCheckResult + Clone + Send,
        Status: From<Check::CheckResult> + Default,
        Event: Send + 'static,
    {
        for check in checks {
            let tx_clone = sender.clone();
            let network_checker = self.to_owned();

            join_set.spawn(async move {
                tx_clone
                    .send(Some(map_to_event(&check, Status::default())))
                    .await?;

                loop {
                    let result = check.run(&network_checker).await;
                    tx_clone
                        .send(Some(map_to_event(&check, Status::from(result.clone()))))
                        .await?;

                    if result.should_retry() {
                        sleep(retry_interval).await;
                    } else {
                        return tx_clone.send(None).await;
                    }
                }
            });
        }
    }
}
