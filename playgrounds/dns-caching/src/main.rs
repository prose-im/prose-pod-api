use std::{thread::sleep, time::Duration};

#[allow(unused)]
use hickory_resolver::{
    config::{LookupIpStrategy, ResolverConfig, ResolverOpts, ServerOrderingStrategy},
    Resolver,
};

const HOST: &'static str = "test6.prose.dev.remibardon.com.";
const PAUSE: Duration = Duration::from_secs(5);

fn main() {
    // simple();
    // clear_cache();
    // clear_cache_short_pause();
    clear_cache_long_pause();
    // stash();
}

fn pause() {
    println!("Waiting 5 seconds…");
    sleep(PAUSE);
}

fn lookup(resolver: &Resolver) {
    println!("Running lookup for `A` records on '{HOST}'…");
    match resolver.ipv4_lookup(HOST) {
        Ok(res) => println!("{res:#?}"),
        Err(err) => println!("Resolve error: {err}"),
    };
}

#[allow(unused)]
fn simple() {
    let resolver = Resolver::from_system_conf().unwrap();

    loop {
        lookup(&resolver);

        pause();

        println!("");
    }
}

#[allow(unused)]
fn clear_cache() {
    let resolver = Resolver::from_system_conf().unwrap();

    loop {
        println!("Cleaning resolver cache…");
        resolver.clear_cache();

        lookup(&resolver);

        pause();

        println!("");
    }
}

#[allow(unused)]
fn clear_cache_short_pause() {
    let resolver = Resolver::from_system_conf().unwrap();

    loop {
        println!("Cleaning resolver cache…");
        resolver.clear_cache();
        sleep(Duration::from_millis(100));

        lookup(&resolver);

        pause();

        println!("");
    }
}

#[allow(unused)]
fn clear_cache_long_pause() {
    let resolver = Resolver::from_system_conf().unwrap();

    loop {
        println!("Cleaning resolver cache…");
        resolver.clear_cache();
        sleep(Duration::from_millis(1000));

        lookup(&resolver);

        pause();

        println!("");
    }
}

#[allow(unused)]
fn stash() {
    let resolver = Resolver::from_system_conf().unwrap();
    // #[allow(unused)]
    // let (config, mut options) = hickory_resolver::system_conf::read_system_conf().unwrap();
    // // options.cache_size = 0;
    // // options.ip_strategy = LookupIpStrategy::Ipv6thenIpv4;
    // options.preserve_intermediates = false;
    // // options.recursion_desired;
    // // options.edns0 = true;
    // let resolver = Resolver::new(config, options).unwrap();

    loop {
        println!("Cleaning resolver cache…");
        resolver.clear_cache();
        sleep(Duration::from_millis(1000));

        lookup(&resolver);

        pause();

        println!("");
    }
}
