// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs as _},
    str::FromStr as _,
    sync::{Arc, LazyLock},
    time::{Duration, Instant},
};

use async_trait::async_trait;
use hickory_proto::rr::RecordType;
use hickory_resolver::{
    config::{NameServerConfigGroup, ResolverConfig, ResolverOpts},
    lookup::NsLookup,
    name_server::TokioConnectionProvider,
    Name as DomainName, ResolveError, TokioResolver,
};
use parking_lot::RwLock;
use tracing::{debug, trace, warn};

use crate::app_config::NetworkChecksConfig;

use super::{DnsLookupError, DnsRecord, NetworkCheckerImpl, SrvLookupResponse};

/// NOTE: [`Resolver::default`] uses Google as the resolver… which is… unexpected…
///   so we use [`Resolver::from_system_conf`] explicitly.
static SYSTEM_RESOLVER: LazyLock<Arc<TokioResolver>> =
    LazyLock::new(|| Arc::new(TokioResolver::builder_tokio().unwrap().build()));

/// NOTE: [`Debug`] is implemented by hand, make sure to update it when adding new fields.
pub struct LiveNetworkChecker {
    /// Caches non-recursive DNS resolvers by domain name, along with the time it was cached at
    /// to allow cache expiry.
    direct_resolvers: Arc<RwLock<HashMap<DomainName, (Instant, Arc<TokioResolver>)>>>,
    dns_cache_ttl: Duration,
}

impl LiveNetworkChecker {
    pub fn from_config(config: &NetworkChecksConfig) -> Self {
        Self {
            direct_resolvers: Default::default(),
            dns_cache_ttl: config.dns_cache_ttl.into_std_duration(),
        }
    }

    /// A DNS resolver which queries the name servers directly and stores no cache.
    async fn direct_resolver(&self, domain: &DomainName) -> Arc<TokioResolver> {
        // Read the cache to avoid unnecessary DNS queries.
        {
            let mut resolvers_guard = self.direct_resolvers.upgradable_read();
            if let Some((cached_at, resolver)) = resolvers_guard.get(domain) {
                if cached_at.elapsed() < self.dns_cache_ttl {
                    return Arc::clone(&resolver);
                } else {
                    // Clear the cache if it's expired.
                    resolvers_guard.with_upgraded(|r| r.remove(domain));
                }
            }
        }

        /// Creates a DNS resolver which queries the name servers directly and stores no cache.
        async fn create_direct_resolver(domain: &DomainName) -> Arc<TokioResolver> {
            /// Recursively queries the authoritative name servers for the domain.
            async fn recursive_ns_lookup(
                resolver: &TokioResolver,
                mut domain: DomainName,
            ) -> Result<NsLookup, ResolveError> {
                let mut first_error: Option<ResolveError> = None;
                loop {
                    match resolver.ns_lookup(domain.clone()).await {
                        Ok(res) => {
                            trace!("Found NS records for `{domain}`.");
                            return Ok(res);
                        }
                        Err(err) => {
                            let first_error = first_error.get_or_insert(err);
                            domain = domain.base_name();
                            if domain.is_root() {
                                return Err(first_error.clone());
                            }
                        }
                    }
                }
            }
            let Ok(ns_response) = recursive_ns_lookup(&SYSTEM_RESOLVER, domain.base_name()).await
            else {
                warn!("No authoritative name server found for `{domain}` (reached `.`). Will use the system-defined DNS name servers to run DNS checks.");
                // NOTE: This scenario should never happen, because the TLD should always have an authoritative
                //   name server, but as a safe fallback we return the recursive system-defined DNS resolver.
                //   Results won't be as good because of DNS caching at multiple layers, but at least there
                //   will be results.
                return Arc::clone(&SYSTEM_RESOLVER);
            };

            if ns_response.iter().next().is_none() {
                warn!("No authoritative name server found for `{domain}` (response is empty). Will use the system-defined DNS name servers to run DNS checks.");
                // NOTE: This scenario should never happen, because the TLD should always have an authoritative
                //   name server, but as a safe fallback we return the recursive system-defined DNS resolver.
                //   Results won't be as good because of DNS caching at multiple layers, but at least there
                //   will be results.
                return Arc::clone(&SYSTEM_RESOLVER);
            }

            // Resolve the IP addresses of the authoritative name servers.
            trace!(
                "Authoritative name servers for `{domain}`: {:?}",
                ns_response.iter().collect::<Vec<_>>(),
            );
            let mut name_servers: Vec<IpAddr> = Vec::with_capacity(ns_response.iter().count());
            for ns in ns_response.iter() {
                match SYSTEM_RESOLVER.lookup_ip(ns.0.clone()).await {
                    Ok(ips) => name_servers.extend(ips.iter()),
                    Err(_) => {}
                }
            }

            // Create the DNS resolver.
            let config = ResolverConfig::from_parts(
                None,
                vec![],
                NameServerConfigGroup::from_ips_clear(name_servers.as_slice(), 53, false),
            );
            let mut options = ResolverOpts::default();
            options.recursion_desired = false;
            options.cache_size = 0;
            Arc::new(
                TokioResolver::builder_with_config(config, TokioConnectionProvider::default())
                    .with_options(options)
                    .build(),
            )
        }
        let domain_clone = domain.clone();
        let resolver = create_direct_resolver(&domain_clone).await;

        // Cache the resolver for faster query next time.
        self.direct_resolvers
            .write()
            .insert(domain.clone(), (Instant::now(), resolver.clone()));

        resolver
    }

