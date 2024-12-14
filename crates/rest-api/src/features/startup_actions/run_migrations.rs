// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{Build, Rocket};
use sea_orm_rocket::Database as _;
use service::MigratorTrait as _;

use crate::{guards::Db, AppState};

pub async fn run_migrations(rocket: &Rocket<Build>) -> Result<(), String> {
    let conn = &Db::fetch(&rocket).expect("Db not attached").conn;
    let _ = service::Migrator::up(conn, None).await;
    Ok(())
}

pub async fn run_migrations_axum(_app_state: &AppState) -> Result<(), String> {
    todo!()
}
