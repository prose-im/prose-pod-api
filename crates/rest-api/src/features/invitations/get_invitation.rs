// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref as _;

use rocket::serde::json::Json;
use service::invitations::{InvitationService, InvitationToken};

use crate::{
    error::{self, Error},
    forms::Uuid,
    guards::LazyGuard,
};

use super::{forms::InvitationTokenType, model::*};

/// Get information about a workspace invitation.
#[rocket::get("/v1/invitations/<invitation_id>")]
pub async fn get_invitation_route<'r>(
    invitation_service: LazyGuard<InvitationService>,
    invitation_id: i32,
) -> Result<Json<WorkspaceInvitation>, Error> {
    let invitation_service = invitation_service.inner?;

    let invitation = invitation_service.get(&invitation_id).await?;
    let Some(invitation) = invitation else {
        return Err(Error::from(error::NotFound {
            reason: format!("No invitation with id '{invitation_id}'"),
        }));
    };

    let response = WorkspaceInvitation::from(invitation);
    Ok(response.into())
}

pub async fn get_invitation_route_axum() {
    todo!()
}

/// Get information about an invitation from an accept or reject token.
#[rocket::get("/v1/invitation-tokens/<token>/details?<token_type>")]
pub async fn get_invitation_token_details_route<'r>(
    invitation_service: LazyGuard<InvitationService>,
    token: Uuid,
    token_type: InvitationTokenType,
) -> Result<Json<WorkspaceInvitationBasicDetails>, Error> {
    let invitation_service = invitation_service.inner?;
    let token = InvitationToken::from(*token.deref());

    let invitation = match token_type {
        InvitationTokenType::Accept => invitation_service.get_by_accept_token(token).await,
        InvitationTokenType::Reject => invitation_service.get_by_reject_token(token).await,
    }?;
    let Some(invitation) = invitation else {
        return Err(error::Forbidden("No invitation found for provided token".to_string()).into());
    };

    let response = WorkspaceInvitationBasicDetails::from(invitation);
    Ok(response.into())
}

pub async fn get_invitation_by_token_route_axum() {
    todo!()
}
