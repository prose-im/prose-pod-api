// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use ::entity::member_invite;
use ::entity::model::{member_invite::MemberInviteContact, MemberRole};
use chrono::{DateTime, Utc};
use entity::model::JID;
use rocket::http::uri::{Host, Origin};
use rocket::response::status::{self, NoContent};
use rocket::serde::json::Json;
use rocket::{delete, get, post};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::sea_orm::{prelude::*, EntityTrait};
use service::Mutation;
use service::Query;

use crate::error::Error;
use crate::forms::{MemberInviteTokenType, Timestamp, Uuid};
use crate::guards::{Db, Notifier, UserFactory, JID as JIDGuard};
use crate::responders::Paginated;

pub type R<T> = Result<Json<T>, Error>;
pub type Created<T> = Result<status::Created<Json<T>>, Error>;

#[derive(Serialize, Deserialize)]
pub struct InviteMemberRequest {
    pub jid: JID,
    pub pre_assigned_role: MemberRole,
    #[serde(flatten)]
    pub contact: MemberInviteContact,
}

pub type InviteMemberResponse = member_invite::Model;

/// Invite a new member.
#[utoipa::path(
    tag = "Invites",
    responses(
        (status = 200, description = "Success", body = InviteMemberResponse),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 401, description = "Unauthorized", body = Error),
        (status = 409, description = "Pod already initialized", body = Error),
    )
)]
#[post("/v1/invites", format = "json", data = "<req>")]
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

    let invite = Mutation::create_member_invite(
        db,
        req.jid.clone(),
        req.pre_assigned_role,
        req.contact.clone(),
    )
    .await
    .map_err(Error::DbErr)?;
    let accept_token = invite.accept_token;
    let reject_token = invite.reject_token;

    notifier
        .inner?
        .send_member_invite(accept_token, reject_token)
        .await?;

    let resource_uri = match host {
        Some(host) => {
            let origin = Origin::parse_owned(host.to_string()).unwrap();
            uri!(origin, get_invite(invite.id)).to_string()
        }
        None => uri!(get_invite(invite.id)).to_string(),
    };
    Ok(status::Created::new(resource_uri).body(invite.into()))
}

/// Get member invitations.
#[utoipa::path(
    tag = "Invites",
    responses(
        (status = 200, description = "Success", body = Paginated<member_invite::Model>),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 401, description = "Unauthorized", body = Error),
        (status = 409, description = "Pod already initialized", body = Error),
    )
)]
#[get("/v1/invites?<page_number>&<page_size>&<until>", rank = 2)]
pub(super) async fn get_invites(
    conn: Connection<'_, Db>,
    page_number: Option<u64>,
    page_size: Option<u64>,
    until: Option<Timestamp>,
) -> Result<Paginated<member_invite::Model>, Error> {
    let db = conn.into_inner();
    let page_number = page_number.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);
    let until: Option<DateTime<Utc>> = match until {
        Some(t) => Some(t.try_into()?),
        None => None,
    };
    let (pages_metadata, invites) = Query::get_invites(db, page_number, page_size, until)
        .await
        .map_err(Error::DbErr)?;
    Ok(Paginated::new(
        invites,
        page_number,
        page_size,
        pages_metadata,
    ))
}

/// Get information about one member invitation.
#[utoipa::path(
    tag = "Invites",
    responses(
        (status = 200, description = "Success", body = member_invite::Model),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 401, description = "Unauthorized", body = Error),
        (status = 409, description = "Pod already initialized", body = Error),
    )
)]
#[get("/v1/invites/<_>")]
pub(super) fn get_invite() -> Json<member_invite::Model> {
    todo!()
}

#[derive(Serialize, Deserialize)]
pub struct GetInviteByTokenResponse {
    pub invite_id: i32,
    pub pre_assigned_role: MemberRole,
    pub accept_token_expires_at: DateTimeUtc,
}

impl From<member_invite::Model> for GetInviteByTokenResponse {
    fn from(value: member_invite::Model) -> Self {
        Self {
            invite_id: value.id,
            pre_assigned_role: value.pre_assigned_role,
            accept_token_expires_at: value.accept_token_expires_at,
        }
    }
}

/// Get information about an invitation from an accept or reject token.
#[utoipa::path(
    tag = "Invites",
    responses(
        (status = 200, description = "Success", body = GetInviteByTokenResponse),
        (status = 401, description = "Unauthorized", body = Error),
    )
)]
#[get("/v1/invites?<token>&<token_type>", rank = 1)]
pub(super) async fn get_invite_by_token(
    conn: Connection<'_, Db>,
    token: Uuid,
    token_type: MemberInviteTokenType,
) -> R<GetInviteByTokenResponse> {
    let db = conn.into_inner();
    let invite = match token_type {
        MemberInviteTokenType::Accept => Query::get_invite_by_accept_token(db, &token).await,
        MemberInviteTokenType::Reject => Query::get_invite_by_reject_token(db, &token).await,
    }
    .map_err(Error::DbErr)?;
    let Some(invite) = invite else {
        return Err(Error::Unauthorized);
    };

    let response: GetInviteByTokenResponse = invite.into();
    Ok(response.into())
}

