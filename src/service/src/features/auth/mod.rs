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
