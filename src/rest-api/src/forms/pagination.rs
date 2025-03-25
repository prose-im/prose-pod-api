// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::Deserialize;

use super::Timestamp;

#[derive(Deserialize)]
pub struct Pagination {
    pub page_number: Option<u64>,
    pub page_size: Option<u64>,
    pub until: Option<Timestamp>,
}
