// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::Display;

use super::models::Member;
use ::entity::member_invite;
use ::entity::model::{member_invite::MemberInviteContact, MemberRole};
use chrono::{DateTime, Utc};
use rocket::form::{Errors, FromFormField, ValueField};
use rocket::response::status::{self, NoContent};
use rocket::serde::json::Json;
use rocket::{delete, get, post, put};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::sea_orm::EntityTrait;
use service::Mutation;
use service::Query;

use crate::error::Error;
use crate::forms::{Timestamp, Uuid};
use crate::guards::{Db, JID as JIDGuard};
use crate::responders::Paginated;

pub type R<T> = Result<Json<T>, Error>;
pub type Created<T> = Result<status::Created<Json<T>>, Error>;

/// Get all members
#[utoipa::path(
    tag = "Members",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[get("/v1/members")]
pub(super) fn get_members() -> String {
    let members = vec![
        Member {
            jid: "valerian@crisp.chat".to_string(),
            name: "Valerian Saliou".to_string(),
        },
        Member {
            jid: "baptiste@crisp.chat".to_string(),
            name: "Baptiste Jamin".to_string(),
        },
    ];
    members
        .iter()
        .map(|m| format!("{:?}", m))
        .collect::<Vec<_>>()
        .join(",")
}

#[derive(Serialize, Deserialize)]
pub struct InviteMemberRequest {
    pub pre_assigned_role: MemberRole,
    #[serde(flatten)]
    pub contact: MemberInviteContact,
}

pub type InviteMemberResponse = member_invite::Model;

/// Invite a new member.
#[utoipa::path(
    tag = "Members",
    responses(
        (status = 200, description = "Success", body = InviteMemberResponse),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 401, description = "Unauthorized", body = Error),
        (status = 409, description = "Pod already initialized", body = Error),
    )
)]
#[post("/v1/members/invites", format = "json", data = "<req>")]
pub(super) async fn invite_member(
    conn: Connection<'_, Db>,
    jid: JIDGuard,
    req: Json<InviteMemberRequest>,
) -> Created<InviteMemberResponse> {
    let db = conn.into_inner();

    // TODO: Use a request guard instead of checking in the route body if the user can invite members.
    if !Query::is_admin(db, &jid).await.map_err(Error::DbErr)? {
        return Err(Error::Unauthorized);
    }

    let invite = Mutation::create_member_invite(db, req.pre_assigned_role, req.contact.clone())
        .await
        .map_err(Error::DbErr)?;
    Ok(status::Created::new("http://test.org").body(invite.into()))
    // Ok(status::Created::new(uri!(get_invite(invite.id)).to_string()).body(invite.into()))
}

/// Get member invitations.
#[utoipa::path(
    tag = "Members",
    responses(
        (status = 200, description = "Success", body = Paginated<member_invite::Model>),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 401, description = "Unauthorized", body = Error),
        (status = 409, description = "Pod already initialized", body = Error),
    )
)]
#[get("/v1/members/invites?<page_number>&<page_size>&<until>")]
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
    tag = "Members",
    responses(
        (status = 200, description = "Success", body = member_invite::Model),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 401, description = "Unauthorized", body = Error),
        (status = 409, description = "Pod already initialized", body = Error),
    )
)]
#[get("/v1/members/invites/<_>")]
pub(super) fn get_invite() -> Json<member_invite::Model> {
    todo!()
}

pub enum InviteAction {
    Accept,
    Reject,
}

impl<'v> FromFormField<'v> for InviteAction {
    fn from_value(field: ValueField<'v>) -> Result<Self, Errors<'v>> {
        match field.value {
            "accept" => Ok(Self::Accept),
            "reject" => Ok(Self::Reject),
            _ => Err(field.unexpected())?,
        }
    }
}

impl Display for InviteAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Accept => write!(f, "accept"),
            Self::Reject => write!(f, "reject"),
        }
    }
}

/// Accept or reject a member invitation.
#[utoipa::path(
    tag = "Members",
    responses(
        (status = 204, description = "Success"),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 409, description = "Pod already initialized", body = Error),
    )
)]
#[post("/v1/members/invites/<invite_id>?<action>&<token>")]
pub(super) async fn invite_action(
    conn: Connection<'_, Db>,
    _jid: Option<JIDGuard>,
    invite_id: i32,
    action: InviteAction,
    token: Uuid,
) -> Result<NoContent, Error> {
    let db = conn.into_inner();

    // NOTE: We don't check that the invite status is "RECEIVED"
    //   because it would cause more useless edge cases.
    match action {
        InviteAction::Accept => {
            let model = Query::get_invite_by_accept_token(db, &token)
                .await
                .map_err(Error::DbErr)?;
            let Some(model) = model else {
                return Err(Error::NotFound {
                    reason: "No invite found for given accept token",
                });
            };

            if model.accept_token_expires_at < Utc::now() {
                return Err(Error::NotFound {
                    reason: "Invite accept token has expired",
                });
            }

            // FIXME: Add the new user.
        }
        InviteAction::Reject => {
            // Nothing to do
        }
    }

    member_invite::Entity::delete_by_id(invite_id)
        .exec(db)
        .await
        .map_err(Error::DbErr)?;

    Ok(NoContent)
}

/// Resend a member invitation.
#[utoipa::path(
    tag = "Members",
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 401, description = "Unauthorized", body = Error),
        (status = 409, description = "Pod already initialized", body = Error),
    )
)]
#[post("/v1/members/invites/<invite_id>?action=resend", rank = 1)]
pub(super) async fn invite_resend(
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

    Ok(())
}

/// Cancel a member invitation.
#[utoipa::path(
    tag = "Members",
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 401, description = "Unauthorized", body = Error),
        (status = 409, description = "Pod already initialized", body = Error),
    )
)]
#[post("/v1/members/invites/<invite_id>?action=cancel", rank = 2)]
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
    tag = "Members",
    responses(
        (status = 200, description = "Success", body = member_invite::Model)
    )
)]
#[delete("/v1/members/invites/<_invite_id>")]
pub(super) fn cancel_invite(_invite_id: &str) -> Json<member_invite::Model> {
    todo!()
}

/// Search for members.
#[utoipa::path(
    tag = "Members",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[get("/v1/members/search")]
pub(super) fn search_members() -> String {
    todo!()
}

/// Get information about one member.
#[utoipa::path(
    tag = "Members",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[get("/v1/members/<_member_id>")]
pub(super) fn get_member(_member_id: &str) -> String {
    todo!()
}

/// Change a member's role.
#[utoipa::path(
    tag = "Members",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[put("/v1/members/<_member_id>/role")]
pub(super) fn set_member_role(_member_id: &str) -> String {
    todo!()
}

/// Change a member's Multi-Factor Authentication (MFA) status.
#[utoipa::path(
    tag = "Members",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[put("/v1/members/<_member_id>/mfa")]
pub(super) fn set_member_mfa(_member_id: &str) -> String {
    todo!()
}

/// Log a member out from all of its devices.
#[utoipa::path(
    tag = "Members",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[put("/v1/members/<_member_id>/logout")]
pub(super) fn logout_member(_member_id: &str) -> String {
    todo!()
}
