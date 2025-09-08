// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{extract::Path, Json};
use serdev::Serialize;
use service::{
    auth::UserInfo,
    members::NICKNAME_MAX_LENGTH,
    xmpp::{BareJid, XmppService},
};
use validator::Validate;

use crate::error::{self, Error};

#[derive(Clone, Debug)]
#[derive(Validate, serdev::Deserialize)]
#[serde(validate = "Validate::validate")]
#[cfg_attr(feature = "test", derive(serdev::Serialize))]
pub struct SetMemberNicknameRequest {
    #[validate(length(min = 1, max = NICKNAME_MAX_LENGTH), non_control_character)]
    pub nickname: String,
}

#[derive(Clone, Debug)]
#[derive(Serialize)]
pub struct SetMemberNicknameResponse {
    pub jid: BareJid,
    pub nickname: String,
}

/// Change a member's nickname.
pub async fn set_member_nickname_route(
    Path(member_id): Path<BareJid>,
    UserInfo { jid, .. }: UserInfo,
    xmpp_service: XmppService,
    Json(req): Json<SetMemberNicknameRequest>,
) -> Result<Json<SetMemberNicknameResponse>, Error> {
    if jid != member_id {
        Err(error::Forbidden(
            "You can’t change someone else’s nickname.".to_string(),
        ))?
    }

    xmpp_service.set_own_nickname(&req.nickname).await?;

    Ok(Json(SetMemberNicknameResponse {
        jid,
        nickname: req.nickname,
    }))
}
