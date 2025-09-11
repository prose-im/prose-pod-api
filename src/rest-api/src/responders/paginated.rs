// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    http::{HeaderValue, StatusCode},
    response::IntoResponse,
};
use serdev::Serialize;

#[derive(Debug)]
pub struct Paginated<T>(pub service::models::Paginated<T>);

impl<T> From<service::models::Paginated<T>> for Paginated<T> {
    fn from(value: service::models::Paginated<T>) -> Self {
        Self(value)
    }
}

impl<T> Paginated<T> {
    fn status(&self) -> StatusCode {
        if self.0.is_partial() {
            StatusCode::PARTIAL_CONTENT
        } else {
            StatusCode::OK
        }
    }
}

impl<T: Serialize> IntoResponse for Paginated<T> {
    fn into_response(self) -> axum::response::Response {
        fn u64_to_str(n: u64) -> HeaderValue {
            // NOTE: We can safely unwrap here as the string representation
            //   of a `u64` only uses ASCII characters.
            HeaderValue::from_str(&n.to_string()).unwrap()
        }
        IntoResponse::into_response((
            self.status(),
            [
                ("Pagination-Current-Page", u64_to_str(self.0.current_page)),
                ("Pagination-Page-Size", u64_to_str(self.0.page_size)),
                ("Pagination-Page-Count", u64_to_str(self.0.page_count)),
                ("Pagination-Item-Count", u64_to_str(self.0.item_count)),
            ],
            axum::Json(self.0.data),
        ))
    }
}
