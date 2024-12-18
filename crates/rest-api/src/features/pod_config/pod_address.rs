// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::{Ipv4Addr, Ipv6Addr};

use hickory_resolver::proto::rr::Name as DomainName;
use rocket::{response::status::Created, serde::json::Json, Either};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::{
    auth::UserInfo,
    members::MemberRepository,
    pod_config::{PodAddress, PodConfig, PodConfigCreateForm, PodConfigRepository},
};

use crate::{
    error::{self, Error},
    features::init::PodAddressNotInitialized,
    guards::{Db, LazyGuard},
};

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

#[put("/v1/pod/config/address", format = "json", data = "<req>")]
pub async fn set_pod_address_route<'r>(
    conn: Connection<'r, Db>,
    user_info: LazyGuard<UserInfo>,
    req: Json<SetPodAddressRequest>,
) -> Result<Either<Created<Json<PodAddress>>, Json<PodAddress>>, Error> {
    let db = conn.into_inner();
    let req = req.into_inner();

    let jid = user_info.inner?.jid;
    // TODO: Use a request guard instead of checking in the route body if the user can invite members.
    if !MemberRepository::is_admin(db, &jid).await? {
        return Err(error::Forbidden(format!("<{jid}> is not an admin")).into());
    }

    if PodConfigRepository::get(db).await?.is_some() {
        let model = PodConfigRepository::set(db, req).await?;

        let res = PodConfig::from(model).address.unwrap();
        Ok(Either::Right(res.into()))
    } else {
        let model = PodConfigRepository::create(db, req).await?;

        let resource_uri = uri!(get_pod_address_route).to_string();
        let res = PodConfig::from(model).address.unwrap();
        Ok(Either::Left(Created::new(resource_uri).body(res.into())))
    }
}

#[get("/v1/pod/config/address")]
pub async fn get_pod_address_route<'r>(
    conn: Connection<'r, Db>,
    user_info: LazyGuard<UserInfo>,
) -> Result<Json<PodAddress>, Error> {
    let db = conn.into_inner();

    let jid = user_info.inner?.jid;
    // TODO: Use a request guard instead of checking in the route body if the user can invite members.
    if !MemberRepository::is_admin(db, &jid).await? {
        return Err(error::Forbidden(format!("<{jid}> is not an admin")).into());
    }

    let Some(address) = PodConfigRepository::get(db)
        .await?
        .and_then(|model| PodConfig::from(model).address)
    else {
        return Err(PodAddressNotInitialized.into());
    };

    Ok(address.into())
}
