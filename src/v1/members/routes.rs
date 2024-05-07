// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::models::Member;
use entity::model::{MemberRole, JID};
use rocket::serde::json::Json;
use rocket::{get, put};

use crate::forms::JID as JIDUriParam;

/// Get all members
#[get("/v1/members")]
pub(super) fn get_members() -> Json<Vec<Member>> {
    vec![
        Member {
            jid: JID::new("valerian", "crisp.chat").unwrap(),
            name: "Valerian Saliou".to_string(),
            role: MemberRole::Admin,
        },
        Member {
            jid: JID::new("baptiste", "crisp.chat").unwrap(),
            name: "Baptiste Jamin".to_string(),
            role: MemberRole::Admin,
        },
    ]
    .into()
}

/// Search for members.
#[get("/v1/members/search")]
pub(super) fn search_members() -> String {
    todo!()
}

/// Get information about one member.
#[get("/v1/members/<jid>")]
pub(super) fn get_member(jid: JIDUriParam) -> String {
    let _jid = jid;
    todo!()
}

/// Change a member's role.
#[put("/v1/members/<_member_id>/role")]
pub(super) fn set_member_role(_member_id: &str) -> String {
    todo!()
}

/// Change a member's Multi-Factor Authentication (MFA) status.
#[put("/v1/members/<_member_id>/mfa")]
pub(super) fn set_member_mfa(_member_id: &str) -> String {
    todo!()
}

/// Log a member out from all of its devices.
#[put("/v1/members/<_member_id>/logout")]
pub(super) fn logout_member(_member_id: &str) -> String {
    todo!()
}
