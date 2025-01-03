// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref as _;

use axum::{extract::Path, Json};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use service::{auth::UserInfo, models::BareJid, xmpp::XmppService};

use crate::error::{self, Error};

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
pub async fn set_member_avatar_route(
    Path(member_id): Path<BareJid>,
    UserInfo { jid }: UserInfo,
    xmpp_service: XmppService,
    Json(req): Json<SetMemberAvatarRequest>,
) -> Result<Json<SetMemberAvatarResponse>, Error> {
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

    Ok(Json(SetMemberAvatarResponse {
        jid: jid.to_owned(),
        image: req.image.to_owned(),
    }))
}
