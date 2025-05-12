// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[derive(Debug, PartialEq, Eq)]
#[derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)]
#[derive(strum::Display, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum InvitationTokenType {
    Accept,
    Reject,
}
