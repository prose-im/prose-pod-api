// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref as _;

use axum::{extract::Path, Json};
use rocket::serde::json::Json as JsonRocket;
use serde::{Deserialize, Serialize};
use service::{
    auth::UserInfo,
    xmpp::{BareJid, XmppService},
};

use crate::{
    error::{self, Error},
    forms::JID as JIDUriParam,
    guards::LazyGuard,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetMemberNicknameRequest {
    nickname: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetMemberNicknameResponse {
    jid: BareJid,
    nickname: String,
}

/// Change a member's nickname.
#[rocket::put("/v1/members/<member_id>/nickname", format = "json", data = "<req>")]
pub async fn set_member_nickname_route<'r>(
    member_id: JIDUriParam,
    user_info: LazyGuard<UserInfo>,
    xmpp_service: LazyGuard<XmppService>,
    req: JsonRocket<SetMemberNicknameRequest>,
) -> Result<JsonRocket<SetMemberNicknameResponse>, Error> {
    let jid = user_info.inner?.jid;
    let xmpp_service = xmpp_service.inner?;

    if jid.deref() != member_id.deref() {
        Err(error::Forbidden(
            "You can't change someone else's nickname.".to_string(),
        ))?
    }

    xmpp_service.set_own_nickname(&req.nickname).await?;

    Ok(SetMemberNicknameResponse {
        jid: jid.to_owned(),
        nickname: req.nickname.to_owned(),
    }
    .into())
}

pub async fn set_member_nickname_route_axum(
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