#[derive(Serialize, Deserialize)]
pub struct AcceptInviteRequest {
    pub nickname: String,
    pub password: String,
}

/// Accept or reject a member invitation.
#[utoipa::path(
    tag = "Invites",
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 409, description = "Pod already initialized", body = Error),
    )
)]
#[post(
    "/v1/invites/<invite_id>?action=accept&<token>",
    format = "json",
    data = "<req>",
    rank = 1
)]
pub(super) async fn invite_accept(
    conn: Connection<'_, Db>,
    invite_id: i32,
    token: Uuid,
    user_factory: UserFactory<'_>,
    req: Json<AcceptInviteRequest>,
) -> Result<(), Error> {
    let db = conn.into_inner();

    // NOTE: We don't check that the invite status is "RECEIVED"
    //   because it would cause more useless edge cases.
    let invite = Query::get_invite_by_id(db, &invite_id)
        .await
        .map_err(Error::DbErr)?
        .ok_or(Error::NotFound {
            reason: format!("No invite with ID {invite_id}"),
        })?;
    if token != invite.accept_token {
        return Err(Error::Unauthorized);
    }
    if invite.accept_token_expires_at < Utc::now() {
        return Err(Error::NotFound {
            reason: "Invite accept token has expired".to_string(),
        });
    }

    user_factory
        .create_user(&invite.jid, &req.password, &req.nickname)
        .await?;

    Mutation::accept_invite(db, invite)
        .await
        .map_err(Error::MutationErr)?;

    Ok(())
}

/// Reject an invitation.
#[utoipa::path(
    tag = "Invites",
    responses(
        (status = 204, description = "Success"),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 409, description = "Pod already initialized", body = Error),
    )
)]
#[post("/v1/invites/<invite_id>?action=reject&<token>", rank = 3)]
pub(super) async fn invite_reject(
    conn: Connection<'_, Db>,
    invite_id: i32,
    token: Uuid,
) -> Result<NoContent, Error> {
    let db = conn.into_inner();

    // Nothing to do
    // NOTE: We don't check that the invite status is "RECEIVED"
    //   because it would cause more useless edge cases.

    let invite = Query::get_invite_by_id(db, &invite_id)
        .await
        .map_err(Error::DbErr)?
        .ok_or(Error::NotFound {
            reason: format!("No invite with ID {invite_id}"),
        })?;
    if token != invite.reject_token {
        return Err(Error::Unauthorized);
    }

    invite.delete(db).await.map_err(Error::DbErr)?;

    Ok(NoContent)
}

/// Resend a member invitation.
#[utoipa::path(
    tag = "Invites",
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 401, description = "Unauthorized", body = Error),
        (status = 409, description = "Pod already initialized", body = Error),
    )
)]
#[post("/v1/invites/<invite_id>?action=resend", rank = 2)]
pub(super) async fn invite_resend(
    conn: Connection<'_, Db>,
    jid: Option<JIDGuard>,
    notifier: Notifier<'_>,
    invite_id: i32,
) -> Result<(), Error> {
    let db = conn.into_inner();

    let Some(jid) = jid else {
        return Err(Error::Unauthorized);
    };
    // TODO: Use a request guard instead of checking in the route body if the user can invite members.
    if !Query::is_admin(db, &jid).await.map_err(Error::DbErr)? {
        return Err(Error::Unauthorized);
    }

    let invite = Query::get_invite_by_id(db, &invite_id)
        .await
        .map_err(Error::DbErr)?
        .ok_or(Error::NotFound {
            reason: format!("Could not find the invite with id '{invite_id}'"),
        })?;

    notifier
        .inner?
        .send_member_invite(invite.accept_token, invite.reject_token)
        .await?;

    Ok(())
}

/// Cancel a member invitation.
#[utoipa::path(
    tag = "Invites",
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 401, description = "Unauthorized", body = Error),
        (status = 409, description = "Pod already initialized", body = Error),
    )
)]
#[post("/v1/invites/<invite_id>?action=cancel", rank = 4)]
pub(super) async fn invite_cancel(
    conn: Connection<'_, Db>,
    jid: Option<JIDGuard>,
    invite_id: i32,
) -> Result<(), Error> {
    let db = conn.into_inner();

    let Some(jid) = jid else {
        return Err(Error::Unauthorized);
    };
    // TODO: Use a request guard instead of checking in the route body if the user can invite members.
    if !Query::is_admin(db, &jid).await.map_err(Error::DbErr)? {
        return Err(Error::Unauthorized);
    }

    member_invite::Entity::delete_by_id(invite_id)
        .exec(db)
        .await
        .map_err(Error::DbErr)?;

    Ok(())
}

/// Cancel one member invitation.
#[utoipa::path(
    tag = "Invites",
    responses(
        (status = 200, description = "Success", body = member_invite::Model)
    )
)]
#[delete("/v1/invites/<_invite_id>")]
pub(super) fn cancel_invite(_invite_id: &str) -> Json<member_invite::Model> {
    todo!()
}
