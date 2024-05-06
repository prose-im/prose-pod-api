// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use sea_orm::{entity::prelude::*, Set};

use crate::model::{JIDNode, MemberRole};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "member")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    id: String,
    pub role: MemberRole,
}

impl Model {
    pub fn username(&self) -> JIDNode {
        JIDNode::from_str(&self.id).unwrap()
    }
}

impl ActiveModel {
    pub fn set_username(&mut self, username: &JIDNode) {
        self.id = Set(username.to_string());
    }
}

impl Entity {
    pub fn find_by_username(username: &JIDNode) -> Select<Self> {
        Self::find_by_id(username.to_string())
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
