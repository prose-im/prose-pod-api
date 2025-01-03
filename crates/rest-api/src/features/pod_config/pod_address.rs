// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::{Ipv4Addr, Ipv6Addr};

use axum::{extract::State, http::HeaderValue, Json};
use axum_extra::either::Either;
use hickory_resolver::proto::rr::Name as DomainName;
use serde::{Deserialize, Serialize};
use service::pod_config::{PodAddress, PodConfig, PodConfigCreateForm, PodConfigRepository};

use crate::{
    error::Error, features::init::PodAddressNotInitialized, responders::Created, AppState,
};

use super::POD_ADDRESS_ROUTE;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SetPodAddressRequest {
    pub ipv4: Option<Ipv4Addr>,
    pub ipv6: Option<Ipv6Addr>,
    pub hostname: Option<DomainName>,
}

impl Into<PodConfigCreateForm> for SetPodAddressRequest {
    fn into(self) -> PodConfigCreateForm {
        PodConfigCreateForm {
            ipv4: self.ipv4,
            ipv6: self.ipv6,
            hostname: self.hostname,
        }
    }
}

pub async fn set_pod_address_route(
    State(AppState { db, .. }): State<AppState>,
    Json(req): Json<SetPodAddressRequest>,
) -> Result<Either<Created<PodAddress>, Json<PodAddress>>, Error> {
    if PodConfigRepository::get(&db).await?.is_some() {
        let model = PodConfigRepository::set(&db, req).await?;

        let res = PodConfig::from(model).address.unwrap();
        Ok(Either::E2(Json(res)))
    } else {
        let model = PodConfigRepository::create(&db, req).await?;

        let resource_uri = HeaderValue::from_static(POD_ADDRESS_ROUTE);
        let res = PodConfig::from(model).address.unwrap();
        Ok(Either::E1(Created {
            location: resource_uri,
            body: res,
        }))
    }
}

pub async fn get_pod_address_route(
    State(AppState { db, .. }): State<AppState>,
) -> Result<Json<PodAddress>, Error> {
    let Some(address) = PodConfigRepository::get(&db)
        .await?
        .and_then(|model| PodConfig::from(model).address)
    else {
        return Err(PodAddressNotInitialized.into());
    };

    Ok(address.into())
}
