// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{sea_orm_string, wrapper_type};

wrapper_type!(Url, url::Url [+FromStr]; serde_with::DeserializeFromStr);

impl Url {
    #[inline]
    pub fn parse(input: &str) -> Result<Self, url::ParseError> {
        url::Url::parse(input).map(Self)
    }
}

sea_orm_string!(Url);
