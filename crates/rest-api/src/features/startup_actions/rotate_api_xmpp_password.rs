// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{Build, Rocket};
use service::{
    secrets::SecretsStore,
    xmpp::{ServerCtl, ServerManager},
    AppConfig,
};

pub async fn rotate_api_xmpp_password(rocket: &Rocket<Build>) -> Result<(), String> {
    debug!("Rotating Prose Pod API's XMPP password…");

    let server_ctl: &ServerCtl = rocket.state().unwrap();
    let app_config: &AppConfig = rocket.state().unwrap();
    let secrets_store: &SecretsStore = rocket.state().unwrap();

    if let Err(err) =
        ServerManager::rotate_api_xmpp_password(server_ctl, app_config, secrets_store).await
    {
        return Err(format!("Could not rotate the API XMPP password: {err}"));
    }

    Ok(())
}
