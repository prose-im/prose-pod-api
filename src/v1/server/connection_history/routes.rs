// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::get;

/// Get user connect history (aggregated for each user, eg. John connected 3 times today).
#[utoipa::path(
    tag = "Server / Connection History",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[get("/v1/server/connection-history")]
pub(super) fn get_history() -> String {
    todo!()
}

/// Get connection audit log (with countries and IP addresses).
#[utoipa::path(
    tag = "Server / Connection History",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[get("/v1/server/connection-history/audit-log")]
pub(super) fn get_audit_log() -> String {
    todo!()
}

/// Get security events (eg. failed user logins).
#[utoipa::path(
    tag = "Server / Connection History",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[get("/v1/server/connection-history/security-events")]
pub(super) fn get_security_events() -> String {
    todo!()
}
