// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::models::Member;
use rocket::{delete, get, post, put};

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

/// Invite a new member.
#[utoipa::path(
    tag = "Members",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[post("/v1/members")]
pub(super) fn invite_member() -> String {
    todo!()
}

/// Get all member invitations.
#[utoipa::path(
    tag = "Members",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[get("/v1/members/invites")]
pub(super) fn get_invites() -> String {
    todo!()
}

/// Get information about one member invitation.
#[utoipa::path(
    tag = "Members",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[get("/v1/members/invites/<_invite_id>")]
pub(super) fn get_invite(_invite_id: &str) -> String {
    todo!()
}

/// Cancel one member invitation.
#[utoipa::path(
    tag = "Members",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[delete("/v1/members/invites/<_invite_id>")]
pub(super) fn cancel_invite(_invite_id: &str) -> String {
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
