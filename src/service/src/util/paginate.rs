// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[derive(Clone, Debug)]
pub struct ItemsAndPagesNumber {
    pub number_of_items: usize,
    pub number_of_pages: usize,
}

#[inline]
pub fn paginate_iter<T: Clone>(
    full_list: impl Iterator<Item = T> + Clone,
    page_number: usize,
    page_size: usize,
) -> (ItemsAndPagesNumber, Vec<T>) {
    let number_of_items = full_list.clone().count();
    let number_of_pages = number_of_items.div_ceil(page_size);
    let pages_metadata = ItemsAndPagesNumber {
        number_of_items,
        number_of_pages,
    };

    let start = (page_number - 1) * page_size;
    let items: Vec<T> = full_list.skip(start).take(page_size).collect();

    (pages_metadata, items)
}

#[inline]
pub fn paginate_vec<T>(
    full_list: &Vec<T>,
    page_number: usize,
    page_size: usize,
) -> (ItemsAndPagesNumber, &[T]) {
    let number_of_items = full_list.len();
    let number_of_pages = number_of_items.div_ceil(page_size);
    let pages_metadata = ItemsAndPagesNumber {
        number_of_items: number_of_items,
        number_of_pages,
    };

    let start = (page_number - 1) * page_size;
    let end = std::cmp::min(start + page_size, number_of_items);
    let items: &[T] = full_list
        .get(start..end)
        .expect("Range should be in bounds");

    (pages_metadata, items)
}

#[inline]
pub fn paginate_vec_to_vec<T: Clone>(
    full_list: &Vec<T>,
    page_number: usize,
    page_size: usize,
) -> (ItemsAndPagesNumber, Vec<T>) {
    let (metadata, slice) = self::paginate_vec(full_list, page_number, page_size);
    (metadata, slice.to_vec())
}
