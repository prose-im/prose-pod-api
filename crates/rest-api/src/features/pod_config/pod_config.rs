// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::serde::json::Json;
use sea_orm_rocket::Connection;
use service::{
    auth::UserInfo,
    members::MemberRepository,
    pod_config::{PodConfig, PodConfigRepository},
};

use crate::{
    error::{self, Error},
    guards::{Db, LazyGuard},
};

#[rocket::get("/v1/pod/config")]
pub async fn get_pod_config_route<'r>(
    conn: Connection<'r, Db>,
    user_info: LazyGuard<UserInfo>,
) -> Result<Json<PodConfig>, Error> {
    let db = conn.into_inner();

    let jid = user_info.inner?.jid;
    // TODO: Use a request guard instead of checking in the route body if the user can invite members.
    if !MemberRepository::is_admin(db, &jid).await? {
        return Err(error::Forbidden(format!("<{jid}> is not an admin")).into());
    }

    let model = PodConfigRepository::get(db).await?;

    let res = model.map(PodConfig::from).unwrap_or_default();
    Ok(res.into())
}

pub async fn get_pod_config_route_axum() {
    todo!()
}
