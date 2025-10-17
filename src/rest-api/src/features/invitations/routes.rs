// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderValue, StatusCode},
    Json,
};
#[cfg(debug_assertions)]
use axum_extra::either::Either;
use secrecy::ExposeSecret;
use service::{
    auth::{AuthToken, IsAdmin},
    invitations::{
        errors::{InvitationNotFound, InvitationNotFoundForToken},
        invitation_service::AcceptAccountInvitationCommand,
        *,
    },
    members::{MemberRole, Nickname},
    models::{EmailAddress, PaginationForm},
    xmpp::JidNode,
    AppConfig,
};
use validator::Validate;

use crate::{
    error::Error,
    features::auth::models::Password,
    responders::{Created, Paginated},
    AppState,
};

use super::dtos::*;

// MARK: Create

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

#[derive(Debug)]
#[derive(serdev::Deserialize)]
// NOTE: Cannot use `serde(deny_unknown_fields)` because of a `serde(flatten)`
//   + `serde(tag)` bug. See <https://github.com/serde-rs/serde/issues/1358>.
#[cfg_attr(feature = "test", derive(serdev::Serialize))]
pub struct InviteMemberRequest {
    pub username: JidNode,

    #[serde(default)]
    pub pre_assigned_role: MemberRole,

    #[serde(flatten)]
    pub contact: InvitationContact,
}

#[cfg(debug_assertions)]
#[derive(Debug)]
#[derive(serdev::Deserialize)]
pub struct InviteMemberQuery {
    #[serde(default)]
    pub auto_accept: bool,
}

/// Invite a new member and auto-accept the invitation if enabled.
pub async fn invite_member_route(
    #[cfg(debug_assertions)] State(ref app_config): State<Arc<AppConfig>>,
    ref invitation_service: InvitationService,
    ref auth: AuthToken,
    #[cfg(debug_assertions)] Query(InviteMemberQuery { auto_accept }): Query<InviteMemberQuery>,
    Json(req): Json<InviteMemberRequest>,
) -> InviteMemberResponse {
    let res = invitation_controller::invite_member(
        invitation_service,
        #[cfg(debug_assertions)]
        app_config,
        auth,
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

    let resource_uri = format!("/v1/invitations/{id}", id = invitation.id.expose_secret());
    ok(invitation.into(), HeaderValue::from_str(&resource_uri)?)
}

pub async fn can_invite_member_route(
    is_admin: Option<IsAdmin>,
    State(ref app_config): State<Arc<AppConfig>>,
) -> StatusCode {
    if is_admin.is_none() {
        StatusCode::FORBIDDEN
    } else if app_config.notifiers.email.is_some() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::PRECONDITION_FAILED
    }
}

// MARK: Get one

pub async fn get_invitation_route(
    State(AppState {
        ref invitation_repository,
        ..
    }): State<AppState>,
    ref auth: AuthToken,
    Path(invitation_id): Path<InvitationId>,
) -> Result<Json<WorkspaceInvitationDto>, Error> {
    match invitation_controller::get_invitation(invitation_repository, &invitation_id, auth).await?
    {
        Some(invitation) => Ok(Json(invitation.into())),
        None => Err(Error::from(InvitationNotFound(invitation_id))),
    }
}

pub async fn get_invitation_by_token_route(
    State(AppState {
        ref invitation_repository,
        ..
    }): State<AppState>,
    Path(ref token): Path<InvitationToken>,
) -> Result<Json<WorkspaceInvitationBasicDetails>, Error> {
    match invitation_controller::get_invitation_by_token(invitation_repository, token).await? {
        Some(details) => Ok(Json(details)),
        None => Err(Error::from(InvitationNotFoundForToken)),
    }
}

// MARK: Get many

pub async fn get_invitations_route(
    State(AppState {
        ref invitation_repository,
        ..
    }): State<AppState>,
    ref auth: AuthToken,
    Query(pagination): Query<PaginationForm>,
) -> Result<Paginated<WorkspaceInvitationDto>, Error> {
    match invitation_controller::get_invitations(invitation_repository, pagination, auth).await? {
        invitations => Ok(invitations.map(Into::into).into()),
    }
}

// MARK: Actions

#[derive(Debug)]
#[derive(Validate, serdev::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(validate = "Validate::validate")]
#[cfg_attr(feature = "test", derive(serdev::Serialize))]
pub struct AcceptWorkspaceInvitationRequest {
    #[validate(nested)]
    pub nickname: Nickname,

    #[validate(skip)] // NOTE: Will be checked later.
    pub password: Password,

    #[serde(default)]
    #[validate(skip)]
    pub email: Option<EmailAddress>,
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
    ref auth: AuthToken,
    Path(ref invitation_id): Path<InvitationId>,
) -> Result<StatusCode, Error> {
    invitation_controller::invitation_resend(invitation_service, invitation_id, auth).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn invitation_cancel_route(
    ref invitation_service: InvitationService,
    ref auth: AuthToken,
    Path(invitation_id): Path<InvitationId>,
) -> Result<StatusCode, Error> {
    invitation_controller::invitation_cancel(invitation_service, invitation_id, auth).await?;
    Ok(StatusCode::NO_CONTENT)
}

// MARK: - Boilerplate

impl From<AcceptWorkspaceInvitationRequest> for AcceptAccountInvitationCommand {
    fn from(req: AcceptWorkspaceInvitationRequest) -> Self {
        Self {
            nickname: req.nickname,
            password: req.password.into(),
            email: req.email,
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
