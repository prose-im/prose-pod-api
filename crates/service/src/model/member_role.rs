// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm::sea_query;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::{EnumIter, EnumString, IntoEnumIterator as _};

use crate::sea_orm_string_enum;

// NOTE: When adding a new case to this enum, make sure to
//   add a new migration to update the column size.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[derive(SerializeDisplay, DeserializeFromStr)]
#[derive(EnumIter, EnumString, strum::Display)]
pub enum MemberRole {
    #[strum(serialize = "MEMBER")]
    Member,
    #[strum(serialize = "ADMIN")]
    Admin,
}

impl Default for MemberRole {
    fn default() -> Self {
        Self::Member
    }
}

sea_orm_string_enum!(MemberRole);
