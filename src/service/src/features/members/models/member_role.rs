// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::cmp::Ordering;

use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::{EnumIter, EnumString};

// NOTE: When adding a new case to this enum, make sure to
//   add a new migration to update the column size.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl PartialOrd for MemberRole {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (*self as u8).partial_cmp(&(*other as u8))
    }
}

crate::sea_orm_string!(MemberRole; enum);
