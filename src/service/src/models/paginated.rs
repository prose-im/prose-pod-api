// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[derive(Debug)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub current_page: usize,
    pub page_size: usize,
    pub page_count: usize,
    pub item_count: usize,
}

impl<T> Paginated<T> {
    pub fn new(
        data: Vec<T>,
        current_page: usize,
        page_size: usize,
        metadata: crate::util::paginate::ItemsAndPagesNumber,
    ) -> Self {
        Self {
            data,
            current_page,
            page_size,
            page_count: metadata.number_of_pages,
            item_count: metadata.number_of_items,
        }
    }

    pub fn is_partial(&self) -> bool {
        // NOTE: Page number starts at `1` and `number_of_pages` can be `0` if there are `0` items.
        self.current_page < self.page_count
    }

    pub fn map<T2>(self, f: impl Fn(T) -> T2) -> Paginated<T2> {
        Paginated {
            data: self.data.into_iter().map(f).collect(),
            current_page: self.current_page,
            page_size: self.page_size,
            page_count: self.page_count,
            item_count: self.item_count,
        }
    }
}
