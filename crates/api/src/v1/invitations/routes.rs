// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use chrono::{DateTime, Utc};
use rocket::{
    response::status::{self, NoContent},
    serde::json::Json,
    {delete, get, post, State},
};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::{
    config::AppConfig,
    controllers::invitation_controller::{
        InvitationAcceptForm, InvitationController, InviteMemberForm,
    },
    model::{InvitationContact, InvitationStatus, JidNode, MemberRole, ServerConfig},
    prose_xmpp::BareJid,
    repositories::InvitationToken,
    repositories::MemberRepository,
    services::notifier::Notifier,
    util::to_bare_jid,
};

use super::forms::InvitationTokenType;
use crate::{
    error::{self, Error},
    forms::{Timestamp, Uuid},
    guards::{Db, LazyGuard, UnauthenticatedInvitationService},
    model::SerializableSecretString,
    responders::Paginated,
    v1::{Created, R},
};

#[derive(Serialize, Deserialize)]
pub struct InviteMemberRequest {
    pub username: JidNode,
    #[serde(default)]
    pub pre_assigned_role: MemberRole,
    #[serde(flatten)]
    pub contact: InvitationContact,
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

/// Invite a new member.
#[post("/v1/invitations", format = "json", data = "<req>")]
pub(super) async fn invite_member<'r>(
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

    let resource_uri = uri!(get_invitation(invitation.id)).to_string();
    let response: WorkspaceInvitation = invitation.into();
    Ok(status::Created::new(resource_uri).body(response.into()))
}

/// Get workspace invitations.
#[get("/v1/invitations?<page_number>&<page_size>&<until>", rank = 2)]
pub(super) async fn get_invitations<'r>(
    invitation_controller: LazyGuard<InvitationController<'r>>,
    page_number: Option<u64>,
    page_size: Option<u64>,
    until: Option<Timestamp>,
) -> Result<Paginated<WorkspaceInvitation>, Error> {
    let invitation_controller = invitation_controller.inner?;
    let page_number = page_number.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);
    let until: Option<DateTime<Utc>> = match until {
        Some(t) => Some(t.try_into()?),
        None => None,
    };

    let (pages_metadata, invitations) = invitation_controller
        .get_invitations(page_number, page_size, until)
        .await?;

    Ok(Paginated::new(
        invitations.into_iter().map(Into::into).collect(),
        page_number,
        page_size,
        pages_metadata,
    ))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceInvitation {
    pub invitation_id: i32,
    pub created_at: DateTime<Utc>,
    pub status: InvitationStatus,
    pub jid: BareJid,
    pub pre_assigned_role: MemberRole,
    pub contact: InvitationContact,
    pub accept_token_expires_at: DateTime<Utc>,
}

impl From<service::model::Invitation> for WorkspaceInvitation {
    fn from(value: service::model::Invitation) -> Self {
        Self {
            invitation_id: value.id,
            created_at: value.created_at,
            status: value.status,
            jid: to_bare_jid(&value.jid).unwrap(),
            pre_assigned_role: value.pre_assigned_role,
            contact: value.contact(),
            accept_token_expires_at: value.accept_token_expires_at,
        }
    }
}

/// Get information about a workspace invitation.
#[get("/v1/invitations/<_>", rank = 2)]
pub(super) fn get_invitation() -> R<WorkspaceInvitation> {
    Err(error::NotImplemented("Get invitation").into())
}

/// Get information about an invitation from an accept or reject token.
#[get("/v1/invitations/<token>?<token_type>", rank = 1)]
pub(super) async fn get_invitation_by_token<'r>(
    invitation_controller: LazyGuard<InvitationController<'r>>,
    token: Uuid,
    token_type: InvitationTokenType,
) -> R<WorkspaceInvitation> {
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

#[derive(Serialize, Deserialize)]
pub struct AcceptWorkspaceInvitationRequest {
    pub nickname: String,
    pub password: SerializableSecretString,
}

impl Into<InvitationAcceptForm> for AcceptWorkspaceInvitationRequest {
    fn into(self) -> InvitationAcceptForm {
        InvitationAcceptForm {
            nickname: self.nickname,
            password: self.password.into(),
        }
    }
}

/// Accept a workspace invitation.
#[put("/v1/invitations/<token>/accept", format = "json", data = "<req>")]
pub(super) async fn invitation_accept<'r>(
    invitation_controller: LazyGuard<InvitationController<'r>>,
    invitation_service: LazyGuard<UnauthenticatedInvitationService<'r>>,
    token: Uuid,
    req: Json<AcceptWorkspaceInvitationRequest>,
) -> Result<(), Error> {
    invitation_controller
        .inner?
        .accept(
            InvitationToken::from(*token.deref()),
            invitation_service.inner?.deref(),
            req.into_inner(),
        )
        .await?;

    Ok(())
}

/// Reject a workspace invitation.
#[put("/v1/invitations/<token>/reject")]
pub(super) async fn invitation_reject<'r>(
    invitation_controller: LazyGuard<InvitationController<'r>>,
    token: Uuid,
) -> Result<NoContent, Error> {
    invitation_controller
        .inner?
        .reject(InvitationToken::from(*token.deref()))
        .await?;

    Ok(NoContent)
}

/// Resend a workspace invitation.
#[post("/v1/invitations/<invitation_id>/resend")]
pub(super) async fn invitation_resend<'r>(
    conn: Connection<'r, Db>,
    invitation_controller: LazyGuard<InvitationController<'r>>,
    app_config: &State<AppConfig>,
    jid: LazyGuard<BareJid>,
    notifier: LazyGuard<Notifier<'r>>,
    invitation_id: i32,
) -> Result<NoContent, Error> {
    let db = conn.into_inner();
    let invitation_controller = invitation_controller.inner?;
    let notifier = notifier.inner?;

    let jid = jid.inner?;
    // TODO: Use a request guard instead of checking in the route body if the user can invitation members.
    if !MemberRepository::is_admin(db, &jid).await? {
        return Err(error::Forbidden(format!("<{jid}> is not an admin")).into());
    }

    invitation_controller
        .resend(&app_config, &notifier, invitation_id)
        .await?;

    Ok(NoContent)
}

/// Cancel a workspace invitation.
#[delete("/v1/invitations/<invitation_id>")]
pub(super) async fn invitation_cancel<'r>(
    conn: Connection<'r, Db>,
    invitation_controller: LazyGuard<InvitationController<'r>>,
    jid: LazyGuard<BareJid>,
    invitation_id: i32,
) -> Result<NoContent, Error> {
    let db = conn.into_inner();
    let invitation_controller = invitation_controller.inner?;

    let jid = jid.inner?;
    // TODO: Use a request guard instead of checking in the route body if the user can invitation members.
    if !MemberRepository::is_admin(db, &jid).await? {
        return Err(error::Forbidden(format!("<{jid}> is not an admin")).into());
    }

    invitation_controller.cancel(invitation_id).await?;

    Ok(NoContent)
}
