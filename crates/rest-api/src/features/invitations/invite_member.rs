// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref as _;

use rocket::{post, response::status, serde::json::Json, State};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::{
    invitations::{InvitationContact, InvitationController, InviteMemberError, InviteMemberForm},
    members::{MemberRepository, MemberRole},
    models::{BareJid, JidNode},
    notifications::Notifier,
    server_config::ServerConfig,
    AppConfig,
};

use crate::{
    error::prelude::*,
    guards::{Db, LazyGuard},
    responders::Created,
};

use super::{guards::*, model::*, rocket_uri_macro_get_invitation_route};

#[derive(Serialize, Deserialize)]
pub struct InviteMemberRequest {
    pub username: JidNode,
    #[serde(default)]
    pub pre_assigned_role: MemberRole,
    #[serde(flatten)]
    pub contact: InvitationContact,
}

/// Invite a new member.
#[post("/v1/invitations", format = "json", data = "<req>")]
pub async fn invite_member_route<'r>(
    conn: Connection<'r, Db>,
    app_config: &State<AppConfig>,
    server_config: LazyGuard<ServerConfig>,
    jid: LazyGuard<BareJid>,
    notifier: LazyGuard<Notifier<'r>>,
    invitation_controller: LazyGuard<InvitationController<'r>>,
    req: Json<InviteMemberRequest>,
    #[cfg(debug_assertions)] invitation_service: LazyGuard<UnauthenticatedInvitationService<'r>>,
) -> Created<WorkspaceInvitation> {
    let db = conn.into_inner();
    let server_config = server_config.inner?;
    let notifier = notifier.inner?;
    let invitation_controller = invitation_controller.inner?;
    let form = req.into_inner();

    let jid = jid.inner?;
    // TODO: Use a request guard instead of checking in the route body if the user can invite members.
    if !MemberRepository::is_admin(db, &jid).await? {
        return Err(error::Forbidden(format!("<{jid}> is not an admin")).into());
    }

    let invitation = invitation_controller
        .invite_member(
            app_config,
            &server_config,
            &notifier,
            form,
            #[cfg(debug_assertions)]
            invitation_service.inner?.deref(),
        )
        .await?;

    let resource_uri = uri!(get_invitation_route(invitation.id)).to_string();
    let response: WorkspaceInvitation = invitation.into();
    Ok(status::Created::new(resource_uri).body(response.into()))
}

// ERRORS

impl CustomErrorCode for InviteMemberError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::InvalidJid(_) => ErrorCode::BAD_REQUEST,
            Self::CouldNotUpdateInvitationStatus { .. } => ErrorCode::INTERNAL_SERVER_ERROR,
            #[cfg(debug_assertions)]
            Self::CouldNotAutoAcceptInvitation(err) => err.code(),
            Self::DbErr(err) => err.code(),
        }
    }
}
impl_into_error!(InviteMemberError);

// BOILERPLATE

impl Into<InviteMemberForm> for InviteMemberRequest {
    fn into(self) -> InviteMemberForm {
        InviteMemberForm {
            username: self.username,
            pre_assigned_role: self.pre_assigned_role,
            contact: self.contact,
        }
    }
}
