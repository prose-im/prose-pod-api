// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};
use rocket::get;
use service::{controllers::member_controller::MemberController, prose_xmpp::BareJid};

use crate::{error::Error, forms::Timestamp, guards::LazyGuard, responders::Paginated};

use super::model::*;

#[get("/v1/members?<page_number>&<page_size>&<until>")]
pub async fn get_members_route<'r>(
    member_controller: LazyGuard<MemberController<'r>>,
    jid: LazyGuard<BareJid>,
    page_number: Option<u64>,
    page_size: Option<u64>,
    until: Option<Timestamp>,
) -> Result<Paginated<Member>, Error> {
    // Make sure the user is logged in.
    let _ = jid.inner?;

    let member_controller = member_controller.inner?;
    let page_number = page_number.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);
    let until: Option<DateTime<Utc>> = match until {
        Some(t) => Some(t.try_into()?),
        None => None,
    };

    let (pages_metadata, members) = member_controller
        .get_members(page_number, page_size, until)
        .await?;

    Ok(Paginated::new(
        members.into_iter().map(Into::into).collect(),
        page_number,
        page_size,
        pages_metadata,
    ))
}
