// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    cli::run_cli(service::Migrator).await;
}
