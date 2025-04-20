// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::Deserialize;

#[derive(Deserialize)]
pub struct SearchQuery {
    #[serde(alias = "search")]
    pub q: String,
}
