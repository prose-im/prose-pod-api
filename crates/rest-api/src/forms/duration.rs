// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use iso8601_duration::Duration as ISODuration;
use rocket::form::{self, FromFormField, ValueField};

#[derive(Debug, Clone, PartialEq)]
pub struct Duration(pub(crate) ISODuration);

impl<'v> FromFormField<'v> for Duration {
    fn from_value(field: ValueField<'v>) -> form::Result<'v, Self> {
        ISODuration::parse(field.value)
            .map(Self)
            .ok()
            .ok_or(field.unexpected().with_name("invalid_duration").into())
    }
}

impl Deref for Duration {
    type Target = ISODuration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<ISODuration> for Duration {
    fn into(self) -> ISODuration {
        self.0
    }
}
