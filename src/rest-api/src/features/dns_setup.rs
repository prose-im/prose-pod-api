// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{middleware::from_extractor_with_state, routing::get};
use service::auth::IsAdmin;

use crate::AppState;

pub use self::routes::*;

use super::NETWORK_ROUTE;

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .nest(
            NETWORK_ROUTE,
            axum::Router::new().route("/dns/records", get(get_dns_records_route)),
        )
        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone()))
        .with_state(app_state)
}

mod routes {
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
}
