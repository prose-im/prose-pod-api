// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{extract::Path, Json};
use service::{members::MemberService, xmpp::BareJid};

use crate::error::{self, Error};

use super::EnrichedMember;

pub async fn get_member_route(
    Path(jid): Path<BareJid>,
    member_service: MemberService,
) -> Result<Json<EnrichedMember>, Error> {
    let member = member_service.enrich_jid(&jid).await?;
    let Some(member) = member else {
        return Err(Error::from(error::NotFound {
            reason: format!("No member with id '{jid}'"),
        }));
    };

    let response = EnrichedMember::from(member);
    Ok(response.into())
}
