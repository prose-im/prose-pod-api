// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[cfg(debug_assertions)]
use axum::extract::State;
use axum::{http::HeaderValue, Json};
#[cfg(debug_assertions)]
use axum_extra::either::Either;
use rocket::{response::status, serde::json::Json as JsonRocket};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::{
    auth::UserInfo,
    invitations::{InvitationContact, InvitationService, InviteMemberError, InviteMemberForm},
    members::{MemberRepository, MemberRole},
    models::JidNode,
    notifications::Notifier,
    server_config::ServerConfig,
    AppConfig,
};

#[cfg(not(debug_assertions))]
use crate::responders::RocketCreated;
use crate::{
    error::prelude::*,
    guards::{Db, LazyGuard},
    responders::Created,
};
#[cfg(debug_assertions)]
use crate::{
    features::members::{rocket_uri_macro_get_member_route, Member},
    forms::JID as JIDUriParam,
    responders::Either as EitherRocket,
    AppState,
};

use super::{model::*, rocket_uri_macro_get_invitation_route};

#[cfg(not(debug_assertions))]
pub type InviteMemberResponseRocket = RocketCreated<WorkspaceInvitation>;
#[cfg(not(debug_assertions))]
fn ok_rocket(invitation: WorkspaceInvitation, resource_uri: String) -> InviteMemberResponseRocket {
    Ok(status::Created::new(resource_uri).body(invitation.into()))
}
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
pub type InviteMemberResponseRocket = Result<
    EitherRocket<
        status::Created<JsonRocket<WorkspaceInvitation>>,
        status::Created<JsonRocket<Member>>,
    >,
    Error,
>;
#[cfg(debug_assertions)]
fn ok_rocket(invitation: WorkspaceInvitation, resource_uri: String) -> InviteMemberResponseRocket {
    Ok(EitherRocket::left(
        status::Created::new(resource_uri).body(invitation.into()),
    ))
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
#[rocket::post("/v1/invitations", format = "json", data = "<req>")]
pub async fn invite_member_route<'r>(
    conn: Connection<'r, Db>,
    app_config: &rocket::State<AppConfig>,
    server_config: LazyGuard<ServerConfig>,
    user_info: LazyGuard<UserInfo>,
    notifier: LazyGuard<Notifier>,
    invitation_service: LazyGuard<InvitationService>,
    req: JsonRocket<InviteMemberRequest>,
) -> InviteMemberResponseRocket {
    let db = conn.into_inner();
    let server_config = server_config.inner?;
    let notifier = notifier.inner?;
    let invitation_service = invitation_service.inner?;
    let form = req.into_inner();

    {
        let jid = user_info.inner?.jid;
        // TODO: Use a request guard instead of checking in the route body if the user can invite members.
        if !MemberRepository::is_admin(db, &jid).await? {
            return Err(error::Forbidden(format!("<{jid}> is not an admin")).into());
        }
    }

    let invitation = invitation_service
        .invite_member(app_config, &server_config, &notifier, form)
        .await?;

    #[cfg(debug_assertions)]
    {
        if app_config.debug_only.automatically_accept_invitations {
            let jid = invitation.jid;
            let resource_uri = rocket::uri!(get_member_route(jid.clone().into())).to_string();
            let member = MemberRepository::get(db, &jid).await?.unwrap();
            let response: Member = member.into();
            return Ok(EitherRocket::right(
                status::Created::new(resource_uri).body(response.into()),
            ));
        }
    }

    let resource_uri = rocket::uri!(get_invitation_route(invitation.id)).to_string();
    ok_rocket(invitation.into(), resource_uri)
}

/// Invite a new member and auto-accept the invitation if enabled.
pub async fn invite_member_route_axum(
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
