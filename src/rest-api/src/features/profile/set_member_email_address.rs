// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    extract::{Path, State},
    Json,
};
use service::{
    auth::UserInfo,
    members::{MemberRepository, MemberRole},
    models::{BareJid, EmailAddress},
};

use crate::{
    error::{self, Error},
    AppState,
};

pub async fn set_member_email_address_route(
    State(AppState { ref db, .. }): State<AppState>,
    Path(jid): Path<BareJid>,
    caller: UserInfo,
    Json(email_address): Json<EmailAddress>,
) -> Result<(), Error> {
    if !(caller.jid == jid || caller.role == MemberRole::Admin) {
        Err(error::Forbidden("You cannot do that.".to_string()))?
    }

    MemberRepository::set_email_address(db, &jid, Some(email_address)).await?;

    Ok(())
}
