// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[cfg(debug_assertions)]
use axum::extract::State;
use axum::{http::HeaderValue, Json};
#[cfg(debug_assertions)]
use axum_extra::either::Either;
use serde::{Deserialize, Serialize};
use service::{
    invitations::{InvitationContact, InvitationService, InviteMemberError, InviteMemberForm},
    members::{MemberRepository, MemberRole},
    models::JidNode,
    notifications::Notifier,
    server_config::ServerConfig,
    AppConfig,
};

use crate::{error::prelude::*, responders::Created};
#[cfg(debug_assertions)]
use crate::{features::members::Member, AppState};

use super::model::*;

#[cfg(not(debug_assertions))]
pub type InviteMemberResponse = Result<Created<WorkspaceInvitation>, Error>;
#[cfg(not(debug_assertions))]
fn ok(invitation: WorkspaceInvitation, resource_uri: HeaderValue) -> InviteMemberResponse {
    Ok(Created {
        location: resource_uri,
        body: invitation,
    })
}
#[cfg(debug_assertions)]
pub type InviteMemberResponse =
    Result<Either<Created<WorkspaceInvitation>, Created<Member>>, Error>;
#[cfg(debug_assertions)]
fn ok(invitation: WorkspaceInvitation, resource_uri: HeaderValue) -> InviteMemberResponse {
    Ok(Either::E1(Created {
        location: resource_uri,
        body: invitation,
    }))
}

#[derive(Serialize, Deserialize)]
pub struct InviteMemberRequest {
    pub username: JidNode,
    #[serde(default)]
    pub pre_assigned_role: MemberRole,
    #[serde(flatten)]
    pub contact: InvitationContact,
}

/// Invite a new member and auto-accept the invitation if enabled.
pub async fn invite_member_route(
    #[cfg(debug_assertions)] State(AppState { db, .. }): State<AppState>,
    app_config: AppConfig,
    server_config: ServerConfig,
    notifier: Notifier,
    invitation_service: InvitationService,
    Json(req): Json<InviteMemberRequest>,
) -> InviteMemberResponse {
    let invitation = invitation_service
        .invite_member(&app_config, &server_config, &notifier, req)
        .await?;

    #[cfg(debug_assertions)]
    {
        if app_config.debug_only.automatically_accept_invitations {
            let jid = invitation.jid;
            let resource_uri = format!("/v1/members/{jid}");
            let member = MemberRepository::get(&db, &jid).await?.unwrap();
            let response: Member = member.into();
            return Ok(Either::E2(Created {
                location: HeaderValue::from_str(&resource_uri)?,
                body: response,
            }));
        }
    }

    let resource_uri = format!("/v1/invitations/{}", invitation.id);
    ok(invitation.into(), HeaderValue::from_str(&resource_uri)?)
}

// ERRORS

impl ErrorCode {
    const INVITE_ALREADY_EXISTS: Self = Self {
        value: "invitation_already_exists",
        http_status: StatusCode::CONFLICT,
        log_level: LogLevel::Info,
    };
}
impl ErrorCode {
    pub(super) const MEMBER_ALREADY_EXISTS: Self = Self {
        value: "member_already_exists",
        http_status: StatusCode::CONFLICT,
        log_level: LogLevel::Info,
    };
}

impl CustomErrorCode for InviteMemberError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::InvalidJid(_) => ErrorCode::BAD_REQUEST,
            Self::InvitationConfict => ErrorCode::INVITE_ALREADY_EXISTS,
            Self::UsernameConfict => ErrorCode::MEMBER_ALREADY_EXISTS,
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
