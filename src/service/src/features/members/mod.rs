// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod entities;
pub mod member_controller;
pub mod member_repository;
pub mod member_service;
pub(crate) mod migrations;
pub mod models;
pub mod unauthenticated_member_service;

pub use member_repository::*;
pub use member_service::*;
pub use models::*;
pub use unauthenticated_member_service::*;

pub const NICKNAME_MAX_LENGTH: u64 = 48;
