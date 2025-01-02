// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr as _;

use rocket::{
    form::{self, FromFormField, ValueField},
    http::uri::fmt::{FromUriParam, Query},
};
use serde_with::{DeserializeFromStr, SerializeDisplay};

// ========== TOKEN TYPES ==========

#[derive(Debug, PartialEq, Eq)]
#[derive(SerializeDisplay, DeserializeFromStr)]
#[derive(strum::Display, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum InvitationTokenType {
    Accept,
    Reject,
}

impl<'v> FromFormField<'v> for InvitationTokenType {
    fn from_value(field: ValueField<'v>) -> form::Result<'v, Self> {
        Self::from_str(field.value).ok().ok_or(
            field
                .unexpected()
                .with_name("invalid_workspace_invitation_token_type")
                .into(),
        )
    }
}

impl FromUriParam<Query, uuid::Uuid> for InvitationTokenType {
    type Target = String;

    fn from_uri_param(param: uuid::Uuid) -> Self::Target {
        param.to_string()
    }
}
