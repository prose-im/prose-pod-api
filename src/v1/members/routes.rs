// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::models::Member;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{get, put};

use crate::error::Error;
use crate::forms::JID as JIDUriParam;

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
pub(super) fn get_member(_member_id: JIDUriParam) -> String {
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
