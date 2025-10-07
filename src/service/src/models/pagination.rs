// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};

#[derive(Debug)]
#[derive(serdev::Deserialize)]
pub struct PaginationForm {
    pub page_number: Option<usize>,
    pub page_size: Option<usize>,
    pub until: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct Pagination {
    pub page_number: usize,
    pub page_size: usize,
    pub until: Option<DateTime<Utc>>,
}
