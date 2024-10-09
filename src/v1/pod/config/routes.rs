// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{response::status::Created, serde::json::Json, Either};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::{
    model::{PodAddress, PodConfig},
    repositories::{PodConfigCreateForm, PodConfigRepository},
};

use crate::{
    error::{self, Error},
    guards::Db,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SetPodAddressRequest {
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
    pub hostname: Option<String>,
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

#[get("/v1/pod/config")]
pub async fn get_pod_config<'r>(conn: Connection<'r, Db>) -> Result<Json<PodConfig>, Error> {
    let db = conn.into_inner();

    let model = PodConfigRepository::get(db).await?;

    let res = model.map(PodConfig::from).unwrap_or_default();
    Ok(res.into())
}

#[put("/v1/pod/config/address", format = "json", data = "<req>")]
pub async fn set_pod_address<'r>(
    conn: Connection<'r, Db>,
    req: Json<SetPodAddressRequest>,
) -> Result<Either<Created<Json<PodAddress>>, Json<PodAddress>>, Error> {
    let db = conn.into_inner();
    let req = req.into_inner();

    if PodConfigRepository::get(db).await?.is_some() {
        let model = PodConfigRepository::set(db, req).await?;

        let res = PodConfig::from(model).address.unwrap();
        Ok(Either::Right(res.into()))
    } else {
        let model = PodConfigRepository::create(db, req).await?;

        let resource_uri = uri!(get_pod_address).to_string();
        let res = PodConfig::from(model).address.unwrap();
        Ok(Either::Left(Created::new(resource_uri).body(res.into())))
    }
}

#[get("/v1/pod/config/address")]
pub async fn get_pod_address<'r>(conn: Connection<'r, Db>) -> Result<Json<PodAddress>, Error> {
    let db = conn.into_inner();

    let Some(address) = PodConfigRepository::get(db)
        .await?
        .and_then(|model| PodConfig::from(model).address)
    else {
        return Err(error::PodAddressNotInitialized.into());
    };

    Ok(address.into())
}
