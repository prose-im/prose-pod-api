// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::future::Future;

use tracing::{instrument, trace, trace_span, Instrument as _};

use crate::network_checks::NetworkChecker;

/// Resolves SRV records for the host and follow them if possible, then runs the check on SRV targets.
/// Falls back to checking on the host itself if the check didn't pass before that.
#[instrument(level = "trace", skip_all, fields(host), ret)]
pub(super) async fn flattened_srv_lookup<Fut>(
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

        tracing::Span::current()
            .record("records", format!("{:?}", res.records))
            .record(
                "recursively_resolved_ips",
                format!("{:?}", res.recursively_resolved_ips),
            );

        for ip in res.recursively_resolved_ips {
            if check(&ip.to_string())
                .instrument(trace_span!("check_ip", ip = ip.to_string()))
                .await
            {
                return true;
            }
        }

        tracing::Span::current().record("srv_targets", format!("{:?}", res.srv_targets));

        for target in res.srv_targets {
            // NOTE(RemiBardon): We might want to recursively call `flattened_run` here.
            //   I just found no use case so let's call the faster `check`.
            if check(&target.to_string())
                .instrument(trace_span!("check_domain", target = target.to_string()))
                .await
            {
                return true;
            }
        }

        check(host).await
    } else {
        trace!("-> {host} has no SRV record");
        check(host).await
    }
}
