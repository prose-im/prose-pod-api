// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use async_trait::async_trait;
use hickory_resolver::{error::ResolveError, Resolver};
use lazy_static::lazy_static;
use tracing::{debug, trace};

use crate::model::dns::DnsRecord;

use super::network_checker::{DnsLookupError, NetworkCheckerImpl};
use super::SrvLookupResponse;

lazy_static! {
    /// NOTE: [`Resolver::default`] uses Google as the resolver… which is… unexpected…
    ///   so we use [`Resolver::from_system_conf`] explicitly.
    static ref RESOLVER: Resolver = Resolver::from_system_conf().unwrap();
}

#[derive(Debug)]
pub struct LiveNetworkChecker;

#[async_trait]
impl NetworkCheckerImpl for LiveNetworkChecker {
    async fn ipv4_lookup(&self, domain: &str) -> Result<Vec<DnsRecord>, DnsLookupError> {
        let domain = domain.to_string();
        let ipv4_lookup = tokio::task::spawn_blocking(move || RESOLVER.ipv4_lookup(domain))
            .await
            .expect("Join error")?;
        Ok(ipv4_lookup
            .as_lookup()
            .record_iter()
            .flat_map(|r| DnsRecord::try_from(r).ok())
            .collect())
    }
    async fn ipv6_lookup(&self, domain: &str) -> Result<Vec<DnsRecord>, DnsLookupError> {
        let domain = domain.to_string();
        let ipv6_lookup = tokio::task::spawn_blocking(move || RESOLVER.ipv6_lookup(domain))
            .await
            .expect("Join error")?;
        Ok(ipv6_lookup
            .as_lookup()
            .record_iter()
            .flat_map(|r| DnsRecord::try_from(r).ok())
            .collect())
    }
    async fn srv_lookup(&self, domain: &str) -> Result<SrvLookupResponse, DnsLookupError> {
        let domain = domain.to_string();
        let srv_lookup = tokio::task::spawn_blocking(move || RESOLVER.srv_lookup(domain))
            .await
            .expect("Join error")?;
        Ok(SrvLookupResponse {
            records: srv_lookup
                .as_lookup()
                .record_iter()
                .map(DnsRecord::try_from)
                .flatten()
                .collect(),
            recursively_resolved_ips: srv_lookup.ip_iter().collect(),
            srv_targets: srv_lookup.iter().map(|rec| rec.target()).cloned().collect(),
        })
    }

    fn is_reachable(&self, addr: SocketAddr) -> bool {
        trace!("Checking if {addr} is reachable…");
        let reachable = TcpStream::connect_timeout(&addr, Duration::from_secs(3)).is_ok();
        trace!("{addr} reachable: {reachable}");
        reachable
    }

    async fn is_ipv4_available(&self, host: &str) -> bool {
        let host = host.to_string();
        let ipv4_lookup = match tokio::task::spawn_blocking(move || RESOLVER.ipv4_lookup(host))
            .await
            .expect("Join error")
        {
            Ok(ipv4_lookup) => ipv4_lookup,
            Err(err) => {
                debug!("IPv4 lookup failed: {err}");
                return false;
            }
        };
        !ipv4_lookup.as_lookup().records().is_empty()
    }
    async fn is_ipv6_available(&self, host: &str) -> bool {
        let host = host.to_string();
        let ipv6_lookup = match tokio::task::spawn_blocking(move || RESOLVER.ipv6_lookup(host))
            .await
            .expect("Join error")
        {
            Ok(ipv6_lookup) => ipv6_lookup,
            Err(err) => {
                debug!("IPv6 lookup failed: {err}");
                return false;
            }
        };
        !ipv6_lookup.as_lookup().records().is_empty()
    }
}

impl From<ResolveError> for DnsLookupError {
    fn from(err: ResolveError) -> Self {
        Self(err.to_string())
    }
}
