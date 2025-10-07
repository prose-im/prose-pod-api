// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod prosody_admin_rest;
mod prosody_bootstrap_config;
pub mod prosody_config;
mod prosody_config_from_db;
pub mod prosody_http_admin_api;
pub mod prosody_invites_register_api;
mod prosody_oauth2;
mod prosody_prose_version;
mod prosody_rest;

use crate::members::MemberRole;
pub use prosody_admin_rest::ProsodyAdminRest;
pub use prosody_bootstrap_config::prosody_bootstrap_config;
pub use prosody_config::ProsodyConfig;
pub use prosody_config_from_db::{prosody_config_from_db, IntoProsody};
pub use prosody_http_admin_api::ProsodyHttpAdminApi;
pub use prosody_invites_register_api::ProsodyInvitesRegisterApi;
pub use prosody_oauth2::{ProsodyOAuth2, ProsodyOAuth2Error};
pub use prosody_prose_version::ProsodyProseVersion;
pub use prosody_rest::ProsodyRest;
use serdev::Deserialize;

// MARK: - Mapping to Prosody

/// Map our types to their representation in Prosody.
///
/// E.g. our `ADMIN` role maps to `"prosody:admin"`.
pub trait AsProsody {
    type ProsodyType;
    fn as_prosody(&self) -> Self::ProsodyType;
}

crate::wrapper_type!(ProsodyRoleName, String);

// See [Prosody built-in roles](https://prosody.im/doc/roles#built-in-roles).
impl ProsodyRoleName {
    pub const OPERATOR_RAW: &'static str = "prosody:operator";
    pub const ADMIN_RAW: &'static str = "prosody:admin";
    pub const MEMBER_RAW: &'static str = "prosody:member";
    pub const REGISTERED_RAW: &'static str = "prosody:registered";
    pub const GUEST_RAW: &'static str = "prosody:guest";
}

impl AsRef<str> for ProsodyRoleName {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl AsProsody for MemberRole {
    type ProsodyType = ProsodyRoleName;

    /// NOTE: Built-in roles are defined in `mod_authz_internal.lua` (under section `-- Default roles`).
    #[inline]
    fn as_prosody(&self) -> ProsodyRoleName {
        match self {
            MemberRole::Member => ProsodyRoleName(ProsodyRoleName::MEMBER_RAW.to_owned()),
            MemberRole::Admin => ProsodyRoleName(ProsodyRoleName::ADMIN_RAW.to_owned()),
        }
    }
}

impl TryFrom<&ProsodyRoleName> for MemberRole {
    type Error = UnsupportedProsodyRole;

    fn try_from(role: &ProsodyRoleName) -> Result<Self, Self::Error> {
        match role.as_str() {
            ProsodyRoleName::MEMBER_RAW => Ok(Self::Member),
            ProsodyRoleName::ADMIN_RAW => Ok(Self::Admin),
            _ => Err(UnsupportedProsodyRole(role.clone())),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProsodyRole {
    /// E.g. `"EHKt_OKcF-5K"`.
    pub id: String,
    /// E.g. `"prosody:member"`.
    pub name: ProsodyRoleName,
    /// E.g. `35`.
    pub priority: i16,
    #[serde(default)]
    pub inherits: Vec<ProsodyRole>,
}

impl ProsodyRole {
    /// Greater than or equal to another role (“is or inherits”).
    #[doc(alias = "inherits")]
    pub fn gte(&self, other: impl AsRef<str>) -> bool {
        let other = other.as_ref();

        let mut stack = vec![self];
        while let Some(role) = stack.pop() {
            if role.name.as_str() == other {
                return true;
            }
            // NOTE: We’re using a LIFO stack so we need to
            //   reverse the new entries to preserve order.
            stack.extend(role.inherits.iter().rev());
        }

        false
    }
}

impl TryFrom<&ProsodyRole> for MemberRole {
    type Error = UnsupportedProsodyRole;

    fn try_from(role: &ProsodyRole) -> Result<Self, Self::Error> {
        use crate::prosody::AsProsody as _;

        if role.gte(&MemberRole::Admin.as_prosody()) {
            Ok(MemberRole::Admin)
        } else if role.gte(&MemberRole::Member.as_prosody()) {
            Ok(MemberRole::Member)
        } else {
            // NOTE: Service accounts do not have the "prosody:member" role,
            //   only "prosody:registered" (> "prosody:guest").
            Err(UnsupportedProsodyRole(role.name.clone()))
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Unsupported Prosody role: {0}")]
pub struct UnsupportedProsodyRole(ProsodyRoleName);

// MARK: - Boilerplate

impl From<&str> for ProsodyRoleName {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
