// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::Display;
use std::str::FromStr;

use rocket::form::{self, FromFormField, ValueField};
use rocket::http::uri::fmt::{FromUriParam, Query};

const TOKEN_TYPE_ACCEPT: &'static str = "accept";
const TOKEN_TYPE_REJECT: &'static str = "reject";

#[derive(Debug, PartialEq, Eq)]
pub enum MemberInviteTokenType {
    Accept,
    Reject,
}

impl FromStr for MemberInviteTokenType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            TOKEN_TYPE_ACCEPT => Ok(Self::Accept),
            TOKEN_TYPE_REJECT => Ok(Self::Reject),
            s => Err(format!("Invalid member invite token type: {s}")),
        }
    }
}

impl<'v> FromFormField<'v> for MemberInviteTokenType {
    fn from_value(field: ValueField<'v>) -> form::Result<'v, Self> {
        Self::from_str(field.value).ok().ok_or(
            field
                .unexpected()
                .with_name("invalid_member_invite_token_type")
                .into(),
        )
    }
}

impl FromUriParam<Query, uuid::Uuid> for MemberInviteTokenType {
    type Target = String;

    fn from_uri_param(param: uuid::Uuid) -> Self::Target {
        param.to_string()
    }
}

impl Display for MemberInviteTokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Accept => TOKEN_TYPE_ACCEPT,
                Self::Reject => TOKEN_TYPE_REJECT,
            }
        )
    }
}
