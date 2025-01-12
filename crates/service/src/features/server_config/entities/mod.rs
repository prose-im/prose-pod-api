// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod server_config;

use crate::sea_orm_string_enum;

use super::TlsProfile;

sea_orm_string_enum!(TlsProfile);
