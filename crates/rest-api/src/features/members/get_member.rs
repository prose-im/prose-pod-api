// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{get, response::status::NoContent};

use crate::{
    error::{self, Error},
    forms::JID as JIDUriParam,
};

#[get("/v1/members/<_jid>")]
pub fn get_member_route(_jid: JIDUriParam) -> Result<NoContent, Error> {
    Err(error::NotImplemented("Get member").into())
}
