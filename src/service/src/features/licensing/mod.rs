// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod license_service;
pub mod license_validator;
pub mod licensing_controller;

use biscuit::{AuthorizerBuilder, Biscuit};

pub use self::license_service::*;
pub use self::license_validator::*;

/// See https://github.com/eclipse-biscuit/website/commit/6284753980acceabbf0e18b784d93e3bfebe53fd#r163283775.
pub trait BiscuitExt {
    fn authorize(&self, authorizer: &AuthorizerBuilder) -> Result<usize, biscuit::error::Token>;
}

impl BiscuitExt for Biscuit {
    fn authorize(&self, authorizer: &AuthorizerBuilder) -> Result<usize, biscuit::error::Token> {
        authorizer.clone().build(self)?.authorize()
    }
}
