// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_use]
extern crate rocket;

use prose_pod_api::{custom_rocket, guards::Db};
use rocket::fairing::AdHoc;
use sea_orm_rocket::Database as _;
use service::{
    config::Config,
    dependencies::Notifier,
    prose_xmpp::UUIDProvider,
    prosody::{ProsodyAdminRest, ProsodyOAuth2},
    services::{
        auth_service::{AuthService, LiveAuthService},
        jwt_service::JWTService,
        live_secrets_store::LiveSecretsStore,
        secrets_store::SecretsStore,
        server_ctl::ServerCtl,
        xmpp_service::{LiveXmppService, XmppServiceInner},
    },
    HttpClient,
};
use std::sync::Arc;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[launch]
fn rocket() -> _ {
    let config = Config::figment();
    #[cfg(debug_assertions)]
    dbg!(&config);
    let jwt_service = match JWTService::from_env() {
        Ok(service) => service,
        Err(err) => panic!("{err}"),
    };
    let secrets_store = SecretsStore::new(Arc::new(LiveSecretsStore::from_config(&config)));
    let http_client = HttpClient::new();
    let prosody_admin_rest =
        ProsodyAdminRest::from_config(&config, http_client.clone(), secrets_store.clone());
    let server_ctl = ServerCtl::new(Arc::new(prosody_admin_rest.clone()));
    let xmpp_service = XmppServiceInner::new(Arc::new(LiveXmppService::from_config(
        &config,
        http_client.clone(),
        Arc::new(prosody_admin_rest),
        Arc::new(UUIDProvider::new()),
    )));
    let prosody_oauth2 = ProsodyOAuth2::from_config(&config, http_client.clone());
    let auth_service = AuthService::new(Arc::new(LiveAuthService::new(
        jwt_service.clone(),
        prosody_oauth2,
    )));
    let notifier = Notifier::from_config(&config).unwrap_or_else(|e| panic!("{e}"));

    let rocket = rocket::build().attach(Db::init()).attach(AdHoc::on_ignite(
        "Tracing subsciber",
        |rocket| async move {
            let subscriber = FmtSubscriber::builder()
                .with_env_filter(EnvFilter::from_default_env())
                .finish();
            tracing::subscriber::set_global_default(subscriber)
                .expect("Failed to set tracing subscriber.");
            rocket
        },
    ));

    custom_rocket(
        rocket,
        config,
        server_ctl,
        xmpp_service,
        auth_service,
        notifier,
        jwt_service,
        secrets_store,
    )
}
