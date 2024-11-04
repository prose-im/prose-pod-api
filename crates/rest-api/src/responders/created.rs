// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{response::status, serde::json::Json};

use crate::error::Error;

pub type Created<T> = Result<status::Created<Json<T>>, Error>;
