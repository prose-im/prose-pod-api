// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use ::entity::model::{MemberRole, JID};
use ::entity::workspace_invitation::{self, InvitationContact, InvitationStatus};
use chrono::{DateTime, Utc};
use rocket::response::status::{self, NoContent};
use rocket::serde::json::Json;
use rocket::{delete, get, post, State};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::config::Config;
use service::sea_orm::{prelude::*, EntityTrait};
use service::{Mutation, Query};

use super::forms::InvitationTokenType;
use crate::error::Error;
use crate::forms::{Timestamp, Uuid};
use crate::guards::{Db, LazyGuard, Notifier, UserFactory, UuidGenerator, JID as JIDGuard};
use crate::responders::Paginated;

pub type R<T> = Result<Json<T>, Error>;
pub type Created<T> = Result<status::Created<Json<T>>, Error>;

#[derive(Serialize, Deserialize)]
pub struct InviteMemberRequest {
    // TODO: Validate user input
    pub username: String,
    #[serde(default)]
    pub pre_assigned_role: MemberRole,
    #[serde(flatten)]
    pub contact: InvitationContact,
}

/// Invite a new member.
#[post("/v1/invitations", format = "json", data = "<req>")]
pub(super) async fn invite_member<'r>(
    conn: Connection<'_, Db>,
    uuid_gen: UuidGenerator,
    config: &State<Config>,
    jid: LazyGuard<JIDGuard>,
    notifier: LazyGuard<Notifier<'_>>,
    req: Json<InviteMemberRequest>,
) -> Created<WorkspaceInvitation> {
    let db = conn.into_inner();
    let jid = jid.inner?;

    // TODO: Use a request guard instead of checking in the route body if the user can invite members.
    if !Query::is_admin(db, &jid).await? {
        debug!("<{}> is not an admin", jid.to_string());
        return Err(Error::Unauthorized);
    }

    let invitation = Mutation::create_workspace_invitation(
        db,
        &uuid_gen,
        JID::new(req.username.clone(), config.server.domain.clone()),
        req.pre_assigned_role,
        req.contact.clone(),
    )
    .await?;

    if let Err(err) = notifier
        .inner?
        .send_workspace_invitation(
            &config.branding,
            invitation.accept_token,
            invitation.reject_token,
        )
        .await
    {
        error!("Could not send workspace invitation: {err}");
        Mutation::update_workspace_invitation_status(
            db,
            invitation.clone(),
            InvitationStatus::SendFailed,
        )
        .await
        .map_or_else(
            |err| {
                error!(
                    "Could not mark workspace invitation `{}` as `{}`: {err}",
                    invitation.id,
                    InvitationStatus::SendFailed
                )
            },
            |_| {
                debug!(
                    "Marked invitation `{}` as `{}`",
                    invitation.id,
                    InvitationStatus::SendFailed
                )
            },
        );
    };

    Mutation::update_workspace_invitation_status(db, invitation.clone(), InvitationStatus::Sent)
        .await
        .inspect_err(|err| {
            error!(
                "Could not mark workspace invitation as `{}`: {err}",
                InvitationStatus::Sent
            )
        })?;

    let resource_uri = uri!(get_invitation(invitation.id)).to_string();
    let response: WorkspaceInvitation = invitation.into();
    Ok(status::Created::new(resource_uri).body(response.into()))
}

/// Get workspace invitations.
#[get("/v1/invitations?<page_number>&<page_size>&<until>", rank = 2)]
pub(super) async fn get_invitations(
    conn: Connection<'_, Db>,
    page_number: Option<u64>,
    page_size: Option<u64>,
    until: Option<Timestamp>,
) -> Result<Paginated<WorkspaceInvitation>, Error> {
    let db = conn.into_inner();
    let page_number = page_number.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);
    let until: Option<DateTime<Utc>> = match until {
        Some(t) => Some(t.try_into()?),
        None => None,
    };
    let (pages_metadata, invitations) =
        Query::get_workspace_invitations(db, page_number, page_size, until).await?;
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
    pub created_at: DateTimeUtc,
    pub status: InvitationStatus,
    pub jid: JID,
    pub pre_assigned_role: MemberRole,
    pub contact: InvitationContact,
    pub accept_token_expires_at: DateTimeUtc,
}

impl From<workspace_invitation::Model> for WorkspaceInvitation {
    fn from(value: workspace_invitation::Model) -> Self {
        Self {
            invitation_id: value.id,
            created_at: value.created_at,
            status: value.status,
            jid: value.jid.clone(),
            pre_assigned_role: value.pre_assigned_role,
            contact: value.contact(),
            accept_token_expires_at: value.accept_token_expires_at,
        }
    }
}

/// Get information about a workspace invitation.
#[get("/v1/invitations/<_>", rank = 2)]
pub(super) fn get_invitation() -> Json<WorkspaceInvitation> {
    todo!()
}

