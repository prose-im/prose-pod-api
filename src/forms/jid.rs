// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;
use std::{fmt::Display, ops::Deref};

use entity::model;
use rocket::form::{self, FromFormField, ValueField};
use rocket::http::uri::fmt::{FromUriParam, Path, Query};
use rocket::request::FromParam;

#[derive(Debug, Eq)]
pub struct JID(pub(crate) model::JID);

impl<'v> FromFormField<'v> for JID {
    fn from_value(field: ValueField<'v>) -> form::Result<'v, Self> {
        model::JID::from_str(field.value)
            .map(Self)
            .ok()
            .ok_or(field.unexpected().with_name("invalid_jid").into())
    }
}

impl FromUriParam<Path, model::JID> for JID {
    type Target = String;

    fn from_uri_param(param: model::JID) -> Self::Target {
        param.to_string()
    }
}

impl FromUriParam<Query, model::JID> for JID {
    type Target = String;

    fn from_uri_param(param: model::JID) -> Self::Target {
        param.to_string()
    }
}

impl<'v> FromParam<'v> for JID {
    type Error = <model::JID as FromStr>::Err;

    fn from_param(param: &'v str) -> Result<Self, Self::Error> {
        model::JID::from_str(param).map(Self)
    }
}

impl Deref for JID {
    type Target = model::JID;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for JID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq for JID {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<model::JID> for JID {
    fn eq(&self, other: &model::JID) -> bool {
        &self.0 == other
    }
}
