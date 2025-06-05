// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use service::{
    auth::UserInfo,
    members::{MemberRepository, MemberRole},
    models::{BareJid, EmailAddress},
};

use crate::{
    error::{self, Error},
    AppState,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetMemberEmailAddressRequest {
    image: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetMemberEmailAddressResponse {
    jid: BareJid,
    // Base64 encoded image
    image: String,
}

/// Change a member's avatar.
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
