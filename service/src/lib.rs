// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod mutation;
mod query;

pub use mutation::*;
pub use query::*;

pub use sea_orm;
