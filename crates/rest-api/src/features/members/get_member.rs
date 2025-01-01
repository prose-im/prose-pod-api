// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{get, serde::json::Json};
use service::{auth::UserInfo, members::MemberService};

use crate::{
    error::{self, Error},
    forms::JID as JIDUriParam,
    guards::LazyGuard,
};

use super::EnrichedMember;

#[get("/v1/members/<jid>")]
pub async fn get_member_route<'r>(
    jid: JIDUriParam,
    member_service: LazyGuard<MemberService>,
    user_info: LazyGuard<UserInfo>,
) -> Result<Json<EnrichedMember>, Error> {
    // Make sure the user is logged in.
    let _ = user_info.inner?;

    let member_service = member_service.inner?;

    let member = member_service.enrich_member(&jid).await?;
    let Some(member) = member else {
        return Err(Error::from(error::NotFound {
            reason: format!("No member with id '{jid}'"),
        }));
    };

    let response = EnrichedMember::from(member);
    Ok(response.into())
}
