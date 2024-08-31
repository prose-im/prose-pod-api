// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::serde::json::Json;
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::{
    model::PodConfig,
    repositories::{PodConfigCreateForm, PodConfigRepository},
};

use crate::{error::Error, guards::Db};

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

#[put("/v1/pod/config/address", format = "json", data = "<req>")]
pub async fn set_pod_address<'r>(
    conn: Connection<'r, Db>,
    req: Json<SetPodAddressRequest>,
) -> Result<Json<PodConfig>, Error> {
    let db = conn.into_inner();
    let req = req.into_inner();

    let model = PodConfigRepository::set(db, req).await?;

    let res = PodConfig::from(model);
    Ok(res.into())
}
