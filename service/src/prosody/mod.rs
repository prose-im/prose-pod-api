// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod prosody_admin_rest;
mod prosody_config_from_db;
mod prosody_oauth2;
mod prosody_rest;

use entity::model::MemberRole;
pub use prosody_admin_rest::ProsodyAdminRest;
pub use prosody_config_from_db::{prosody_config_from_db, IntoProsody, ProsodyConfig};
pub use prosody_oauth2::{ProsodyOAuth2, ProsodyOAuth2Error};
pub use prosody_rest::ProsodyRest;

// ===== Mapping to Prosody =====

/// Map our types to their representation in Prosody.
///
/// E.g. our `ADMIN` role maps to `"prosody:admin"`.
trait AsProsody {
    fn as_prosody(&self) -> String;
}

impl AsProsody for MemberRole {
    /// NOTE: Built-in roles are defined in `mod_authz_internal.lua` (under section `-- Default roles`).
    fn as_prosody(&self) -> String {
        match self {
            MemberRole::Member => "prosody:member",
            MemberRole::Admin => "prosody:admin",
        }
        .to_string()
    }
}
