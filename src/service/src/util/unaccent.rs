// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};

/// COPYRIGHT: <https://github.com/crowdtech-io/unaccent/blob/83f4efb92976ae15db850ae6130d7664b242dba7/src/lib.rs#L39-L46>,
///   although it really is just a single function using
///   `unicode_normalization`.
pub fn unaccent<T: AsRef<str>>(input: T) -> String {
    input
        .as_ref()
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .nfc()
        .collect()
}
