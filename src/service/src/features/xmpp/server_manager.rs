// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{
    auth::AuthToken, prose_pod_server_service::ProsePodServerService, sea_orm::DatabaseConnection,
    server_config,
};

pub async fn reset_server_config(
    db: &DatabaseConnection,
    server: &ProsePodServerService,
    auth: &AuthToken,
) -> anyhow::Result<()> {
    server_config::reset(db).await?;

    // Write the bootstrap configuration.
    server.reset_config(auth).await?;

    // Apply the bootstrap configuration.
    server.reload(Some(auth)).await?;

    Ok(())
}
