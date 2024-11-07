// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::future::Future;

use tracing::trace;

use crate::network_checks::NetworkChecker;

/// Resolves SRV records for the host and follow them if possible, then runs the check on SRV targets.
/// Falls back to checking on the host itself if the check didn't pass before that.
pub(super) async fn flattened_run<Fut>(
    host: &str,
    check: impl Fn(&str) -> Fut,
    network_checker: &NetworkChecker,
) -> bool
where
    Fut: Future<Output = bool>,
{
    trace!("Running SRV lookup for {host}…");
    if let Ok(res) = network_checker.srv_lookup(host).await {
        trace!("-> {host} has SRV records");

        trace!("-> records = {:#?}", res.records);

        trace!(
            "-> recursively_resolved_ips = {:#?}",
            res.recursively_resolved_ips,
        );

        for ip in res.recursively_resolved_ips {
            trace!("---> Checking {ip}…");
            if check(&ip.to_string()).await {
                return true;
            }
        }

        trace!("-> srv_targets = {:#?}", res.srv_targets);

        for target in res.srv_targets {
            trace!("---> Checking {target}…");
            // NOTE: We might want to recursively call `flattened_run` here.
            //   I just found no use case so I called the faster `check`.
            if check(&target.to_string()).await {
                return true;
            }
        }

        check(host).await
    } else {
        trace!("-> {host} has no SRV record");
        check(host).await
    }
}
