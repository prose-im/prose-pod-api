// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::{Deserialize, Serialize};

use entity::model::MemberRole;

#[derive(Debug, Serialize, Deserialize)]
pub struct Member {
    pub jid: String,
    pub name: String,
    pub role: MemberRole,
}
