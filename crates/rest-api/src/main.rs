// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_use]
extern crate rocket;

use std::sync::Arc;

use prose_pod_api::{custom_rocket, guards::Db};
use rocket::fairing::AdHoc;
use sea_orm_rocket::Database as _;
use service::{
    auth::{AuthService, LiveAuthService},
    network_checks::{LiveNetworkChecker, NetworkChecker},
    notifications::dependencies::Notifier,
    prose_xmpp::UUIDProvider,
    prosody::{ProsodyAdminRest, ProsodyOAuth2},
    secrets::{LiveSecretsStore, SecretsStore},
    xmpp::{LiveServerCtl, LiveXmppService, ServerCtl, XmppServiceInner},
    AppConfig, HttpClient,
};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[launch]
fn rocket() -> _ {
    let config = AppConfig::figment();
    #[cfg(debug_assertions)]
    dbg!(&config);
    let secrets_store = SecretsStore::new(Arc::new(LiveSecretsStore::from_config(&config)));
    let http_client = HttpClient::new();
    let prosody_admin_rest = Arc::new(ProsodyAdminRest::from_config(
        &config,
        http_client.clone(),
        secrets_store.clone(),
    ));
    let server_ctl = ServerCtl::new(Arc::new(LiveServerCtl::from_config(
        &config,
        prosody_admin_rest.clone(),
    )));
    let xmpp_service = XmppServiceInner::new(Arc::new(LiveXmppService::from_config(
        &config,
        http_client.clone(),
        prosody_admin_rest.clone(),
        Arc::new(UUIDProvider::new()),
    )));
    let prosody_oauth2 = Arc::new(ProsodyOAuth2::from_config(&config, http_client.clone()));
    let auth_service = AuthService::new(Arc::new(LiveAuthService::new(prosody_oauth2.clone())));
    let notifier = Notifier::from_config(&config).unwrap_or_else(|e| panic!("{e}"));
    let network_checker = NetworkChecker::new(Arc::new(LiveNetworkChecker::default()));

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
        secrets_store,
        network_checker,
    )
}
