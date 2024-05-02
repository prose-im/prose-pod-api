// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use ::entity::server_config;
use sea_orm::*;

pub struct Mutation;

impl Mutation {
    pub async fn create_server_config(
        db: &DbConn,
        form_data: server_config::ActiveModel,
    ) -> Result<server_config::ActiveModel, DbErr> {
        form_data.save(db).await
    }

    // pub async fn update_server_config_by_id(
    //     db: &DbConn,
    //     id: i32,
    //     form_data: server_config::Model,
    // ) -> Result<server_config::Model, DbErr> {
    //     let server_config: server_config::ActiveModel = ServerConfig::find_by_id(id)
    //         .one(db)
    //         .await?
    //         .ok_or(DbErr::Custom("Cannot find server_config.".to_owned()))
    //         .map(Into::into)?;

    //     server_config::ActiveModel {
    //         id: server_config.id,
    //         title: Set(form_data.title.to_owned()),
    //         text: Set(form_data.text.to_owned()),
    //     }
    //     .update(db)
    //     .await
    // }
}
