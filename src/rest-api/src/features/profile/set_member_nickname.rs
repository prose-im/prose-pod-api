// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{extract::Path, response::NoContent, Json};
use service::{
    auth::UserInfo,
    members::Nickname,
    xmpp::{BareJid, XmppService},
};

use crate::error::{self, Error};

pub async fn set_member_nickname_route(
    Path(member_id): Path<BareJid>,
    UserInfo { jid, .. }: UserInfo,
    xmpp_service: XmppService,
    Json(req): Json<Nickname>,
) -> Result<NoContent, Error> {
    if jid != member_id {
        Err(error::Forbidden(
            "You can’t change someone else’s nickname.".to_string(),
        ))?
    }

    xmpp_service.set_own_nickname(&req).await?;

    Ok(NoContent)
}
