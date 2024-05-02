// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use ::entity::model::{MemberRole, JID};
use ::entity::prelude::*;
use ::entity::server_config;
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn server_config(db: &DbConn) -> Result<Option<server_config::Model>, DbErr> {
        ServerConfig::find()
            .order_by_asc(server_config::Column::Id)
            .one(db)
            .await
    }

    pub async fn is_admin(db: &DbConn, jid: &JID) -> Result<bool, DbErr> {
        // TODO: Use a [Custom Struct](https://www.sea-ql.org/SeaORM/docs/advanced-query/custom-select/#custom-struct) to query only the `role` field.
        let member = Member::find_by_id(jid.to_string()).one(db).await?;

        // If the member is not found, do not send an error but rather send `false` as it is not an admin anyway.
        let Some(member) = member else {
            return Ok(false);
        };

        Ok(member.role == MemberRole::Admin)
    }
}
