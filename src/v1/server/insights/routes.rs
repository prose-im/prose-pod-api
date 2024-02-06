// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::get;

/// Get how many users are active.
#[utoipa::path(
    tag = "Server / Insights",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[get("/v1/server/insights/active-users")]
pub(super) fn get_active_users() -> String {
    todo!()
}

/// Retrieve how many users are active in real-time (as Server Sent Events).
#[utoipa::path(
    tag = "Server / Insights",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[get("/v1/server/insights/active-users/stream")]
pub(super) fn stream_active_users() -> String {
    todo!()
}

/// Get server-to-server stats (message stats to/from external servers).
#[utoipa::path(
    tag = "Server / Insights",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[get("/v1/server/insights/server-to-server-stats")]
pub(super) fn get_server_to_server_stats() -> String {
    todo!()
}
