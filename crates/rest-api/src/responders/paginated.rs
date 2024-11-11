// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{http::Header, response::Responder, serde::json::Json};
use service::sea_orm::ItemsAndPagesNumber;

#[derive(Responder)]
#[response(content_type = "json")]
pub enum Paginated<T> {
    #[response(status = 206)]
    Partial(InnerPagination<T>),
    #[response(status = 200)]
    End(InnerPagination<T>),
}

#[derive(Responder)]
pub struct InnerPagination<T> {
    inner: Json<Vec<T>>,
    current_page: Header<'static>,
    page_size: Header<'static>,
    page_count: Header<'static>,
    item_count: Header<'static>,
}

impl<T> Paginated<T> {
    pub fn new(data: Vec<T>, page: u64, page_size: u64, metadata: ItemsAndPagesNumber) -> Self {
        let inner = InnerPagination {
            inner: data.into(),
            current_page: Header::new("Pagination-Current-Page", format!("{page}")),
            page_size: Header::new("Pagination-Page-Size", format!("{page_size}")),
            page_count: Header::new(
                "Pagination-Page-Count",
                format!("{}", metadata.number_of_pages),
            ),
            item_count: Header::new(
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
