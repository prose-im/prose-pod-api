// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;
use std::ops::Deref;

use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use prose_pod_core::prose_xmpp::BareJid;
use prose_pod_core::repositories::MemberRepository;
use rocket::form::Strict;
use rocket::response::stream::{Event, EventStream};
use rocket::serde::json::Json;
use rocket::{get, put};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};

use super::models::*;
use crate::error::Error;
use crate::forms::{Timestamp, JID as JIDUriParam};
use crate::guards::{Db, LazyGuard, XmppService, JID as JIDGuard};
use crate::responders::Paginated;

#[get("/v1/members?<page_number>&<page_size>&<until>")]
pub(super) async fn get_members(
    conn: Connection<'_, Db>,
    jid: LazyGuard<JIDGuard>,
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
        MemberRepository::get_all(db, page_number, page_size, until).await?;
    Ok(Paginated::new(
        members.into_iter().map(Into::into).collect(),
        page_number,
        page_size,
        pages_metadata,
    ))
}

fn enriched_member(xmpp_service: &XmppService, jid: &BareJid) -> EnrichedMember {
    trace!("Enriching `{jid}`…");

    let vcard = match xmpp_service.get_vcard(jid) {
        Ok(Some(vcard)) => Some(vcard),
        Ok(None) => {
            debug!("`{jid}` has no vCard.");
            None
        }
        Err(err) => {
            // Log error
            warn!("Could not get `{jid}`'s vCard: {err}");
            // But dismiss it
            None
        }
    };
    let nickname = vcard
        .and_then(|vcard| vcard.nickname.first().cloned())
        .map(|p| p.value);

    let avatar = match xmpp_service.get_avatar(jid) {
        Ok(Some(avatar)) => Some(avatar.base64().to_string()),
        Ok(None) => {
            debug!("`{jid}` has no avatar.");
            None
        }
        Err(err) => {
            // Log error
            warn!("Could not get `{jid}`'s avatar: {err}");
            // But dismiss it
            None
        }
    };

    let online = xmpp_service
        .is_connected(jid)
        // Log error
        .inspect_err(|err| warn!("Could not get `{jid}`'s online status: {err}"))
        // But dismiss it
        .ok();

    EnrichedMember {
        jid: jid.to_owned(),
        nickname,
        avatar,
        online,
    }
}

#[get("/v1/enrich-members?<jids..>", format = "application/json")]
pub(super) fn enrich_members<'r>(
    xmpp_service: LazyGuard<XmppService<'r>>,
    jids: Strict<JIDs>,
) -> Result<Json<HashMap<BareJid, EnrichedMember>>, Error> {
    let xmpp_service = xmpp_service.inner?;
    let jids = jids.into_inner();

    let mut res = HashMap::with_capacity(jids.len());
    for jid in jids.iter() {
        res.insert(jid.deref().to_owned(), enriched_member(&xmpp_service, jid));
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
            let res = enriched_member(&xmpp_service, jid);
            yield logged(Event::json(&res).id(jid.to_string()).event("enriched-member"));
        }

        yield logged(Event::empty().event("end").id("end").with_comment("End of stream"));
    })
}

/// Get information about one member.
#[get("/v1/members/<jid>")]
pub(super) fn get_member(jid: JIDUriParam) -> String {
    let _jid = jid;
    todo!()
}

/// Change a member's role.
#[put("/v1/members/<_member_id>/role")]
pub(super) fn set_member_role(_member_id: &str) -> String {
    todo!()
}

/// Change a member's Multi-Factor Authentication (MFA) status.
#[put("/v1/members/<_member_id>/mfa")]
pub(super) fn set_member_mfa(_member_id: &str) -> String {
    todo!()
}

/// Log a member out from all of its devices.
#[put("/v1/members/<_member_id>/logout")]
pub(super) fn logout_member(_member_id: &str) -> String {
    todo!()
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
pub(super) fn set_member_nickname(
    member_id: JIDUriParam,
    jid: LazyGuard<JIDGuard>,
    xmpp_service: LazyGuard<XmppService>,
    req: Json<SetMemberNicknameRequest>,
) -> Result<Json<SetMemberNicknameResponse>, Error> {
    let jid = jid.inner?;
    let xmpp_service = xmpp_service.inner?;

    if jid.deref() != member_id.deref() {
        Err(Error::Unauthorized)?
    }

    xmpp_service.set_own_nickname(&req.nickname)?;

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
pub(super) fn set_member_avatar(
    member_id: JIDUriParam,
    jid: LazyGuard<JIDGuard>,
    xmpp_service: LazyGuard<XmppService>,
    req: Json<SetMemberAvatarRequest>,
) -> Result<Json<SetMemberAvatarResponse>, Error> {
    let jid = jid.inner?;
    let xmpp_service = xmpp_service.inner?;

    if jid.deref() != member_id.deref() {
        Err(Error::Unauthorized)?
    }

    let image_data = general_purpose::STANDARD
        .decode(req.image.to_owned())
        .map_err(|err| Error::BadRequest {
            reason: format!("Invalid `image` field: data should be base64-encoded. Error: {err}"),
        })?;

    xmpp_service.set_own_avatar(image_data)?;

    Ok(SetMemberAvatarResponse {
        jid: jid.to_owned(),
        image: req.image.to_owned(),
    }
    .into())
}
