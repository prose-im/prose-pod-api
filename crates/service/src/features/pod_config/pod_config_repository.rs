// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::{Ipv4Addr, Ipv6Addr};

use hickory_proto::rr::Name as DomainName;
use sea_orm::{prelude::*, QueryOrder as _, Set, Unchanged};

use super::entities::pod_config::{self, ActiveModel, Column, Entity};

pub enum PodConfigRepository {}

impl PodConfigRepository {
    pub async fn create(
        db: &impl ConnectionTrait,
        form: impl Into<PodConfigCreateForm>,
    ) -> Result<pod_config::Model, DbErr> {
        form.into().into_active_model().insert(db).await
    }

    pub async fn get(db: &impl ConnectionTrait) -> Result<Option<pod_config::Model>, DbErr> {
        Entity::find().order_by_asc(Column::Id).one(db).await
    }

    pub async fn set(
        db: &impl ConnectionTrait,
        form: impl Into<PodConfigCreateForm>,
    ) -> Result<pod_config::Model, DbErr> {
        let mut active_model = form.into().into_active_model();
        active_model.id = Unchanged(1);
        active_model.update(db).await
    }
}

#[derive(Debug, Clone, Default)]
pub struct PodConfigCreateForm {
    pub ipv4: Option<Ipv4Addr>,
    pub ipv6: Option<Ipv6Addr>,
    pub hostname: Option<DomainName>,
}

impl PodConfigCreateForm {
    fn into_active_model(self) -> ActiveModel {
        ActiveModel {
            ipv4: Set(self.ipv4.map(|v| v.to_string())),
            ipv6: Set(self.ipv6.map(|v| v.to_string())),
            hostname: Set(self.hostname.map(|v| v.to_string())),
            ..Default::default()
        }
    }
}
