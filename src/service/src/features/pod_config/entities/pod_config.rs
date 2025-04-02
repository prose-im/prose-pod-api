// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::{Ipv4Addr, Ipv6Addr};

use sea_orm::entity::prelude::*;

use crate::{
    models::{sea_orm::SeaOrmAsString, Url},
    pod_config::{NetworkAddress, PodAddress},
};

/// Prose Pod configuration, as stored in the database.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "pod_config")]
pub struct Model {
    #[sea_orm(primary_key)]
    id: i32,
    ipv4: Option<SeaOrmAsString<Ipv4Addr>>,
    ipv6: Option<SeaOrmAsString<Ipv6Addr>>,
    hostname: Option<String>,
    dashboard_url: Option<Url>,
}

impl Model {
    pub fn address(&self) -> Option<PodAddress> {
        match (&self.ipv4, &self.ipv6, &self.hostname) {
            (None, None, None) => None,
            (ipv4, ipv6, hostname) => Some(PodAddress {
                ipv4: ipv4.as_deref().cloned(),
                ipv6: ipv6.as_deref().cloned(),
                hostname: hostname.clone(),
            }),
        }
    }
    pub fn network_address(&self) -> Option<NetworkAddress> {
        NetworkAddress::try_from_or_warn(
            self.hostname.as_ref(),
            self.ipv4.as_deref().cloned(),
            self.ipv6.as_deref().cloned(),
            "Pod address in database is invalid",
        )
    }
    pub fn dashboard_url(&self) -> Option<Url> {
        self.dashboard_url.clone()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
