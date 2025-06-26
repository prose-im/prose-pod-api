use std::{str::FromStr as _, thread::sleep, time::Duration};

#[allow(unused)]
use hickory_resolver::{
    config::{
        LookupIpStrategy, NameServerConfig, NameServerConfigGroup, Protocol, ResolverConfig,
        ResolverOpts, ServerOrderingStrategy,
    },
    lookup::NsLookup,
    Name, Resolver,
};

const HOST: &'static str = "test16.prose.dev.remibardon.com.";
const PAUSE: Duration = Duration::from_secs(5);

fn main() {
    // simple();
    // clear_cache();
    // clear_cache_short_pause();
    // clear_cache_long_pause();
    // no_cache();
    via_authoritative_ns();
}

fn pause() {
    println!("Waiting 5 seconds…");
    sleep(PAUSE);
}

fn lookup(resolver: &Resolver, host: &str) {
    println!("Running lookup for `A` records on '{HOST}'…");
    match resolver.ipv4_lookup(host) {
        Ok(res) => println!("{res:#?}"),
        Err(err) => println!("Resolve error: {err}"),
    };
}

/// Never picks up changes.
#[allow(unused)]
fn simple() {
    let resolver = Resolver::from_system_conf().unwrap();

    loop {
        lookup(&resolver, HOST);

        pause();

        println!("");
    }
}

/// Never picks up changes.
#[allow(unused)]
fn clear_cache() {
    let resolver = Resolver::from_system_conf().unwrap();

    loop {
        println!("Cleaning resolver cache…");
        resolver.clear_cache();

        lookup(&resolver, HOST);

        pause();

        println!("");
    }
}

/// Never picks up changes.
#[allow(unused)]
fn clear_cache_short_pause() {
    let resolver = Resolver::from_system_conf().unwrap();

    loop {
        println!("Cleaning resolver cache…");
        resolver.clear_cache();
        sleep(Duration::from_millis(100));

        lookup(&resolver, HOST);

        pause();

        println!("");
    }
}

/// Picks up changes after some time, but can "revert" its resolution later if it uses another resolver.
#[allow(unused)]
fn clear_cache_long_pause() {
    let resolver = Resolver::from_system_conf().unwrap();

    loop {
        println!("Cleaning resolver cache…");
        resolver.clear_cache();
        sleep(Duration::from_millis(1000));

        lookup(&resolver, HOST);

        pause();

        println!("");
    }
}

#[allow(unused)]
fn no_cache() {
    // let resolver = Resolver::from_system_conf().unwrap();
    #[allow(unused)]
    let (config, mut options) = hickory_resolver::system_conf::read_system_conf().unwrap();
    options.cache_size = 0;
    // // options.ip_strategy = LookupIpStrategy::Ipv6thenIpv4;
    // // options.preserve_intermediates = false;
    // options.recursion_desired = false;
    // options.edns0 = true;
    // options.server_ordering_strategy = ServerOrderingStrategy::UserProvidedOrder;
    // options.negative_max_ttl = Some(Duration::ZERO);
    // options.positive_max_ttl = Some(Duration::ZERO);
    // options.authentic_data = true;
    let resolver = Resolver::new(config, options).unwrap();
    resolver.clear_cache();

    loop {
        // println!("Cleaning resolver cache…");
        // resolver.clear_cache();
        // sleep(Duration::from_millis(1000));

        lookup(&resolver, HOST);

        pause();

        println!("");
    }
}

/// Gives live results, meaning new records and changes appear instantly,
/// but sometimes records reappear after deletion.
/// Not a huge problem since we will stop querying when we find a given record anyway.
#[allow(unused)]
fn via_authoritative_ns() {
    // Create a resolver with default options
    let resolver = Resolver::from_system_conf().unwrap();

    // Step 1: Query the authoritative nameservers for the domain
    fn ns_lookup(resolver: &Resolver, domain: Name) -> NsLookup {
        match resolver.ns_lookup(domain.clone()) {
            Ok(res) => {
                println!("Found NS records for {domain}");
                res
            }
            Err(_) => ns_lookup(resolver, domain.base_name()),
        }
    }
    let ns_response = ns_lookup(&resolver, Name::from_str(HOST).unwrap().base_name());

    if ns_response.iter().next().is_none() {
        panic!("No authoritative nameservers found.");
    }

    println!("Authoritative nameservers:");
    for ns in ns_response.iter() {
        println!("- {:?}", ns);
    }
    let name_servers = ns_response
        .iter()
        .map(|ns| match resolver.lookup_ip(ns.0.clone()) {
            Ok(ips) => ips.iter().collect(),
            Err(_) => vec![],
        })
        .flatten()
        .collect::<Vec<_>>();

    let config = ResolverConfig::from_parts(
        None,
        vec![],
        NameServerConfigGroup::from_ips_clear(name_servers.as_slice(), 53, false),
    );
    let mut options = ResolverOpts::default();
    options.recursion_desired = false;
    options.cache_size = 0;
    // options.server_ordering_strategy = ServerOrderingStrategy::UserProvidedOrder;
    // options.authentic_data = true;
    let resolver = Resolver::new(config, options).unwrap();

    loop {
        lookup(&resolver, HOST);

        pause();

        println!("");
    }
}
