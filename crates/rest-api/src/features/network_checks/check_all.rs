// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::{
    sync::mpsc::{self, error::SendError},
    task::JoinSet,
};

use crate::features::network_checks::{
    check_dns_records::dns_record_check_result,
    check_ip_connectivity::ip_connectivity_check_result,
    check_ports_reachability::port_reachability_check_result,
};

use super::{model::*, prelude::*, util::*};

#[get(
    "/v1/network/checks?<interval>",
    format = "text/event-stream",
    rank = 2
)]
pub fn check_network_configuration_route<'r>(
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
