// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref as _;

use rocket::{put, serde::json::Json};
use serde::{Deserialize, Serialize};
use service::{prose_xmpp::BareJid, services::xmpp_service::XmppService};

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
#[put("/v1/members/<member_id>/nickname", format = "json", data = "<req>")]
pub async fn set_member_nickname_route<'r>(
    member_id: JIDUriParam,
    jid: LazyGuard<BareJid>,
    xmpp_service: LazyGuard<XmppService<'r>>,
    req: Json<SetMemberNicknameRequest>,
) -> Result<Json<SetMemberNicknameResponse>, Error> {
    let jid = jid.inner?;
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
