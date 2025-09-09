// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{extract::Path, response::NoContent};
use service::{
    auth::UserInfo,
    models::{Avatar, BareJid},
    xmpp::XmppService,
};

use crate::error::{self, Error};

pub async fn set_member_avatar_route<'a>(
    Path(member_id): Path<BareJid>,
    UserInfo { jid, .. }: UserInfo,
    xmpp_service: XmppService,
    avatar: Avatar<'a>,
) -> Result<NoContent, Error> {
    if jid != member_id {
        Err(error::Forbidden(
            "You can’t change someone else’s avatar.".to_string(),
        ))?
    }

    xmpp_service.set_own_avatar(avatar).await?;

    Ok(NoContent)
}
