// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use ::entity::model::{MemberRole, JID};
use ::entity::workspace_invitation::{self, InvitationContact, InvitationStatus};
use chrono::{DateTime, Utc};
use rocket::http::uri::{Host, Origin};
use rocket::response::status::{self, NoContent};
use rocket::serde::json::Json;
use rocket::{delete, get, post};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::sea_orm::{prelude::*, EntityTrait};
use service::{Mutation, Query};

use super::forms::InvitationTokenType;
use crate::error::Error;
use crate::forms::{Timestamp, Uuid};
use crate::guards::{Db, Notifier, UserFactory, JID as JIDGuard};
use crate::responders::Paginated;

pub type R<T> = Result<Json<T>, Error>;
pub type Created<T> = Result<status::Created<Json<T>>, Error>;

#[derive(Serialize, Deserialize)]
pub struct InviteMemberRequest {
    pub jid: JID,
    pub pre_assigned_role: MemberRole,
    #[serde(flatten)]
    pub contact: InvitationContact,
}

pub type InviteMemberResponse = workspace_invitation::Model;

/// Invite a new member.
#[utoipa::path(
    tag = "Invitations",
    responses(
        (status = 200, description = "Success", body = InviteMemberResponse),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 401, description = "Unauthorized", body = Error),
    )
)]
#[post("/v1/invitations", format = "json", data = "<req>")]
pub(super) async fn invite_member<'r>(
    host: Option<&'r Host<'r>>,
    conn: Connection<'_, Db>,
    jid: JIDGuard,
    notifier: Notifier<'_>,
    req: Json<InviteMemberRequest>,
) -> Created<InviteMemberResponse> {
    let db = conn.into_inner();

    // TODO: Use a request guard instead of checking in the route body if the user can invite members.
    if !Query::is_admin(db, &jid).await.map_err(Error::DbErr)? {
        return Err(Error::Unauthorized);
    }

    let invitation = Mutation::create_workspace_invitation(
        db,
        req.jid.clone(),
        req.pre_assigned_role,
        req.contact.clone(),
    )
    .await
    .map_err(Error::DbErr)?;

    if let Err(err) = notifier
        .inner?
        .send_workspace_invitation(invitation.accept_token, invitation.reject_token)
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
                    "Could not mark workspace invitation as `{}`: {err}",
                    InvitationStatus::SendFailed
                )
            },
            |_| (),
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

    let resource_uri = match host {
        Some(host) => {
            let origin = Origin::parse_owned(host.to_string()).unwrap();
            uri!(origin, get_invitation(invitation.id)).to_string()
        }
        None => uri!(get_invitation(invitation.id)).to_string(),
    };
    Ok(status::Created::new(resource_uri).body(invitation.into()))
}

/// Get workspace invitations.
#[utoipa::path(
    tag = "Invitations",
    responses(
        (status = 200, description = "Success", body = Paginated<workspace_invitation::Model>),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 401, description = "Unauthorized", body = Error),
    )
)]
#[get("/v1/invitations?<page_number>&<page_size>&<until>", rank = 2)]
pub(super) async fn get_invitations(
    conn: Connection<'_, Db>,
    page_number: Option<u64>,
    page_size: Option<u64>,
    until: Option<Timestamp>,
) -> Result<Paginated<workspace_invitation::Model>, Error> {
    let db = conn.into_inner();
    let page_number = page_number.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);
    let until: Option<DateTime<Utc>> = match until {
        Some(t) => Some(t.try_into()?),
        None => None,
    };
    let (pages_metadata, invitations) =
        Query::get_workspace_invitations(db, page_number, page_size, until)
            .await
            .map_err(Error::DbErr)?;
    Ok(Paginated::new(
        invitations,
        page_number,
        page_size,
        pages_metadata,
    ))
}

/// Get information about a workspace invitation.
#[utoipa::path(
    tag = "Invitations",
    responses(
        (status = 200, description = "Success", body = workspace_invitation::Model),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 401, description = "Unauthorized", body = Error),
    )
)]
#[get("/v1/invitations/<_>")]
pub(super) fn get_invitation() -> Json<workspace_invitation::Model> {
    todo!()
}

#[derive(Serialize, Deserialize)]
pub struct GetWorkspaceInvitationByTokenResponse {
    pub invitation_id: i32,
    pub pre_assigned_role: MemberRole,
    pub accept_token_expires_at: DateTimeUtc,
}

impl From<workspace_invitation::Model> for GetWorkspaceInvitationByTokenResponse {
    fn from(value: workspace_invitation::Model) -> Self {
        Self {
            invitation_id: value.id,
            pre_assigned_role: value.pre_assigned_role,
            accept_token_expires_at: value.accept_token_expires_at,
        }
    }
}

/// Get information about an invitation from an accept or reject token.
#[utoipa::path(
    tag = "Invitations",
    responses(
        (status = 200, description = "Success", body = GetWorkspaceInvitationByTokenResponse),
        (status = 401, description = "Unauthorized", body = Error),
    )
)]
#[get("/v1/invitations?<token>&<token_type>", rank = 1)]
pub(super) async fn get_invitation_by_token(
    conn: Connection<'_, Db>,
    token: Uuid,
    token_type: InvitationTokenType,
) -> R<GetWorkspaceInvitationByTokenResponse> {
    let db = conn.into_inner();
    let invitation = match token_type {
        InvitationTokenType::Accept => {
            Query::get_workspace_invitation_by_accept_token(db, &token).await
        }
        InvitationTokenType::Reject => {
            Query::get_workspace_invitation_by_reject_token(db, &token).await
        }
    }
    .map_err(Error::DbErr)?;
    let Some(invitation) = invitation else {
        return Err(Error::Unauthorized);
    };

    let response: GetWorkspaceInvitationByTokenResponse = invitation.into();
    Ok(response.into())
}

