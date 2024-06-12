// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use chrono::{DateTime, Utc};
use rocket::form::Strict;
use rocket::response::stream::{Event, EventStream};
use rocket::{get, put};
use sea_orm_rocket::Connection;
use service::Query;

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
    let (pages_metadata, members) = Query::get_members(db, page_number, page_size, until).await?;
    Ok(Paginated::new(
        members.into_iter().map(Into::into).collect(),
        page_number,
        page_size,
        pages_metadata,
    ))
}

#[get("/v1/enrich-members?<jids..>")]
pub(super) fn enrich_members(
    xmpp_service: LazyGuard<XmppService>,
    jids: Strict<JIDs>,
) -> Result<EventStream![], Error> {
    let xmpp_service = xmpp_service.inner?;
    let jids = jids.into_inner();

    Ok(EventStream! {
        for jid in jids.iter() {
            trace!("Enriching `{jid}`…");

            let vcard = match xmpp_service.get_vcard(jid) {
                Ok(Some(vcard)) => Some(vcard),
                Ok(None) => {
                    debug!("`{jid}` has no vCard.");
                    None
                },
                Err(err) => {
                    // Log error
                    error!("Could not get `{jid}`'s vCard: {err}");
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
                },
                Err(err) => {
                    // Log error
                    error!("Could not get `{jid}`'s avatar: {err}");
                    // But dismiss it
                    None
                },
            };

            let res = EnrichedMember {
                jid: jid.deref().to_owned(),
                nickname,
                avatar,
            };
            dbg!(&res);
            yield Event::json(&res).id(jid.to_string());
        }
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
