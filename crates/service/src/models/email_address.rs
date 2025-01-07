// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{sea_orm_string, wrapper_type};

wrapper_type!(EmailAddress, email_address::EmailAddress);

sea_orm_string!(EmailAddress);
