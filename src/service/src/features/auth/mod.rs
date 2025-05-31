// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

//! Authentication & authorization.

pub mod auth_controller;
pub mod auth_service;
pub mod errors;
pub mod live_auth_service;
mod models;
pub mod util;

pub use auth_service::{AuthService, AuthServiceImpl, AuthToken, UserInfo};
pub use live_auth_service::LiveAuthService;

pub use self::models::*;
