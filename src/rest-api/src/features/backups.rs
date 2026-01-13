// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::middleware::from_extractor_with_state;
use axum::routing::MethodRouter;
use service::auth::IsAdmin;

use crate::AppState;

pub use self::routes::*;

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route(
            "/v1/backups",
            MethodRouter::new()
                .get(list_backups_route)
                .post(make_backup_route)
                .delete(delete_backup_route),
        )
        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone()))
        .with_state(app_state)
}

mod routes {
    use axum::{extract::State, response::NoContent, Json};
    use service::backups::backup_service::BackupMetadata;

    use crate::AppState;

    pub async fn list_backups_route(
        State(ref app_state): State<AppState>,
    ) -> Result<Json<Vec<BackupMetadata>>, crate::error::Error> {
        let backups = app_state.backup_service.list().await?;
        todo!()
    }

    pub(super) async fn get_backup_route(
        State(ref app_state): State<AppState>,
    ) -> Result<Json<Option<BackupMetadata>>, crate::error::Error> {
        todo!()
    }

    pub(super) async fn make_backup_route(
        State(ref app_state): State<AppState>,
    ) -> Result<Json<BackupMetadata>, crate::error::Error> {
        todo!()
    }

    pub(super) async fn delete_backup_route(
        State(ref app_state): State<AppState>,
    ) -> Result<NoContent, crate::error::Error> {
        todo!()
    }
}
