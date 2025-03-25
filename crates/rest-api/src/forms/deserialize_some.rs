// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::{Deserialize, Deserializer};

/// Any value that is present is considered `Some` value, including `null`.
///
/// Copyright: [Treat null and missing field as being different · Issue #984 · serde-rs/serde](https://github.com/serde-rs/serde/issues/984#issuecomment-314143738).
pub fn deserialize_some<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}
