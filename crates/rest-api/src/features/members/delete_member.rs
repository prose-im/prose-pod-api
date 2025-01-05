// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use service::{members::MemberService, xmpp::BareJid};

use crate::{error::Error, AppState};

pub async fn delete_member_route(
    State(AppState { db, .. }): State<AppState>,
    Path(jid): Path<BareJid>,
    member_service: MemberService,
) -> Result<StatusCode, Error> {
    member_service.delete_user(&db, &jid).await?;
    Ok(StatusCode::NO_CONTENT)
}
