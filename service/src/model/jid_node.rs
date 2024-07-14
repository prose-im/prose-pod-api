// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr as _;

use sea_orm::sea_query;

use crate::{sea_orm_string, wrapper_type};

wrapper_type!(JidNode, jid::NodePart);

impl From<super::EmailAddress> for JidNode {
    fn from(value: super::EmailAddress) -> Self {
        // NOTE: Email adresses are already parsed, and their local part are equivalent to a JID node part.
        Self::from_str(value.local_part()).unwrap()
    }
}

sea_orm_string!(JidNode);
