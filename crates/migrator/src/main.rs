// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[tokio::main]
async fn main() {
    sea_orm_migration::cli::run_cli(service::Migrator).await;
}
