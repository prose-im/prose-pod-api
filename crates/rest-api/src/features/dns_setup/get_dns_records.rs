// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use service::network_checks::{DnsRecordWithStringRepr, DnsSetupStep, PodNetworkConfig};

use crate::{error::Error, guards::LazyGuard};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDnsRecordsResponse {
    pub steps: Vec<DnsSetupStep<DnsRecordWithStringRepr>>,
}

#[rocket::get("/v1/network/dns/records", format = "json")]
pub async fn get_dns_records_route(
    pod_network_config: LazyGuard<PodNetworkConfig>,
) -> Result<Json<GetDnsRecordsResponse>, Error> {
    let pod_network_config = pod_network_config.inner?;

    let steps: Vec<_> = pod_network_config.dns_setup_steps().collect();

    let res = GetDnsRecordsResponse { steps };
    Ok(res.into())
}

pub async fn get_dns_records_route_axum() {
    todo!()
}
