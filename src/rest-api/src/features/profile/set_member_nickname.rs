// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{extract::Path, Json};
use serdev::Serialize;
use service::{
    auth::UserInfo,
    xmpp::{BareJid, XmppService},
};

use crate::error::{self, Error};

#[derive(Clone, Debug)]
#[derive(serdev::Deserialize)]
#[cfg_attr(feature = "test", derive(serdev::Serialize))]
pub struct SetMemberNicknameRequest {
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
