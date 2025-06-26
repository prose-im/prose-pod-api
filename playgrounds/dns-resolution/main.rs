use std::{
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

#[allow(unused)]
use hickory_resolver::{
    config::ResolverConfig,
    proto::rr::{rdata, LowerName, RData, Record as HickoryRecord, RecordType},
    Resolver,
};

#[allow(unused)]
#[derive(Debug, Clone)]
enum DnsRecord {
    A {
        hostname: LowerName,
        ttl: u32,
        value: Ipv4Addr,
    },
    AAAA {
        hostname: LowerName,
        ttl: u32,
        value: Ipv6Addr,
    },
}

impl Into<HickoryRecord> for DnsRecord {
    fn into(self) -> HickoryRecord {
        match self {
            DnsRecord::A {
                hostname,
                ttl,
                value,
            } => HickoryRecord::from_rdata(hostname.into(), ttl, RData::A(rdata::A(value))),
            DnsRecord::AAAA {
                hostname,
                ttl,
                value,
            } => HickoryRecord::from_rdata(hostname.into(), ttl, RData::AAAA(rdata::AAAA(value))),
        }
    }
}

impl PartialEq<HickoryRecord> for DnsRecord {
    fn eq(&self, other: &HickoryRecord) -> bool {
        other.eq(&self.clone().into())
    }
}

fn main() {
    // test_ipv4_lookup();
    // test_ipv6_lookup();
    // test_srv_lookup();
    // check_srv_records();
    check_srv_c2s();
    check_srv_s2s();
    // test_lookup_ip();
}

#[allow(unused)]
fn test_ipv4_lookup() {
    let resolver = Resolver::from_system_conf().unwrap();
    // resolver.clear_cache();
    // Resolver::new(config, options)
    // Resolver::default()

    // ResolverConfig::new()

    let host = "test2.prose.dev.remibardon.com.";
    let ipv4_lookup = resolver.ipv4_lookup(host).unwrap();
    dbg!(ipv4_lookup.clone());
    // ipv4_lookup = Ipv4Lookup(
    //     Lookup {
    //         query: Query {
    //             name: Name("prose.org."),
    //             query_type: A,
    //             query_class: IN,
    //         },
    //         records: [
    //             Record {
    //                 name_labels: Name("prose.org."),
    //                 rr_type: A,
    //                 dns_class: IN,
    //                 ttl: 600,
    //                 rdata: Some(
    //                     A(
    //                         A(
    //                             140.82.52.95,
    //                         ),
    //                     ),
    //                 ),
    //             },
    //         ],
    //         valid_until: Instant {
    //             tv_sec: 20450,
    //             tv_nsec: 860594354,
    //         },
    //     },
    // )
    //
    // ipv4_lookup = Ipv4Lookup(
    //     Lookup {
    //         query: Query {
    //             name: Name("test2.prose.dev.remibardon.com."),
    //             query_type: A,
    //             query_class: IN,
    //         },
    //         records: [
    //             Record {
    //                 name_labels: Name("test2.prose.dev.remibardon.com."),
    //                 rr_type: CNAME,
    //                 dns_class: IN,
    //                 ttl: 300,
    //                 rdata: Some(
    //                     CNAME(
    //                         CNAME(
    //                             Name("test.prose.dev.remibardon.com."),
    //                         ),
    //                     ),
    //                 ),
    //             },
    //             Record {
    //                 name_labels: Name("test.prose.dev.remibardon.com."),
    //                 rr_type: A,
    //                 dns_class: IN,
    //                 ttl: 600,
    //                 rdata: Some(
    //                     A(
    //                         A(
    //                             78.126.194.31,
    //                         ),
    //                     ),
    //                 ),
    //             },
    //         ],
    //         valid_until: Instant {
    //             tv_sec: 25252,
    //             tv_nsec: 847597560,
    //         },
    //     },
    // )
    let records = ipv4_lookup.as_lookup().records();
    println!("A records for {host}: {:?}", records);

    let test_contains = |expected: DnsRecord| {
        println!(
            "Contains `{:?}`: {}",
            expected.clone(),
            records.contains(&expected.into()),
        );
    };

    let expected_ip = Ipv4Addr::new(140, 82, 52, 95);
    test_contains(DnsRecord::A {
        hostname: LowerName::from_str("prose.org.").unwrap(),
        ttl: 600,
        value: expected_ip,
    }); // true
    test_contains(DnsRecord::A {
        hostname: LowerName::from_str("prose.org").unwrap(),
        ttl: 600,
        value: expected_ip,
    }); // true
    test_contains(DnsRecord::A {
        hostname: LowerName::from_str("prose.org").unwrap(),
        ttl: 300,
        value: expected_ip,
    }); // true
    test_contains(DnsRecord::A {
        hostname: LowerName::from_str("prose.org").unwrap(),
        ttl: 900,
        value: expected_ip,
    }); // true
    test_contains(DnsRecord::A {
        hostname: LowerName::from_str("prose.org").unwrap(),
        ttl: 900,
        value: Ipv4Addr::new(127, 82, 52, 95),
    }); // false
}

#[allow(unused)]
fn test_ipv6_lookup() {
    let resolver = Resolver::default().unwrap();
    // resolver.clear_cache();

    let host = "prose.org.";
    let ipv6_lookup = resolver.ipv6_lookup(host).unwrap();
    dbg!(ipv6_lookup.clone());
    // ipv6_lookup = Ipv6Lookup(
    //     Lookup {
    //         query: Query {
    //             name: Name("prose.org."),
    //             query_type: AAAA,
    //             query_class: IN,
    //         },
    //         records: [
    //             Record {
    //                 name_labels: Name("prose.org."),
    //                 rr_type: AAAA,
    //                 dns_class: IN,
    //                 ttl: 600,
    //                 rdata: Some(
    //                     AAAA(
    //                         AAAA(
    //                             2001:19f0:6801:1840:5400:3ff:feec:b62d,
    //                         ),
    //                     ),
    //                 ),
    //             },
    //         ],
    //         valid_until: Instant {
    //             tv_sec: 25198,
    //             tv_nsec: 554662525,
    //         },
    //     },
    // )
    let records = ipv6_lookup.as_lookup().records();
    println!("AAAA records for {host}: {:?}", records);
}

#[allow(unused)]
fn test_srv_lookup() {
    let resolver = Resolver::default().unwrap();
    // resolver.clear_cache();

    let host = "prose.org.";
    let srv_lookup = resolver.srv_lookup(host).unwrap();
    // called `Result::unwrap()` on an `Err` value: ResolveError { kind: NoRecordsFound { query: Query { name: Name("prose.org."), query_type: SRV, query_class: IN }, soa: Some(Record { name_labels: Name("prose.org."), rr_type: SOA, dns_class: IN, ttl: 300, rdata: Some(SOA { mname: Name("ns1.vultr.com."), rname: Name("hostmaster.prose.org."), serial: 0, refresh: 10800, retry: 3600, expire: 604800, minimum: 3600 }) }), negative_ttl: Some(300), response_code: NoError, trusted: true } }
    dbg!(srv_lookup.clone());
    // srv_lookup =
    let records = srv_lookup.as_lookup().records();
    println!("SRV records for {host}: {:?}", records);
}

#[allow(unused)]
fn check_srv_records() {
    fn check_srv_record(domain: &str) -> bool {
        println!("");
        let resolver = Resolver::default().unwrap();

        // Query for SRV record
        let Ok(response) = resolver.lookup(domain, RecordType::SRV) else {
            println!("No SRV record for {domain}");
            return false;
        };
        let records = response.records();

        // Check if there are any SRV records in the response
        let srv_record_present = records
            .iter()
            .any(|record| matches!(record.data(), Some(RData::SRV(_))));

        println!("{domain} SRV records: {records:?}");
        println!("{domain} has SRV records? {srv_record_present}");

        srv_record_present
    }

    check_srv_record("prose.org.");
    // No SRV record for prose.org.
    check_srv_record("_xmpp-client._tcp.prose.org.");
    // No SRV record for _xmpp-client._tcp.prose.org.
    check_srv_record("prosody.im.");
    // No SRV record for prosody.im.
    check_srv_record("_xmpp-client._tcp.prosody.im.");
    // _xmpp-client._tcp.prosody.im. SRV records: [Record { name_labels: Name("_xmpp-client._tcp.prosody.im."), rr_type: SRV, dns_class: IN, ttl: 21600, rdata: Some(SRV(SRV { priority: 0, weight: 0, port: 5222, target: Name("heavy-horse.co.uk.") })) }]
    // _xmpp-client._tcp.prosody.im. has SRV records? true
    check_srv_record("_xmpp-server._tcp.prosody.im.");
    // _xmpp-server._tcp.prosody.im. SRV records: [Record { name_labels: Name("_xmpp-server._tcp.prosody.im."), rr_type: SRV, dns_class: IN, ttl: 21600, rdata: Some(SRV(SRV { priority: 0, weight: 0, port: 5269, target: Name("heavy-horse.co.uk.") })) }]
    // _xmpp-server._tcp.prosody.im. has SRV records? true
    check_srv_record("valeriansaliou.name.");
    // No SRV record for valeriansaliou.name.
    check_srv_record("_xmpp-client._tcp.valeriansaliou.name.");
    // _xmpp-client._tcp.valeriansaliou.name. SRV records: [Record { name_labels: Name("_xmpp-client._tcp.valeriansaliou.name."), rr_type: SRV, dns_class: IN, ttl: 10800, rdata: Some(SRV(SRV { priority: 1, weight: 1, port: 5222, target: Name("valeriansaliou.name.") })) }]
    // _xmpp-client._tcp.valeriansaliou.name. has SRV records? true
    check_srv_record("test.prose.dev.remibardon.com.");
    // test.prose.dev.remibardon.com. SRV records: [Record { name_labels: Name("test.prose.dev.remibardon.com."), rr_type: SRV, dns_class: IN, ttl: 3600, rdata: Some(SRV(SRV { priority: 0, weight: 5, port: 5222, target: Name("xmpp.test.prose.dev.remibardon.com.") })) }]
    // test.prose.dev.remibardon.com. has SRV records? true
}

enum ConnectionType {
    C2S,
    S2S,
}

impl ConnectionType {
    fn port(&self) -> u16 {
        match self {
            Self::C2S => 5222,
            Self::S2S => 5269,
        }
    }
    fn standard_domain(&self, domain: &str) -> String {
        match self {
            Self::C2S => format!("_xmpp-client._tcp.{domain}"),
            Self::S2S => format!("_xmpp-server._tcp.{domain}"),
        }
    }
}

fn has_srv_record(domain: &str, conn_type: ConnectionType) -> bool {
    let resolver = Resolver::default().unwrap();
    let Ok(srv_lookup) = resolver
        .srv_lookup(conn_type.standard_domain(domain))
        .or_else(|_err| resolver.srv_lookup(domain))
    else {
        return false;
    };
    srv_lookup
        .as_lookup()
        .records()
        .iter()
        .any(|record| match record.data() {
            Some(RData::SRV(srv)) => srv.port() == conn_type.port(),
            _ => false,
        })
}

#[allow(unused)]
fn check_srv_c2s() {
    fn is_valid(domain: &str) -> bool {
        has_srv_record(domain, ConnectionType::C2S)
    }

    assert_eq!(is_valid("prose.org."), false);
    assert_eq!(is_valid("prosody.im."), true);
    assert_eq!(is_valid("valeriansaliou.name."), true);
    assert_eq!(is_valid("test.prose.dev.remibardon.com."), true);
}

#[allow(unused)]
fn check_srv_s2s() {
    fn is_valid(domain: &str) -> bool {
        has_srv_record(domain, ConnectionType::S2S)
    }

    assert_eq!(is_valid("prose.org."), false);
    assert_eq!(is_valid("prosody.im."), true);
    assert_eq!(is_valid("valeriansaliou.name."), true);
    assert_eq!(is_valid("test.prose.dev.remibardon.com."), false);
}

#[allow(unused)]
fn test_lookup_ip() {
    let resolver = Resolver::default().unwrap();
    // resolver.clear_cache();

    let host = "prose.org.";
    let lookup_ip = resolver.lookup_ip(host).unwrap();
    dbg!(lookup_ip.clone());
    // lookup_ip = LookupIp(
    //     Lookup {
    //         query: Query {
    //             name: Name("prose.org."),
    //             query_type: A,
    //             query_class: IN,
    //         },
    //         records: [
    //             Record {
    //                 name_labels: Name("prose.org."),
    //                 rr_type: A,
    //                 dns_class: IN,
    //                 ttl: 600,
    //                 rdata: Some(
    //                     A(
    //                         A(
    //                             140.82.52.95,
    //                         ),
    //                     ),
    //                 ),
    //             },
    //         ],
    //         valid_until: Instant {
    //             tv_sec: 24970,
    //             tv_nsec: 333133236,
    //         },
    //     },
    // )
    let records = lookup_ip.as_lookup().records();
    println!("IP lookup records for {host}: {:?}", records);
}
