// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// WARN: When adding a new case to this enum, make sure to
//   add a new migration to update the column size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(strum::EnumIter, strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
#[derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)]
pub enum InvitationChannel {
    Email,
}
