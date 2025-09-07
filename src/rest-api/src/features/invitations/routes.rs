// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[cfg(debug_assertions)]
use axum::extract::State;
use axum::{
    extract::{Path, Query},
    http::{HeaderValue, StatusCode},
    Json,
};
#[cfg(debug_assertions)]
use axum_extra::either::Either;
use service::{
    auth::IsAdmin,
    invitations::{
        invitation_controller::{self, WorkspaceInvitationBasicDetails},
        InvitationAcceptForm, InvitationContact, InvitationId, InvitationService, InvitationToken,
        InvitationTokenType, InviteMemberForm,
    },
    members::MemberRole,
    models::{PaginationForm, SerializableSecretString},
    notifications::NotificationService,
    workspace::WorkspaceService,
    xmpp::JidNode,
    AppConfig,
};

#[cfg(debug_assertions)]
use crate::AppState;
use crate::{
    error::Error,
    responders::{Created, Paginated},
};

use super::dtos::*;

// MARK: CREATE

#[cfg(not(debug_assertions))]
pub type InviteMemberResponse = Result<Created<WorkspaceInvitationDto>, Error>;
#[cfg(not(debug_assertions))]
fn ok(invitation: WorkspaceInvitationDto, resource_uri: HeaderValue) -> InviteMemberResponse {
    Ok(Created {
        location: resource_uri,
        body: invitation,
    })
}
#[cfg(debug_assertions)]
pub type InviteMemberResponse =
    Result<Either<Created<WorkspaceInvitationDto>, Created<service::members::Member>>, Error>;
#[cfg(debug_assertions)]
fn ok(invitation: WorkspaceInvitationDto, resource_uri: HeaderValue) -> InviteMemberResponse {
    Ok(Either::E1(Created {
        location: resource_uri,
        body: invitation,
    }))
}

#[derive(serdev::Deserialize)]
#[cfg_attr(feature = "test", derive(serdev::Serialize))]
pub struct InviteMemberRequest {
    pub username: JidNode,
    #[serde(default)]
    pub pre_assigned_role: MemberRole,
    #[serde(flatten)]
    pub contact: InvitationContact,
}

#[cfg(debug_assertions)]
#[derive(serdev::Deserialize)]
pub struct InviteMemberQuery {
    #[serde(default)]
    pub auto_accept: bool,
}

/// Invite a new member and auto-accept the invitation if enabled.
pub async fn invite_member_route(
    #[cfg(debug_assertions)] State(AppState { ref db, .. }): State<AppState>,
    ref app_config: AppConfig,
    ref notification_service: NotificationService,
    ref invitation_service: InvitationService,
    ref workspace_service: WorkspaceService,
    #[cfg(debug_assertions)] Query(InviteMemberQuery { auto_accept }): Query<InviteMemberQuery>,
    Json(req): Json<InviteMemberRequest>,
) -> InviteMemberResponse {
    let res = invitation_controller::invite_member(
        #[cfg(debug_assertions)]
        db,
        app_config,
        app_config.server_domain(),
        notification_service,
        invitation_service,
        workspace_service,
        #[cfg(debug_assertions)]
        auto_accept,
        req.into(),
    )
    .await?;

    #[cfg(debug_assertions)]
    let invitation = match res {
        service::util::either::Either::E1(invitation) => invitation,
        service::util::either::Either::E2(member) => {
            let resource_uri = format!("/v1/members/{jid}", jid = member.jid);
            return Ok(Either::E2(Created {
                location: HeaderValue::from_str(&resource_uri)?,
                body: member,
            }));
        }
    };
    #[cfg(not(debug_assertions))]
    let invitation = res;

    let resource_uri = format!("/v1/invitations/{id}", id = invitation.id);
    ok(invitation.into(), HeaderValue::from_str(&resource_uri)?)
}

pub async fn can_invite_member_route(
    is_admin: Option<IsAdmin>,
    ref app_config: AppConfig,
) -> StatusCode {
    if is_admin.is_none() {
        StatusCode::FORBIDDEN
    } else if app_config.notifiers.email.is_some() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::PRECONDITION_FAILED
    }
}

// MARK: GET ONE

pub async fn get_invitation_route(
    invitation_service: InvitationService,
    Path(invitation_id): Path<InvitationId>,
) -> Result<Json<WorkspaceInvitationDto>, Error> {
    match invitation_controller::get_invitation(invitation_id, invitation_service).await? {
        invitation => Ok(Json(invitation.into())),
    }
}

#[derive(serdev::Deserialize)]
pub struct GetInvitationTokenDetailsQuery {
    token_type: InvitationTokenType,
}

pub async fn get_invitation_by_token_route(
    ref invitation_service: InvitationService,
    Path(token): Path<InvitationToken>,
    Query(GetInvitationTokenDetailsQuery { token_type }): Query<GetInvitationTokenDetailsQuery>,
) -> Result<Json<WorkspaceInvitationBasicDetails>, Error> {
    match invitation_controller::get_invitation_by_token(token, token_type, invitation_service)
        .await?
    {
        details => Ok(Json(details)),
    }
}

// MARK: GET MANY

pub async fn get_invitations_route(
    invitation_service: InvitationService,
    Query(pagination): Query<PaginationForm>,
) -> Result<Paginated<WorkspaceInvitationDto>, Error> {
    match invitation_controller::get_invitations(invitation_service, pagination).await? {
        invitations => Ok(invitations.map(Into::into).into()),
    }
}

// MARK: ACTIONS

#[derive(serdev::Deserialize)]
#[cfg_attr(feature = "test", derive(serdev::Serialize))]
pub struct AcceptWorkspaceInvitationRequest {
    pub nickname: String,
    pub password: SerializableSecretString,
}

pub async fn invitation_accept_route(
    ref invitation_service: InvitationService,
    Path(token): Path<InvitationToken>,
    Json(req): Json<AcceptWorkspaceInvitationRequest>,
) -> Result<(), Error> {
    invitation_controller::invitation_accept(invitation_service, token, req.into()).await?;
    Ok(())
}

pub async fn invitation_reject_route(
    ref invitation_service: InvitationService,
    Path(token): Path<InvitationToken>,
) -> Result<StatusCode, Error> {
    invitation_controller::invitation_reject(invitation_service, token).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn invitation_resend_route(
    ref invitation_service: InvitationService,
    ref app_config: AppConfig,
    ref notification_service: NotificationService,
    ref workspace_service: WorkspaceService,
    Path(invitation_id): Path<InvitationId>,
) -> Result<StatusCode, Error> {
    invitation_controller::invitation_resend(
        invitation_service,
        app_config,
        notification_service,
        workspace_service,
        invitation_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn invitation_cancel_route(
    ref invitation_service: InvitationService,
    Path(invitation_id): Path<InvitationId>,
) -> Result<StatusCode, Error> {
    invitation_controller::invitation_cancel(invitation_service, invitation_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// MARK: BOILERPLATE

impl Into<InvitationAcceptForm> for AcceptWorkspaceInvitationRequest {
    fn into(self) -> InvitationAcceptForm {
        InvitationAcceptForm {
            nickname: self.nickname,
            password: self.password.into(),
        }
    }
}

impl Into<InviteMemberForm> for InviteMemberRequest {
    fn into(self) -> InviteMemberForm {
        InviteMemberForm {
            username: self.username,
            pre_assigned_role: self.pre_assigned_role,
            contact: self.contact,
        }
    }
}
