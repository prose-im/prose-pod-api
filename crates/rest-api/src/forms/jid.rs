// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;
use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

use rocket::form::{self, FromFormField, ValueField};
use rocket::http::uri::fmt::{FromUriParam, Path, Query};
use rocket::request::FromParam;
use service::models::BareJid;

#[derive(Clone, Eq)]
pub struct JID(pub(crate) BareJid);

impl<'v> FromFormField<'v> for JID {
    fn from_value(field: ValueField<'v>) -> form::Result<'v, Self> {
        BareJid::from_str(field.value)
            .map(Self)
            .ok()
            .ok_or(field.unexpected().with_name("invalid_jid").into())
    }
}

impl FromUriParam<Path, BareJid> for JID {
    type Target = String;

    fn from_uri_param(param: BareJid) -> Self::Target {
        param.to_string()
    }
}

impl FromUriParam<Query, BareJid> for JID {
    type Target = String;

    fn from_uri_param(param: BareJid) -> Self::Target {
        param.to_string()
    }
}

impl<'v> FromParam<'v> for JID {
    type Error = <BareJid as FromStr>::Err;

    fn from_param(param: &'v str) -> Result<Self, Self::Error> {
        BareJid::from_str(param).map(Self)
    }
}

impl Deref for JID {
    type Target = BareJid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for JID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Debug for JID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

impl PartialEq for JID {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<BareJid> for JID {
    fn eq(&self, other: &BareJid) -> bool {
        &self.0 == other
    }
}
