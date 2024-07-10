// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{delete, get, post, put, response::status::NoContent};

use crate::error::Error;

/// Get custom reactions defined in your workspace.
#[get("/v1/workspace/reactions")]
pub(super) fn get_workspace_reactions() -> Result<NoContent, Error> {
    Err(Error::NotImplemented("Get workspace reactions"))
}

/// Add a custom reaction to your workspace.
#[post("/v1/workspace/reactions")]
pub(super) fn add_workspace_reaction() -> Result<NoContent, Error> {
    Err(Error::NotImplemented("Add workspace reaction"))
}

/// Get details about a custom reaction defined in your workspace.
#[get("/v1/workspace/reactions/<_>")]
pub(super) fn get_workspace_reaction() -> Result<NoContent, Error> {
    Err(Error::NotImplemented("Get workspace reaction"))
}

/// Edit a custom reaction defined in your workspace.
#[put("/v1/workspace/reactions/<_>")]
pub(super) fn edit_workspace_reaction() -> Result<NoContent, Error> {
    Err(Error::NotImplemented("Edit workspace reaction"))
}

// TODO: Allow batch deletion by accepting a list in `reaction_id`.
/// Delete a custom reaction from your workspace.
#[delete("/v1/workspace/reactions/<_>")]
pub(super) fn delete_workspace_reaction() -> Result<NoContent, Error> {
    Err(Error::NotImplemented("Delete workspace reaction"))
}
