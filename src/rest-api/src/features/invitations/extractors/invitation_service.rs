// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{notifications::NotificationService, workspace::WorkspaceService};

use crate::extractors::prelude::*;

impl FromRequestParts<AppState> for service::invitations::InvitationService {
    type Rejection = Error;

    #[tracing::instrument(
        name = "req::extract::invitation_service",
        level = "trace",
        skip_all,
        err
    )]
    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let notification_service = NotificationService::from_request_parts(parts, state).await?;
        let workspace_service = WorkspaceService::from_request_parts(parts, state).await?;

        Ok(Self {
            db: state.db.clone(),
            notification_service,
            invitation_repository: state.invitation_repository.clone(),
            workspace_service,
            auth_service: state.auth_service.clone(),
            xmpp_service: state.xmpp_service.clone(),
            user_repository: state.user_repository.clone(),
            app_config: state.app_config.clone(),
            invitation_application_service: state.invitation_application_service.clone(),
            licensing_service: state.licensing_service.clone(),
        })
    }
}