/// Get information about an invitation from an accept or reject token.
#[get("/v1/invitations/<token>?<token_type>", rank = 1)]
pub(super) async fn get_invitation_by_token(
    conn: Connection<'_, Db>,
    token: Uuid,
    token_type: InvitationTokenType,
) -> R<WorkspaceInvitation> {
    let db = conn.into_inner();
    let invitation = match token_type {
        InvitationTokenType::Accept => {
            Query::get_workspace_invitation_by_accept_token(db, &token).await
        }
        InvitationTokenType::Reject => {
            Query::get_workspace_invitation_by_reject_token(db, &token).await
        }
    }?;
    let Some(invitation) = invitation else {
        debug!("No invitation found for provided token");
        return Err(Error::Unauthorized);
    };

    let response: WorkspaceInvitation = invitation.into();
    Ok(response.into())
}

#[derive(Serialize, Deserialize)]
pub struct AcceptWorkspaceInvitationRequest {
    pub nickname: String,
    pub password: String,
}

/// Accept a workspace invitation.
#[put("/v1/invitations/<token>/accept", format = "json", data = "<req>")]
pub(super) async fn invitation_accept(
    conn: Connection<'_, Db>,
    token: Uuid,
    user_factory: UserFactory<'_>,
    req: Json<AcceptWorkspaceInvitationRequest>,
) -> Result<(), Error> {
    let db = conn.into_inner();

    // NOTE: We don't check that the invitation status is "SENT"
    //   because it would cause a lot of useless edge cases.
    let invitation = Query::get_workspace_invitation_by_accept_token(db, &token)
        .await?
        .ok_or_else(|| {
            debug!("No invitation found for provided token");
            Error::Unauthorized
        })?;
    if token != invitation.accept_token {
        debug!("Accept token is invalid");
        return Err(Error::Unauthorized);
    }
    if invitation.accept_token_expires_at < Utc::now() {
        return Err(Error::NotFound {
            reason: "Invitation accept token has expired".to_string(),
        });
    }

    user_factory
        .accept_workspace_invitation(db, invitation, &req.password, &req.nickname)
        .await?;

    Ok(())
}

/// Reject a workspace invitation.
#[put("/v1/invitations/<token>/reject")]
pub(super) async fn invitation_reject(
    conn: Connection<'_, Db>,
    token: Uuid,
) -> Result<NoContent, Error> {
    let db = conn.into_inner();

    // Nothing to do
    // NOTE: We don't check that the invitation status is "SENT"
    //   because it would cause a lot of useless edge cases.

    let invitation = Query::get_workspace_invitation_by_reject_token(db, &token)
        .await?
        .ok_or_else(|| {
            debug!("No invitation found for provided token");
            Error::Unauthorized
        })?;
    if token != invitation.reject_token {
        debug!("Reject token is invalid");
        return Err(Error::Unauthorized);
    }

    invitation.delete(db).await?;

    Ok(NoContent)
}

/// Resend a workspace invitation.
#[post("/v1/invitations/<invitation_id>/resend")]
pub(super) async fn invitation_resend(
    conn: Connection<'_, Db>,
    config: &State<Config>,
    jid: LazyGuard<JIDGuard>,
    notifier: LazyGuard<Notifier<'_>>,
    invitation_id: i32,
) -> Result<(), Error> {
    let db = conn.into_inner();

    let jid = jid.inner?;
    // TODO: Use a request guard instead of checking in the route body if the user can invitation members.
    if !Query::is_admin(db, &jid).await? {
        debug!("<{}> is not an admin", jid.to_string());
        return Err(Error::Unauthorized);
    }

    let invitation = Query::get_workspace_invitation_by_id(db, &invitation_id)
        .await?
        .ok_or(Error::NotFound {
            reason: format!("Could not find the invitation with id '{invitation_id}'"),
        })?;

    notifier
        .inner?
        .send_workspace_invitation(
            &config.branding,
            invitation.accept_token,
            invitation.reject_token,
        )
        .await?;

    Ok(())
}

/// Cancel a workspace invitation.
#[delete("/v1/invitations/<invitation_id>")]
pub(super) async fn invitation_cancel(
    conn: Connection<'_, Db>,
    jid: LazyGuard<JIDGuard>,
    invitation_id: i32,
) -> Result<NoContent, Error> {
    let db = conn.into_inner();

    let jid = jid.inner?;
    // TODO: Use a request guard instead of checking in the route body if the user can invitation members.
    if !Query::is_admin(db, &jid).await? {
        debug!("<{}> is not an admin", jid.to_string());
        return Err(Error::Unauthorized);
    }

    workspace_invitation::Entity::delete_by_id(invitation_id)
        .exec(db)
        .await?;

    Ok(NoContent)
}
