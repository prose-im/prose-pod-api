// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref as _;

use rocket::{get, serde::json::Json};
use service::invitations::{InvitationController, InvitationToken};

use crate::{
    error::{self, Error},
    forms::Uuid,
    guards::LazyGuard,
};

use super::{forms::InvitationTokenType, model::*};

/// Get information about a workspace invitation.
#[get("/v1/invitations/<_>", rank = 2)]
pub fn get_invitation_route() -> Result<Json<WorkspaceInvitation>, Error> {
    Err(error::NotImplemented("Get invitation").into())
}

/// Get information about an invitation from an accept or reject token.
#[get("/v1/invitations/<token>?<token_type>", rank = 1)]
pub async fn get_invitation_by_token_route<'r>(
    invitation_controller: LazyGuard<InvitationController<'r>>,
    token: Uuid,
    token_type: InvitationTokenType,
) -> Result<Json<WorkspaceInvitation>, Error> {
    let invitation_controller = invitation_controller.inner?;
    let token = InvitationToken::from(*token.deref());

    let invitation = match token_type {
        InvitationTokenType::Accept => invitation_controller.get_by_accept_token(token).await,
        InvitationTokenType::Reject => invitation_controller.get_by_reject_token(token).await,
    }?;
    let Some(invitation) = invitation else {
        return Err(error::Forbidden("No invitation found for provided token".to_string()).into());
    };

    let response: WorkspaceInvitation = invitation.into();
    Ok(response.into())
}
