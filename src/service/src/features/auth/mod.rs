// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

//! Authentication & authorization.

pub mod auth_controller;
pub mod auth_service;
pub mod errors;
pub mod live_auth_service;

pub use auth_service::{AuthService, AuthServiceImpl, AuthToken, UserInfo};
pub use live_auth_service::LiveAuthService;

pub use self::models::*;

mod models {
    use jid::BareJid;
    use secrecy::SecretString;

    pub struct Credentials {
        pub jid: BareJid,
        pub password: SecretString,
    }

    /// Ensures a user is logged in.
    pub struct Authenticated;

    /// Ensures the logged in user is an admin.
    ///
    /// It's not perfect, one day we'll replace it with scopes and permissions,
    /// but it'll do for now.
    pub struct IsAdmin;
}
