// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::{Ipv4Addr, Ipv6Addr};

use sea_orm::{prelude::*, ActiveModelTrait, QueryOrder as _, QuerySelect, Set, Unchanged};
use tracing::instrument;

use crate::models::{sea_orm::SeaOrmAsString, Url};

use super::{
    entities::pod_config::{self, ActiveModel, Column, Entity},
    PodAddress,
};

pub enum PodConfigRepository {}

impl PodConfigRepository {
    #[instrument(
        name = "db::pod_config::is_initialized",
        level = "trace",
        skip_all,
        err
    )]
    pub async fn is_initialized(db: &impl ConnectionTrait) -> Result<bool, DbErr> {
        Ok(Entity::find().count(db).await? > 0)
    }

    #[instrument(name = "db::pod_config::create", level = "trace", skip_all, err)]
    pub async fn create(
        db: &impl ConnectionTrait,
        form: impl Into<PodConfigCreateForm>,
    ) -> Result<pod_config::Model, DbErr> {
        form.into().into_active_model().insert(db).await
    }

    #[instrument(name = "db::pod_config::get", level = "trace", skip_all, err)]
    pub async fn get(db: &impl ConnectionTrait) -> Result<Option<pod_config::Model>, DbErr> {
        Entity::find().order_by_asc(Column::Id).one(db).await
    }

    #[instrument(
        name = "db::pod_config::get_dashboard_url",
        level = "trace",
        skip_all,
        err
    )]
    pub async fn get_dashboard_url(db: &impl ConnectionTrait) -> Result<Option<Url>, DbErr> {
        let res = Entity::find()
            .order_by_asc(Column::Id)
            .select_only()
            .columns([Column::DashboardUrl])
            .into_tuple::<Option<Url>>()
            .one(db)
            .await?
            .flatten();
        Ok(res)
    }

    #[instrument(
        name = "db::pod_config::get_pod_address",
        level = "trace",
        skip_all,
        err
    )]
    pub async fn get_pod_address(db: &impl ConnectionTrait) -> Result<Option<PodAddress>, DbErr> {
        let res = Entity::find()
            .order_by_asc(Column::Id)
            .select_only()
            .columns([
                Column::Ipv4,
                Column::Ipv6,
                Column::Hostname,
            ])
            .into_tuple::<(
                Option<SeaOrmAsString<Ipv4Addr>>,
                Option<SeaOrmAsString<Ipv6Addr>>,
                Option<String>,
            )>()
            .one(db)
            .await?
            .and_then(|(ipv4, ipv6, hostname)| {
                PodAddress::try_from(ipv4.as_deref().cloned(), ipv6.as_deref().cloned(), hostname)
            });
        Ok(res)
    }

    #[instrument(name = "db::pod_config::set", level = "trace", skip_all, err)]
    pub async fn set(
        db: &impl ConnectionTrait,
        form: PodConfigUpdateForm,
    ) -> Result<pod_config::Model, DbErr> {
        let mut active_model = form.into_active_model();

        let is_initialized = Self::is_initialized(db).await?;
        tracing::Span::current().record("is_initialized", is_initialized);

        if is_initialized {
            active_model.id = Unchanged(1);
            active_model.update(db).await
        } else {
            active_model.insert(db).await
        }
    }
}

#[derive(Debug, Default)]
pub struct PodAddressUpdateForm {
    pub ipv4: Option<Option<Ipv4Addr>>,
    pub ipv6: Option<Option<Ipv6Addr>>,
    pub hostname: Option<Option<String>>,
}

#[derive(Debug, Default)]
pub struct PodConfigUpdateForm {
    pub address: Option<PodAddressUpdateForm>,
    pub dashboard_url: Option<Option<Url>>,
}

impl PodConfigUpdateForm {
    fn into_active_model(self) -> ActiveModel {
        let mut active = <ActiveModel as ActiveModelTrait>::default();
        if let Some(address) = self.address {
            if let Some(ipv4) = address.ipv4 {
                active.ipv4 = Set(ipv4.map(SeaOrmAsString));
            }
            if let Some(ipv6) = address.ipv6 {
                active.ipv6 = Set(ipv6.map(SeaOrmAsString));
            }
            if let Some(hostname) = address.hostname {
                active.hostname = Set(hostname);
            }
        };
        if let Some(url) = self.dashboard_url {
            active.dashboard_url = Set(url);
        };
        active
    }
}

#[derive(Debug, Default)]
pub struct NetworkAddressCreateForm {
    pub ipv4: Option<Ipv4Addr>,
    pub ipv6: Option<Ipv6Addr>,
    pub hostname: Option<String>,
}

#[derive(Debug, Default)]
pub struct PodConfigCreateForm {
    pub address: NetworkAddressCreateForm,
    pub dashboard_url: Option<Url>,
}

impl PodConfigCreateForm {
    fn into_active_model(self) -> ActiveModel {
        let mut active = <ActiveModel as ActiveModelTrait>::default();
        active.ipv4 = Set(self.address.ipv4.map(SeaOrmAsString));
        active.ipv6 = Set(self.address.ipv6.map(SeaOrmAsString));
        active.hostname = Set(self.address.hostname);
        active.dashboard_url = Set(self.dashboard_url);
        active
    }
}
