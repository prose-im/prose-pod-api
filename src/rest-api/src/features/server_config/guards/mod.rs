// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod server_config;

pub mod prelude {
    pub use axum::{extract::FromRequestParts, http::request};

    pub use crate::{
        error::{self, Error},
        AppState,
    };
}
