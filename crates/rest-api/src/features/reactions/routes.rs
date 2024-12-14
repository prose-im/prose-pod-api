// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{delete, get, post, put, response::status::NoContent};

use crate::error::{self, Error};

/// Get custom reactions defined in your workspace.
#[get("/v1/workspace/reactions")]
pub fn get_workspace_reactions_route() -> Result<NoContent, Error> {
    Err(error::NotImplemented("Get workspace reactions").into())
}

pub async fn get_workspace_reactions_route_axum() {
    todo!()
}

/// Add a custom reaction to your workspace.
#[post("/v1/workspace/reactions")]
pub fn add_workspace_reaction_route() -> Result<NoContent, Error> {
    Err(error::NotImplemented("Add workspace reaction").into())
}

pub async fn add_workspace_reaction_route_axum() {
    todo!()
}

/// Get details about a custom reaction defined in your workspace.
#[get("/v1/workspace/reactions/<_>")]
pub fn get_workspace_reaction_route() -> Result<NoContent, Error> {
    Err(error::NotImplemented("Get workspace reaction").into())
}

pub async fn get_workspace_reaction_route_axum() {
    todo!()
}

/// Edit a custom reaction defined in your workspace.
#[put("/v1/workspace/reactions/<_>")]
pub fn edit_workspace_reaction_route() -> Result<NoContent, Error> {
    Err(error::NotImplemented("Edit workspace reaction").into())
}

pub async fn edit_workspace_reaction_route_axum() {
    todo!()
}

// TODO: Allow batch deletion by accepting a list in `reaction_id`.
/// Delete a custom reaction from your workspace.
#[delete("/v1/workspace/reactions/<_>")]
pub fn delete_workspace_reaction_route() -> Result<NoContent, Error> {
    Err(error::NotImplemented("Delete workspace reaction").into())
}

pub async fn delete_workspace_reaction_route_axum() {
    todo!()
}
