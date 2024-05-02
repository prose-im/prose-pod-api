// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{delete, get, post, put};

/// Get custom reactions defined in your workspace.
#[get("/v1/workspace/reactions")]
pub(super) fn get_workspace_reactions() -> String {
    todo!()
}

/// Add a custom reaction to your workspace.
#[post("/v1/workspace/reactions")]
pub(super) fn add_workspace_reaction() -> String {
    todo!()
}

/// Get details about a custom reaction defined in your workspace.
#[get("/v1/workspace/reactions/<_reaction_id>")]
pub(super) fn get_workspace_reaction(_reaction_id: &str) -> String {
    todo!()
}

/// Edit a custom reaction defined in your workspace.
#[put("/v1/workspace/reactions/<_reaction_id>")]
pub(super) fn edit_workspace_reaction(_reaction_id: &str) -> String {
    todo!()
}

/// Delete a custom reaction from your workspace.
#[delete("/v1/workspace/reactions/<_reaction_id>")]
pub(super) fn delete_workspace_reaction(_reaction_id: &str) -> String {
    // TODO: Allow batch deletion by accepting a list in `reaction_id`.
    todo!()
}
