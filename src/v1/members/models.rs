// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use utoipa::ToSchema;

#[derive(ToSchema, Debug)]
pub struct Member {
    pub jid: String,
    pub name: String,
}
