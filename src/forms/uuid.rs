// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt::Display, ops::Deref};

use rocket::form::{self, FromFormField, ValueField};
use rocket::http::uri::fmt::{FromUriParam, Query};

#[derive(Debug, Eq)]
pub struct Uuid(uuid::Uuid);

impl<'v> FromFormField<'v> for Uuid {
    fn from_value(field: ValueField<'v>) -> form::Result<'v, Self> {
        uuid::Uuid::parse_str(field.value)
            .map(Uuid)
            .ok()
            .ok_or(field.unexpected().with_name("invalid_uuid").into())
    }
}

impl FromUriParam<Query, uuid::Uuid> for Uuid {
    type Target = String;

    fn from_uri_param(param: uuid::Uuid) -> Self::Target {
        param.to_string()
    }
}

impl Deref for Uuid {
    type Target = uuid::Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq for Uuid {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<uuid::Uuid> for Uuid {
    fn eq(&self, other: &uuid::Uuid) -> bool {
        &self.0 == other
    }
}
