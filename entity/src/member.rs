// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use sea_orm::{entity::prelude::*, Set};

use crate::model::{MemberRole, JID};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "member")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    id: String,
    pub role: MemberRole,
    pub joined_at: DateTimeUtc,
}

impl Model {
    pub fn jid(&self) -> JID {
        JID::from_str(&self.id).unwrap()
    }
}

impl ActiveModel {
    pub fn set_jid(&mut self, jid: &JID) {
        self.id = Set(jid.to_string());
    }
}

impl Entity {
    pub fn find_by_jid(jid: &JID) -> Select<Self> {
        Self::find_by_id(jid.to_string())
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
