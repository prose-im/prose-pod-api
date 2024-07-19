// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    config::AppConfig,
    controllers::workspace_controller::WorkspaceController,
    model::ServerConfig,
    services::{secrets_store::SecretsStore, xmpp_service::XmppServiceInner},
};

use super::prelude::*;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for WorkspaceController<'r> {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);
        let xmpp_service = &try_outcome!(request_state!(req, XmppServiceInner));
        let app_config = &try_outcome!(request_state!(req, AppConfig));
        let server_config = try_outcome!(ServerConfig::from_request(req).await);
        let secrets_store = &try_outcome!(request_state!(req, SecretsStore));

        match WorkspaceController::new(db, xmpp_service, app_config, &server_config, secrets_store)
        {
            Ok(controller) => Outcome::Success(controller),
            Err(err) => Error::from(err).into(),
        }
    }
}
