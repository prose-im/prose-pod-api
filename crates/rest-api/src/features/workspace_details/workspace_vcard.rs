// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{get, put, response::status::NoContent};

use crate::error::{self, Error};

#[get("/v1/workspace/details-card")]
pub fn get_workspace_details_card_route() -> Result<NoContent, Error> {
    Err(error::NotImplemented("Get workspace vCard").into())
}

#[put("/v1/workspace/details-card")]
pub fn set_workspace_details_card_route() -> Result<NoContent, Error> {
    Err(error::NotImplemented("Set workspace vCard").into())
}