#[derive(Serialize, Deserialize)]
pub struct AcceptWorkspaceInvitationRequest {
    pub nickname: String,
    pub password: String,
}

/// Accept or reject a workspace invitation.
#[utoipa::path(
    tag = "Invitations",
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Pod not initialized", body = Error),
    )
)]
#[post(
    "/v1/invitations/<invitation_id>?action=accept&<token>",
    format = "json",
    data = "<req>",
    rank = 1
)]
pub(super) async fn invitation_accept(
    conn: Connection<'_, Db>,
    invitation_id: i32,
    token: Uuid,
    user_factory: UserFactory<'_>,
    req: Json<AcceptWorkspaceInvitationRequest>,
) -> Result<(), Error> {
    let db = conn.into_inner();

    // NOTE: We don't check that the invitation status is "SENT"
    //   because it would cause a lot of useless edge cases.
    let invitation = Query::get_workspace_invitation_by_id(db, &invitation_id)
        .await
        .map_err(Error::DbErr)?
        .ok_or(Error::NotFound {
            reason: format!("No invitation with ID {invitation_id}"),
        })?;
    if token != invitation.accept_token {
        return Err(Error::Unauthorized);
    }
    if invitation.accept_token_expires_at < Utc::now() {
        return Err(Error::NotFound {
            reason: "Invitation accept token has expired".to_string(),
        });
    }

    user_factory
        .create_user(&invitation.jid, &req.password, &req.nickname)
        .await?;

    Mutation::accept_workspace_invitation(db, invitation)
        .await
        .map_err(Error::MutationErr)?;

    Ok(())
}

/// Reject an invitation.
#[utoipa::path(
    tag = "Invitations",
    responses(
        (status = 204, description = "Success"),
        (status = 400, description = "Pod not initialized", body = Error),
    )
)]
#[post("/v1/invitations/<invitation_id>?action=reject&<token>", rank = 3)]
pub(super) async fn invitation_reject(
    conn: Connection<'_, Db>,
    invitation_id: i32,
    token: Uuid,
) -> Result<NoContent, Error> {
    let db = conn.into_inner();

    // Nothing to do
    // NOTE: We don't check that the invitation status is "SENT"
    //   because it would cause a lot of useless edge cases.

    let invitation = Query::get_workspace_invitation_by_id(db, &invitation_id)
        .await
        .map_err(Error::DbErr)?
        .ok_or(Error::NotFound {
            reason: format!("No invitation with ID {invitation_id}"),
        })?;
    if token != invitation.reject_token {
        return Err(Error::Unauthorized);
    }

    invitation.delete(db).await.map_err(Error::DbErr)?;

    Ok(NoContent)
}

/// Resend a workspace invitation.
#[utoipa::path(
    tag = "Invitations",
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 401, description = "Unauthorized", body = Error),
    )
)]
#[post("/v1/invitations/<invitation_id>?action=resend", rank = 2)]
pub(super) async fn invitation_resend(
    conn: Connection<'_, Db>,
    jid: Option<JIDGuard>,
    notifier: Notifier<'_>,
    invitation_id: i32,
) -> Result<(), Error> {
    let db = conn.into_inner();

    let Some(jid) = jid else {
        return Err(Error::Unauthorized);
    };
    // TODO: Use a request guard instead of checking in the route body if the user can invitation members.
    if !Query::is_admin(db, &jid).await.map_err(Error::DbErr)? {
        return Err(Error::Unauthorized);
    }

    let invitation = Query::get_workspace_invitation_by_id(db, &invitation_id)
        .await
        .map_err(Error::DbErr)?
        .ok_or(Error::NotFound {
            reason: format!("Could not find the invitation with id '{invitation_id}'"),
        })?;

    notifier
        .inner?
        .send_workspace_invitation(invitation.accept_token, invitation.reject_token)
        .await?;

    Ok(())
}

/// Cancel a workspace invitation.
#[utoipa::path(
    tag = "Invitations",
    responses(
        (status = 204, description = "Success"),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 401, description = "Unauthorized", body = Error),
    )
)]
#[delete("/v1/invitations/<invitation_id>")]
pub(super) async fn invitation_cancel(
    conn: Connection<'_, Db>,
    jid: Option<JIDGuard>,
    invitation_id: i32,
) -> Result<NoContent, Error> {
    let db = conn.into_inner();

    let Some(jid) = jid else {
        return Err(Error::Unauthorized);
    };
    // TODO: Use a request guard instead of checking in the route body if the user can invitation members.
    if !Query::is_admin(db, &jid).await.map_err(Error::DbErr)? {
        return Err(Error::Unauthorized);
    }

    workspace_invitation::Entity::delete_by_id(invitation_id)
        .exec(db)
        .await
        .map_err(Error::DbErr)?;

    Ok(NoContent)
}
