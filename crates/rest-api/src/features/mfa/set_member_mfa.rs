// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{put, response::status::NoContent};

use crate::error::{self, Error};

/// Change a member's Multi-Factor Authentication (MFA) status.
#[put("/v1/members/<_>/mfa")]
pub fn set_member_mfa_route() -> Result<NoContent, Error> {
    Err(error::NotImplemented("Set member MFA status").into())
}
