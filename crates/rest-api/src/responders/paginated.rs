// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    http::{HeaderValue, StatusCode},
    response::IntoResponse,
};
use rocket::{http::Header as HeaderRocket, response::Responder};
use serde::Serialize;
use service::sea_orm::ItemsAndPagesNumber;

#[derive(Responder)]
#[response(content_type = "json")]
pub enum PaginatedRocket<T> {
    #[response(status = 206)]
    Partial(InnerPaginationRocket<T>),
    #[response(status = 200)]
    End(InnerPaginationRocket<T>),
}

#[derive(Responder)]
pub struct InnerPaginationRocket<T> {
    inner: rocket::serde::json::Json<Vec<T>>,
    current_page: HeaderRocket<'static>,
    page_size: HeaderRocket<'static>,
    page_count: HeaderRocket<'static>,
    item_count: HeaderRocket<'static>,
}

impl<T> PaginatedRocket<T> {
    pub fn new(data: Vec<T>, page: u64, page_size: u64, metadata: ItemsAndPagesNumber) -> Self {
        let inner = InnerPaginationRocket {
            inner: data.into(),
            current_page: HeaderRocket::new("Pagination-Current-Page", format!("{page}")),
            page_size: HeaderRocket::new("Pagination-Page-Size", format!("{page_size}")),
            page_count: HeaderRocket::new(
                "Pagination-Page-Count",
                format!("{}", metadata.number_of_pages),
            ),
            item_count: HeaderRocket::new(
                "Pagination-Item-Count",
                format!("{}", metadata.number_of_items),
            ),
        };
        // NOTE: Page number starts at `1` and `number_of_pages` can be `0` if there are `0` items.
        if page >= metadata.number_of_pages {
            Self::End(inner)
        } else {
            Self::Partial(inner)
        }
    }
}

pub struct Paginated<T> {
    data: Vec<T>,
    current_page: u64,
    page_size: u64,
    page_count: u64,
    item_count: u64,
}

impl<T> Paginated<T> {
    pub fn new(
        data: Vec<T>,
        current_page: u64,
        page_size: u64,
        metadata: ItemsAndPagesNumber,
    ) -> Self {
        Self {
            data,
            current_page,
            page_size,
            page_count: metadata.number_of_pages,
            item_count: metadata.number_of_items,
        }
    }

    fn status(&self) -> StatusCode {
        // NOTE: Page number starts at `1` and `number_of_pages` can be `0` if there are `0` items.
        if self.current_page >= self.page_count {
            StatusCode::OK
        } else {
            StatusCode::PARTIAL_CONTENT
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
                ("Pagination-Current-Page", u64_to_str(self.current_page)),
                ("Pagination-Page-Size", u64_to_str(self.page_size)),
                ("Pagination-Page-Count", u64_to_str(self.page_count)),
                ("Pagination-Item-Count", u64_to_str(self.item_count)),
            ],
            axum::Json(self.data),
        ))
    }
}
