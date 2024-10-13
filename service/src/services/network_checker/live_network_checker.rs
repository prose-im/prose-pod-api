// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use hickory_resolver::{error::ResolveError, Resolver};
use lazy_static::lazy_static;
use tracing::debug;

use crate::model::dns::DnsRecord;

use super::network_checker::{DnsLookupError, NetworkCheckerImpl};

lazy_static! {
    /// NOTE: [`Resolver::default`] uses Google as the resolver… which is… unexpected…
    ///   so we use [`Resolver::from_system_conf`] explicitly.
    static ref RESOLVER: Resolver = Resolver::from_system_conf().unwrap();
}

#[derive(Debug)]
pub struct LiveNetworkChecker;

impl NetworkCheckerImpl for LiveNetworkChecker {
    fn ipv4_lookup(&self, domain: &str) -> Result<Vec<DnsRecord>, DnsLookupError> {
        let ipv4_lookup = RESOLVER.ipv4_lookup(domain)?;
        Ok(ipv4_lookup
            .as_lookup()
            .record_iter()
            .flat_map(|r| DnsRecord::try_from(r).ok())
            .collect())
    }
    fn ipv6_lookup(&self, domain: &str) -> Result<Vec<DnsRecord>, DnsLookupError> {
        let ipv6_lookup = RESOLVER.ipv6_lookup(domain)?;
        Ok(ipv6_lookup
            .as_lookup()
            .record_iter()
            .flat_map(|r| DnsRecord::try_from(r).ok())
            .collect())
    }
    fn srv_lookup(&self, domain: &str) -> Result<Vec<DnsRecord>, DnsLookupError> {
        let srv_lookup = RESOLVER.srv_lookup(domain)?;
        Ok(srv_lookup
            .as_lookup()
            .record_iter()
            .flat_map(|r| DnsRecord::try_from(r).ok())
            .collect())
    }

    fn is_port_open(&self, host: &str, port_number: u32) -> bool {
        let Some(mut addrs) = (host, port_number as u16).to_socket_addrs().ok() else {
            return false;
        };

        addrs.any(|addr| TcpStream::connect_timeout(&addr, Duration::from_secs(3)).is_ok())
    }

    fn is_ipv4_available(&self, host: &str) -> bool {
        let ipv4_lookup = match RESOLVER.ipv4_lookup(host) {
            Ok(ipv4_lookup) => ipv4_lookup,
            Err(err) => {
                debug!("IPv4 lookup failed: {err}");
                return false;
            }
        };
        !ipv4_lookup.as_lookup().records().is_empty()
    }
    fn is_ipv6_available(&self, host: &str) -> bool {
        let ipv6_lookup = match RESOLVER.ipv6_lookup(host) {
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
