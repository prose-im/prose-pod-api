// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod prosody_admin_rest;
mod prosody_bootstrap_config;
pub mod prosody_config;
mod prosody_config_from_db;
pub mod prosody_http_admin_api;
mod prosody_oauth2;
mod prosody_prose_version;
mod prosody_rest;

use crate::members::MemberRole;
pub use prosody_admin_rest::ProsodyAdminRest;
pub use prosody_bootstrap_config::prosody_bootstrap_config;
pub use prosody_config::ProsodyConfig;
pub use prosody_config_from_db::{prosody_config_from_db, IntoProsody};
pub use prosody_http_admin_api::ProsodyHttpAdminApi;
pub use prosody_oauth2::{ProsodyOAuth2, ProsodyOAuth2Error};
pub use prosody_prose_version::ProsodyProseVersion;
pub use prosody_rest::ProsodyRest;

// MARK: - Mapping to Prosody

/// Map our types to their representation in Prosody.
///
/// E.g. our `ADMIN` role maps to `"prosody:admin"`.
pub trait AsProsody {
    fn as_prosody(&self) -> String;
}

impl AsProsody for MemberRole {
    /// NOTE: Built-in roles are defined in `mod_authz_internal.lua` (under section `-- Default roles`).
    #[inline]
    fn as_prosody(&self) -> String {
        match self {
            MemberRole::Member => "prosody:member",
            MemberRole::Admin => "prosody:admin",
        }
        .to_string()
    }
}
