// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod kv_store;
pub(crate) mod migrations;

pub use kv_store::{KvRecord, KvStore};
