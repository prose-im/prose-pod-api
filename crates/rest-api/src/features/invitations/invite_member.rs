// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref as _;

use rocket::{post, response::status, serde::json::Json, State};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::{
    auth::UserInfo,
    invitations::{InvitationContact, InvitationController, InviteMemberError, InviteMemberForm},
    members::{MemberRepository, MemberRole},
    models::JidNode,
    notifications::Notifier,
    server_config::ServerConfig,
    AppConfig,
};

use crate::{
    error::prelude::*,
    guards::{Db, LazyGuard},
    responders::Either,
};
#[cfg(debug_assertions)]
use crate::{
    features::members::{rocket_uri_macro_get_member_route, Member},
    forms::JID as JIDUriParam,
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
    user_info: LazyGuard<UserInfo>,
    notifier: LazyGuard<Notifier>,
    invitation_controller: LazyGuard<InvitationController>,
    req: Json<InviteMemberRequest>,
    #[cfg(debug_assertions)] invitation_service: LazyGuard<UnauthenticatedInvitationService>,
) -> Result<Either<status::Created<Json<WorkspaceInvitation>>, status::Created<Json<Member>>>, Error>
{
    let db = conn.into_inner();
    let server_config = server_config.inner?;
    let notifier = notifier.inner?;
    let invitation_controller = invitation_controller.inner?;
    let form = req.into_inner();

    {
        let jid = user_info.inner?.jid;
        // TODO: Use a request guard instead of checking in the route body if the user can invite members.
        if !MemberRepository::is_admin(db, &jid).await? {
            return Err(error::Forbidden(format!("<{jid}> is not an admin")).into());
        }
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

    if cfg!(debug_assertions) && app_config.debug_only.automatically_accept_invitations {
        let jid = invitation.jid;
        let resource_uri = uri!(get_member_route(jid.clone().into())).to_string();
        let member = MemberRepository::get(db, &jid).await?.unwrap();
        let response: Member = member.into();
        Ok(Either::right(
            status::Created::new(resource_uri).body(response.into()),
        ))
    } else {
        let resource_uri = uri!(get_invitation_route(invitation.id)).to_string();
        let response: WorkspaceInvitation = invitation.into();
        Ok(Either::left(
            status::Created::new(resource_uri).body(response.into()),
        ))
    }
}

// ERRORS

impl ErrorCode {
    const INVITE_ALREADY_EXISTS: Self = Self {
        value: "invitation_already_exists",
        http_status: Status::Conflict,
        log_level: LogLevel::Info,
    };
}

impl CustomErrorCode for InviteMemberError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::InvalidJid(_) => ErrorCode::BAD_REQUEST,
            Self::Confict => ErrorCode::INVITE_ALREADY_EXISTS,
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
