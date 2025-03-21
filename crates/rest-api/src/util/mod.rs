// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod content_type_or;
pub mod database;
mod error_catcher;
pub mod tracing_subscriber_ext;

pub use error_catcher::error_catcher;
