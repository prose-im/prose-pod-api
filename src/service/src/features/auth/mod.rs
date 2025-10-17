// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

//! Authentication & authorization.

pub mod auth_controller;
pub mod auth_service;
pub mod errors;
mod models;
mod password_reset_notification;
pub mod util;

pub use auth_service::{AuthService, AuthServiceImpl, LiveAuthService};

pub use self::models::*;
pub(crate) use self::recovery_emails::kv_store as recovery_emails_store;

mod recovery_emails {
    crate::gen_scoped_kv_store!(pub(crate) recovery_emails);
}
