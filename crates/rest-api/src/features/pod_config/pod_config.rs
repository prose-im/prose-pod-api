// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::serde::json::Json;
use sea_orm_rocket::Connection;
use service::{
    features::{
        members::MemberRepository,
        pod_config::{PodConfig, PodConfigRepository},
    },
    models::BareJid,
};

use crate::{
    error::{self, Error},
    guards::{Db, LazyGuard},
};

#[get("/v1/pod/config")]
pub async fn get_pod_config_route<'r>(
    conn: Connection<'r, Db>,
    jid: LazyGuard<BareJid>,
) -> Result<Json<PodConfig>, Error> {
    let db = conn.into_inner();

    let jid = jid.inner?;
    // TODO: Use a request guard instead of checking in the route body if the user can invite members.
    if !MemberRepository::is_admin(db, &jid).await? {
        return Err(error::Forbidden(format!("<{jid}> is not an admin")).into());
    }

    let model = PodConfigRepository::get(db).await?;

    let res = model.map(PodConfig::from).unwrap_or_default();
    Ok(res.into())
}
