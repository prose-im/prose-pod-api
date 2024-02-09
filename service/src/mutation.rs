// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use ::entity::settings;
use sea_orm::*;

pub struct Mutation;

impl Mutation {
    pub async fn create_settings(
        db: &DbConn,
        form_data: settings::ActiveModel,
    ) -> Result<settings::ActiveModel, DbErr> {
        form_data.save(db).await
    }

    // pub async fn update_settings_by_id(
    //     db: &DbConn,
    //     id: i32,
    //     form_data: settings::Model,
    // ) -> Result<settings::Model, DbErr> {
    //     let settings: settings::ActiveModel = Settings::find_by_id(id)
    //         .one(db)
    //         .await?
    //         .ok_or(DbErr::Custom("Cannot find settings.".to_owned()))
    //         .map(Into::into)?;

    //     settings::ActiveModel {
    //         id: settings.id,
    //         title: Set(form_data.title.to_owned()),
    //         text: Set(form_data.text.to_owned()),
    //     }
    //     .update(db)
    //     .await
    // }
}
