// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    extract::{Path, Query},
    Json,
};
use serde::Deserialize;
use service::invitations::{InvitationService, InvitationToken};

use crate::error::{self, Error};

use super::{forms::InvitationTokenType, model::*};

/// Get information about a workspace invitation.
pub async fn get_invitation_route(
    invitation_service: InvitationService,
    Path(invitation_id): Path<i32>,
) -> Result<Json<WorkspaceInvitation>, Error> {
    let invitation = invitation_service.get(&invitation_id).await?;
    let Some(invitation) = invitation else {
        return Err(Error::from(error::NotFound {
            reason: format!("No invitation with id '{invitation_id}'"),
        }));
    };

    let response = WorkspaceInvitation::from(invitation);
    Ok(response.into())
}

#[derive(Deserialize)]
pub struct GetInvitationTokenDetailsQuery {
    token_type: InvitationTokenType,
}

/// Get information about an invitation from an accept or reject token.
pub async fn get_invitation_by_token_route(
    invitation_service: InvitationService,
    Path(token): Path<InvitationToken>,
    Query(GetInvitationTokenDetailsQuery { token_type }): Query<GetInvitationTokenDetailsQuery>,
) -> Result<Json<WorkspaceInvitationBasicDetails>, Error> {
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
