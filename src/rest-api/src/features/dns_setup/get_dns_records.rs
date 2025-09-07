// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::Json;
use serdev::Serialize;
use service::network_checks::{DnsRecordWithStringRepr, DnsSetupStep, PodNetworkConfig};

use crate::error::Error;

#[derive(Debug)]
#[derive(Serialize)]
#[cfg_attr(feature = "test", derive(serdev::Deserialize))]
pub struct GetDnsRecordsResponse {
    pub steps: Vec<DnsSetupStep<DnsRecordWithStringRepr>>,
}

pub async fn get_dns_records_route(
    pod_network_config: PodNetworkConfig,
) -> Result<Json<GetDnsRecordsResponse>, Error> {
    let steps: Vec<_> = pod_network_config.dns_setup_steps().collect();

    let res = GetDnsRecordsResponse { steps };
    Ok(res.into())
}
