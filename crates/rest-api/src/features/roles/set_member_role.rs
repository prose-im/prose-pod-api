// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{put, response::status::NoContent};

use crate::error::{self, Error};

#[put("/v1/members/<_>/role")]
pub fn set_member_role_route() -> Result<NoContent, Error> {
    Err(error::NotImplemented("Set member role").into())
}
