// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{delete, get, post, put};

/// Get custom reactions defined in your workspace.
#[utoipa::path(
    tag = "Workspace / Reactions",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[get("/v1/workspace/reactions")]
pub(super) fn get_workspace_reactions() -> String {
    todo!()
}

/// Search for custom reactions in your workspace.
#[utoipa::path(
    tag = "Workspace / Reactions",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[get("/v1/workspace/reactions/search")]
pub(super) fn search_workspace_reactions() -> String {
    todo!()
}

/// Add a custom reaction to your workspace.
#[utoipa::path(
    tag = "Workspace / Reactions",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[post("/v1/workspace/reactions")]
pub(super) fn add_workspace_reaction() -> String {
    todo!()
}

/// Get details about a custom reaction defined in your workspace.
#[utoipa::path(
    tag = "Workspace / Reactions",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[get("/v1/workspace/reactions/<_reaction_id>")]
pub(super) fn get_workspace_reaction(_reaction_id: &str) -> String {
    todo!()
}

/// Edit a custom reaction defined in your workspace.
#[utoipa::path(
    tag = "Workspace / Reactions",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[put("/v1/workspace/reactions/<_reaction_id>")]
pub(super) fn edit_workspace_reaction(_reaction_id: &str) -> String {
    todo!()
}

/// Delete a custom reaction from your workspace.
#[utoipa::path(
    tag = "Workspace / Reactions",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[delete("/v1/workspace/reactions/<_reaction_id>")]
pub(super) fn delete_workspace_reaction(_reaction_id: &str) -> String {
    // TODO: Allow batch deletion by accepting a list in `reaction_id`.
    todo!()
}
