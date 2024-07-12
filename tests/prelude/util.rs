// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::Debug;

pub fn assert_contains_if<S: Debug, T: Debug + ?Sized>(
    condition: bool,
    set: &S,
    value: &T,
    contains: impl Fn(&S, &T) -> bool,
) {
    if condition {
        assert!(contains(set, value), "{value:?} not found in {set:#?}");
    } else {
        assert!(!contains(set, value), "{value:?} found in {set:#?}");
    }
}
pub fn assert_defined_if<T: PartialEq + Debug>(condition: bool, value: Option<T>) {
    if condition {
        assert_ne!(value, None);
    } else {
        assert_eq!(value, None);
    }
}
