// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_use]
extern crate rocket;

use prose_pod_api::custom_rocket;
use service::config::Config;
use service::dependencies::Notifier;
use service::prose_xmpp::UUIDProvider;
use service::prosody::{ProsodyAdminRest, ProsodyOAuth2};
use service::xmpp::LiveXmppService;
use service::{AuthService, HttpClient, JWTService, LiveAuthService, ServerCtl, XmppServiceInner};

#[launch]
fn rocket() -> _ {
    env_logger::init();

    let config = Config::figment();
    #[cfg(debug_assertions)]
    dbg!(&config);
    let jwt_service = match JWTService::from_env() {
        Ok(service) => service,
        Err(err) => panic!("{err}"),
    };
    let http_client = HttpClient::new();
    let prosody_admin_rest = ProsodyAdminRest::from_config(&config, http_client.clone());
    let server_ctl = ServerCtl::new(Box::new(prosody_admin_rest.clone()));
    let xmpp_service = XmppServiceInner::new(Box::new(LiveXmppService::from_config(
        &config,
        http_client.clone(),
        Box::new(prosody_admin_rest),
        Box::new(UUIDProvider::new()),
    )));
    let prosody_oauth2 = ProsodyOAuth2::from_config(&config, http_client.clone());
    let auth_service =
        AuthService::new(Box::new(LiveAuthService::new(jwt_service, prosody_oauth2)));
    let notifier = Notifier::from_config(&config).unwrap_or_else(|e| panic!("{e}"));

    custom_rocket(rocket::build(), config, server_ctl, xmpp_service)
        .manage(auth_service)
        .manage(notifier)
}
