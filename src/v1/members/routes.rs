// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;
use std::ops::Deref;

use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use rocket::form::Strict;
use rocket::response::status::NoContent;
use rocket::response::stream::{Event, EventStream};
use rocket::serde::json::Json;
use rocket::{get, put};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::controllers::member_controller::MemberController;
use service::prose_xmpp::BareJid;
use service::services::xmpp_service::XmppService;

use super::models::*;
use crate::error::{self, Error};
use crate::forms::{Timestamp, JID as JIDUriParam};
use crate::guards::{Db, LazyGuard};
use crate::responders::Paginated;

#[get("/v1/members?<page_number>&<page_size>&<until>")]
pub(super) async fn get_members(
    conn: Connection<'_, Db>,
    jid: LazyGuard<BareJid>,
    page_number: Option<u64>,
    page_size: Option<u64>,
    until: Option<Timestamp>,
) -> Result<Paginated<Member>, Error> {
    // Make sure the user is logged in.
    let _ = jid.inner?;

    let db = conn.into_inner();
    let page_number = page_number.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);
    let until: Option<DateTime<Utc>> = match until {
        Some(t) => Some(t.try_into()?),
        None => None,
    };

    let (pages_metadata, members) =
        MemberController::get_members(db, page_number, page_size, until).await?;

    Ok(Paginated::new(
        members.into_iter().map(Into::into).collect(),
        page_number,
        page_size,
        pages_metadata,
    ))
}

#[get("/v1/enrich-members?<jids..>", format = "application/json")]
pub(super) async fn enrich_members<'r>(
    xmpp_service: LazyGuard<XmppService<'r>>,
    jids: Strict<JIDs>,
) -> Result<Json<HashMap<BareJid, EnrichedMember>>, Error> {
    let xmpp_service = xmpp_service.inner?;
    let jids = jids.into_inner();

    let mut res = HashMap::with_capacity(jids.len());
    for jid in jids.iter() {
        let enriched_member = MemberController::enrich_member(&xmpp_service, jid).await;
        res.insert(jid.deref().to_owned(), enriched_member.into());
    }
    Ok(res.into())
}

#[get("/v1/enrich-members?<jids..>", format = "text/event-stream", rank = 2)]
pub(super) fn enrich_members_stream<'r>(
    xmpp_service: LazyGuard<XmppService<'r>>,
    jids: Strict<JIDs>,
) -> Result<EventStream![Event + 'r], Error> {
    let xmpp_service = xmpp_service.inner?;
    let jids = jids.into_inner();

    Ok(EventStream! {
        fn logged(event: Event) -> Event {
            trace!("Sending {event:?}…");
            event
        }

        for jid in jids.iter() {
            let res: EnrichedMember = MemberController::enrich_member(&xmpp_service, jid).await.into();
            yield logged(Event::json(&res).id(jid.to_string()).event("enriched-member"));
        }

        yield logged(Event::empty().event("end").id("end").with_comment("End of stream"));
    })
}

#[get("/v1/members/<_jid>")]
pub(super) fn get_member(_jid: JIDUriParam) -> Result<NoContent, Error> {
    Err(error::NotImplemented("Get member").into())
}

#[put("/v1/members/<_>/role")]
pub(super) fn set_member_role() -> Result<NoContent, Error> {
    Err(error::NotImplemented("Set member role").into())
}

/// Change a member's Multi-Factor Authentication (MFA) status.
#[put("/v1/members/<_>/mfa")]
pub(super) fn set_member_mfa() -> Result<NoContent, Error> {
    Err(error::NotImplemented("Set member MFA status").into())
}

/// Log a member out from all of its devices.
#[put("/v1/members/<_>/logout")]
pub(super) fn logout_member() -> Result<NoContent, Error> {
    Err(error::NotImplemented("Log member out").into())
}

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
pub(super) async fn set_member_nickname<'r>(
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
#[put("/v1/members/<member_id>/avatar", format = "json", data = "<req>")]
pub(super) async fn set_member_avatar<'r>(
    member_id: JIDUriParam,
    jid: LazyGuard<BareJid>,
    xmpp_service: LazyGuard<XmppService<'r>>,
    req: Json<SetMemberAvatarRequest>,
) -> Result<Json<SetMemberAvatarResponse>, Error> {
    let jid = jid.inner?;
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
