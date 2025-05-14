// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PaginationForm {
    pub page_number: Option<u64>,
    pub page_size: Option<u64>,
    pub until: Option<DateTime<Utc>>,
}

pub struct Pagination {
    pub page_number: u64,
    pub page_size: u64,
    pub until: Option<DateTime<Utc>>,
}
