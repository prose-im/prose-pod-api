// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::get;

/// Get how much data is stored on the server for everyone at once.
#[utoipa::path(
    tag = "Server / Data Storage",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[get("/v1/server/data/usage")]
pub(super) fn get_data_usage() -> String {
    todo!()
}

/// Get how much data is stored on the server per-user.
#[utoipa::path(
    tag = "Server / Data Storage",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[get("/v1/server/data/usage-per-user/<_member_id>")]
pub(super) fn get_data_usage_per_user(_member_id: &str) -> String {
    todo!()
}
