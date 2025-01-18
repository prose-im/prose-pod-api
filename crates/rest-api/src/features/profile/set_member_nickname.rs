// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref as _;

use axum::{extract::Path, Json};
use serde::{Deserialize, Serialize};
use service::{
    auth::UserInfo,
    xmpp::{BareJid, XmppService},
};

use crate::error::{self, Error};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetMemberNicknameRequest {
    pub nickname: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetMemberNicknameResponse {
    pub jid: BareJid,
    pub nickname: String,
}

/// Change a member's nickname.
pub async fn set_member_nickname_route(
    Path(member_id): Path<BareJid>,
    UserInfo { jid }: UserInfo,
    xmpp_service: XmppService,
    Json(req): Json<SetMemberNicknameRequest>,
) -> Result<Json<SetMemberNicknameResponse>, Error> {
    if jid.deref() != member_id.deref() {
        Err(error::Forbidden(
            "You can't change someone else's nickname.".to_string(),
        ))?
    }

    xmpp_service.set_own_nickname(&req.nickname).await?;

    Ok(Json(SetMemberNicknameResponse {
        jid: jid.to_owned(),
        nickname: req.nickname.to_owned(),
    }))
}