    fn is_reachable(&self, addr: SocketAddr) -> bool {
        trace!("Checking if {addr} is reachable…");
        let reachable = TcpStream::connect_timeout(&addr, Duration::from_secs(3)).is_ok();
        trace!("{addr} reachable: {reachable}");
        reachable
    }
}

#[async_trait]
impl NetworkCheckerImpl for LiveNetworkChecker {
    async fn ipv4_lookup(&self, domain: &str) -> Result<Vec<DnsRecord>, DnsLookupError> {
        let domain = DomainName::from_str(domain)
            .map_err(|err| DnsLookupError(format!("Invalid domain name `{domain}`: {err}")))?;
        let direct_resolver = self.direct_resolver(&domain).await;

        let domain = domain.to_string();
        let ipv4_lookup = direct_resolver.ipv4_lookup(domain).await?;
        Ok((ipv4_lookup.as_lookup())
            .record_iter()
            .flat_map(|r| DnsRecord::try_from(r).ok())
            .collect())
    }
    async fn ipv6_lookup(&self, domain: &str) -> Result<Vec<DnsRecord>, DnsLookupError> {
        let domain = DomainName::from_str(domain)
            .map_err(|err| DnsLookupError(format!("Invalid domain name `{domain}`: {err}")))?;
        let direct_resolver = self.direct_resolver(&domain).await;

        let domain = domain.to_string();
        let ipv6_lookup = direct_resolver.ipv6_lookup(domain).await?;
        Ok((ipv6_lookup.as_lookup())
            .record_iter()
            .flat_map(|r| DnsRecord::try_from(r).ok())
            .collect())
    }
    async fn srv_lookup(&self, domain: &str) -> Result<SrvLookupResponse, DnsLookupError> {
        let domain = DomainName::from_str(domain)
            .map_err(|err| DnsLookupError(format!("Invalid domain name `{domain}`: {err}")))?;
        let direct_resolver = self.direct_resolver(&domain).await;

        let domain = domain.to_string();
        let srv_lookup = direct_resolver.srv_lookup(domain).await?;
        Ok(SrvLookupResponse {
            records: (srv_lookup.as_lookup())
                .record_iter()
                .map(DnsRecord::try_from)
                .flatten()
                .collect(),
            recursively_resolved_ips: srv_lookup.ip_iter().collect(),
            srv_targets: srv_lookup.iter().map(|rec| rec.target()).cloned().collect(),
        })
    }
    async fn cname_lookup(&self, domain: &str) -> Result<Vec<DnsRecord>, DnsLookupError> {
        let domain = DomainName::from_str(domain)
            .map_err(|err| DnsLookupError(format!("Invalid domain name `{domain}`: {err}")))?;
        let direct_resolver = self.direct_resolver(&domain).await;

        let domain = domain.to_string();
        let cname_lookup = direct_resolver.lookup(domain, RecordType::CNAME).await?;
        Ok(cname_lookup
            .record_iter()
            .flat_map(|r| {
                DnsRecord::try_from(r)
                    .inspect_err(|err| {
                        tracing::error!("Could not deserialize Hickory record: {err}")
                    })
                    .ok()
            })
            .collect())
    }

    fn is_port_open(&self, host: &str, port: u16) -> bool {
        (host, port)
            .to_socket_addrs()
            .is_ok_and(|mut addrs| addrs.any(|a| self.is_reachable(a)))
    }

    async fn is_ipv4_available(&self, host: &str) -> bool {
        let host = host.to_string();
        let ipv4_lookup = match SYSTEM_RESOLVER.ipv4_lookup(host).await {
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
        let ipv6_lookup = match SYSTEM_RESOLVER.ipv6_lookup(host).await {
            Ok(ipv6_lookup) => ipv6_lookup,
            Err(err) => {
                debug!("IPv6 lookup failed: {err}");
                return false;
            }
        };
        !ipv6_lookup.as_lookup().records().is_empty()
    }
}

// MARK: - Boilerplate

impl Debug for LiveNetworkChecker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiveNetworkChecker")
            .field(
                "direct_resolvers",
                &self.direct_resolvers.read().keys().collect::<HashSet<_>>(),
            )
            .finish()
    }
}

impl From<ResolveError> for DnsLookupError {
    fn from(err: ResolveError) -> Self {
        Self(err.to_string())
    }
}
