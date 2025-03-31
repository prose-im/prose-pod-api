// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm::entity::prelude::*;

use crate::{models::Url, pod_config::NetworkAddress};

/// Prose Pod configuration, as stored in the database.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "pod_config")]
pub struct Model {
    #[sea_orm(primary_key)]
    id: i32,
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
    pub hostname: Option<String>,
    pub dashboard_url: Option<Url>,
}

impl Model {
    pub fn pod_address(&self) -> Option<NetworkAddress> {
        NetworkAddress::try_from_or_warn(
            self.hostname.as_ref(),
            self.ipv4.as_ref(),
            self.ipv6.as_ref(),
            "Pod address in database is invalid",
        )
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
