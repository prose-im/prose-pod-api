// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::get;

/// Get server logs between two timestamps.
#[get("/v1/server/logs")]
pub(super) fn get_server_logs() -> String {
    todo!()
}

/// Retrieve real-time server logs (as Server Sent Events).
#[get("/v1/server/logs/stream")]
pub(super) fn stream_server_logs() -> String {
    todo!()
}
