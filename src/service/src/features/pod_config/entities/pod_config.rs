// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm::entity::prelude::*;

use crate::pod_config::{NetworkAddress, NetworkAddressError};

/// Prose Pod configuration, as stored in the database.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "pod_config")]
pub struct Model {
    #[sea_orm(primary_key)]
    id: i32,
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
    pub hostname: Option<String>,
    pub dashboard_ipv4: Option<String>,
    pub dashboard_ipv6: Option<String>,
    pub dashboard_hostname: Option<String>,
}

impl Model {
    pub fn pod_address(&self) -> Result<NetworkAddress, NetworkAddressError> {
        NetworkAddress::try_from(
            self.hostname.as_ref(),
            self.ipv4.as_ref(),
            self.ipv6.as_ref(),
        )
    }
    pub fn dashboard_address(&self) -> Result<NetworkAddress, NetworkAddressError> {
        NetworkAddress::try_from(
            self.dashboard_hostname.as_ref(),
            self.dashboard_ipv4.as_ref(),
            self.dashboard_ipv6.as_ref(),
        )
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
