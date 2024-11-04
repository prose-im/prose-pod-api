// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{response::status, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use service::{
    config::AppConfig,
    controllers::init_controller::InitController,
    model::{JidDomain, ServerConfig},
    repositories::ServerConfigCreateForm,
    services::{auth_service::AuthService, secrets_store::SecretsStore, server_ctl::ServerCtl},
};

use crate::{guards::LazyGuard, responders::Created};

#[derive(Serialize, Deserialize)]
pub struct InitServerConfigRequest {
    /// XMPP server domain (e.g. `crisp.chat`).
    /// This is what will appear in JIDs (e.g. `valerian@`**`crisp.chat`**).
    pub domain: JidDomain,
}

#[put("/v1/server/config", format = "json", data = "<req>")]
pub async fn init_server_config_route<'r>(
    init_controller: LazyGuard<InitController<'r>>,
    server_ctl: &State<ServerCtl>,
    app_config: &State<AppConfig>,
    auth_service: &State<AuthService>,
    secrets_store: &State<SecretsStore>,
    req: Json<InitServerConfigRequest>,
) -> Created<ServerConfig> {
    let init_controller = init_controller.inner?;
    let form = req.into_inner();

    let server_config = init_controller
        .init_server_config(server_ctl, app_config, auth_service, secrets_store, form)
        .await?;

    let resource_uri = uri!(crate::features::server_config::get_server_config_route).to_string();
    Ok(status::Created::new(resource_uri).body(server_config.into()))
}

// BOILERPLATE

impl Into<ServerConfigCreateForm> for InitServerConfigRequest {
    fn into(self) -> ServerConfigCreateForm {
        ServerConfigCreateForm {
            domain: self.domain.to_owned(),
        }
    }
}
