// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{get, response::status::NoContent};

use crate::error::Error;

/// Get how many users are active.
#[get("/v1/server/insights/active-users")]
pub(super) fn get_active_users() -> Result<NoContent, Error> {
    Err(Error::NotImplemented("Get active users"))
}

/// Retrieve how many users are active in real-time (as Server Sent Events).
#[get("/v1/server/insights/active-users/stream")]
pub(super) fn stream_active_users() -> Result<NoContent, Error> {
    Err(Error::NotImplemented("Stream active users"))
}

/// Get server-to-server stats (message stats to/from external servers).
#[get("/v1/server/insights/server-to-server-stats")]
pub(super) fn get_server_to_server_stats() -> Result<NoContent, Error> {
    Err(Error::NotImplemented("s2s stats"))
}
