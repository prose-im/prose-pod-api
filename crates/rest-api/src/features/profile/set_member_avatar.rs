// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use base64::{engine::general_purpose, Engine as _};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use service::{auth::UserInfo, models::BareJid, xmpp::XmppService};

use crate::{
    error::{self, Error},
    forms::JID as JIDUriParam,
    guards::LazyGuard,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetMemberAvatarRequest {
    // Base64 encoded image
    image: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetMemberAvatarResponse {
    jid: BareJid,
    // Base64 encoded image
    image: String,
}

/// Change a member's avatar.
#[rocket::put("/v1/members/<member_id>/avatar", format = "json", data = "<req>")]
pub async fn set_member_avatar_route<'r>(
    member_id: JIDUriParam,
    user_info: LazyGuard<UserInfo>,
    xmpp_service: LazyGuard<XmppService>,
    req: Json<SetMemberAvatarRequest>,
) -> Result<Json<SetMemberAvatarResponse>, Error> {
    let jid = user_info.inner?.jid;
    let xmpp_service = xmpp_service.inner?;

    if jid.deref() != member_id.deref() {
        Err(error::Forbidden(
            "You can't change someone else's avatar.".to_string(),
        ))?
    }

    let image_data = general_purpose::STANDARD
        .decode(req.image.to_owned())
        .map_err(|err| error::BadRequest {
            reason: format!("Invalid `image` field: data should be base64-encoded. Error: {err}"),
        })?;

    xmpp_service.set_own_avatar(image_data).await?;

    Ok(SetMemberAvatarResponse {
        jid: jid.to_owned(),
        image: req.image.to_owned(),
    }
    .into())
}

pub async fn set_member_avatar_route_axum() {
    todo!()
}
