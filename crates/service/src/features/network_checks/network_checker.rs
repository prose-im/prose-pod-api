// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    fmt::Debug,
    net::{IpAddr, SocketAddr, ToSocketAddrs as _},
    ops::Deref,
    sync::Arc,
};

use async_trait::async_trait;
use hickory_proto::rr::Name as DomainName;
use linked_hash_set::LinkedHashSet;
use tokio::{
    sync::mpsc::{error::SendError, Sender},
    task::JoinSet,
};
use tracing::{debug, error, instrument, trace_span, Instrument as _};

use crate::util::ConcurrentTaskRunner;

use super::models::{dns::*, network_checks::*};

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
    #[instrument(level = "trace", skip(self, checks, map_to_event, sender, runner), fields(r#type = Check::check_type()))]
    pub fn run_checks<'a, Check, Status, Event>(
        &self,
        checks: impl Iterator<Item = Check>,
        map_to_event: impl Fn(&Check, Status) -> Event + Copy + Send + 'static,
        sender: Sender<Event>,
        runner: &ConcurrentTaskRunner,
    ) where
        Check: NetworkCheck + Debug + Clone + Send + 'static + Sync,
        Check::CheckResult: RetryableNetworkCheckResult + Clone + Send,
        Status: From<Check::CheckResult> + Default + Send,
        Event: Send + 'static,
    {
        let network_checker = self.clone();
        let mut rx = runner.run_with_retries(
            checks.collect(),
            move |check: Check| {
                let network_checker = network_checker.clone();

                Box::pin(async move {
                    let result = check.run(&network_checker).await;
                    (check, Some(result))
                })
            },
            Some(|check: &Check| (check.clone(), None)),
            move || {},
            |(_, res)| res.as_ref().unwrap().should_retry(),
        );

        tokio::spawn(
            async move {
                while let Some((check, result)) = rx.recv().await {
                    let event = map_to_event(&check, result.map(Status::from).unwrap_or_default());
                    if let Err(err) = sender.send(event).await {
                        if sender.is_closed() {
                            debug!("Cannot send event: Task aborted.");
                        } else {
                            error!("Cannot send event: {err}");
                        }
                        // Close `rx` so this loop breaks and `runner` is informed to cancel its tasks.
                        rx.close();
                    }
                }
            }
            .instrument(trace_span!("task")),
        );
    }

    #[instrument(level = "trace", skip(self, checks, map_to_event, sender, join_set), fields(r#type = Check::check_type()))]
    pub fn run_checks_oneshot<'a, Check, Status, Event>(
        &self,
        checks: impl Iterator<Item = Check>,
        map_to_event: impl Fn(&Check, Status) -> Event + Copy + Send + 'static,
        sender: Sender<Event>,
        join_set: &mut JoinSet<Result<(), SendError<Event>>>,
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
                let result = check.run(&network_checker).await;
                tx_clone
                    .send(map_to_event(&check, Status::from(result.clone())))
                    .await
            });
        }
    }
}
