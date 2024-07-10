// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{get, response::status::NoContent};

use crate::error::Error;

/// Get user connect history (aggregated for each user, eg. John connected 3 times today).
#[get("/v1/server/connection-history")]
pub(super) fn get_history() -> Result<NoContent, Error> {
    Err(Error::NotImplemented("Get user connect history"))
}

/// Get connection audit log (with countries and IP addresses).
#[get("/v1/server/connection-history/audit-log")]
pub(super) fn get_audit_log() -> Result<NoContent, Error> {
    Err(Error::NotImplemented("Get connection audit log"))
}

/// Get security events (eg. failed user logins).
#[get("/v1/server/connection-history/security-events")]
pub(super) fn get_security_events() -> Result<NoContent, Error> {
    Err(Error::NotImplemented("Get security events"))
}
