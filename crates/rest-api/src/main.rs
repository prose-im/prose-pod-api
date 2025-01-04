// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use prose_pod_api::{custom_router, util::database::db_conn, AppState};
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
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() {
    let app_config = AppConfig::from_default_figment();
    #[cfg(debug_assertions)]
    dbg!(&app_config);

    let db = db_conn(&app_config.databases.main)
        .await
        .expect("Could not connect to the database.");

    let secrets_store = SecretsStore::new(Arc::new(LiveSecretsStore::from_config(&app_config)));
    let http_client = HttpClient::new();
    let prosody_admin_rest = Arc::new(ProsodyAdminRest::from_config(
        &app_config,
        http_client.clone(),
        secrets_store.clone(),
    ));
    let server_ctl = ServerCtl::new(Arc::new(LiveServerCtl::from_config(
        &app_config,
        prosody_admin_rest.clone(),
    )));
    let xmpp_service = XmppServiceInner::new(Arc::new(LiveXmppService::from_config(
        &app_config,
        http_client.clone(),
        prosody_admin_rest.clone(),
        Arc::new(UUIDProvider::new()),
    )));
    let prosody_oauth2 = Arc::new(ProsodyOAuth2::from_config(&app_config, http_client.clone()));
    let auth_service = AuthService::new(Arc::new(LiveAuthService::new(prosody_oauth2.clone())));
    let notifier = Notifier::from_config(&app_config).unwrap_or_else(|e| panic!("{e}"));
    let network_checker = NetworkChecker::new(Arc::new(LiveNetworkChecker::default()));

    {
        let subscriber = FmtSubscriber::builder()
            .with_env_filter(EnvFilter::from_default_env())
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set tracing subscriber.");
    }

    let app_state = AppState::new(
        db,
        app_config,
        server_ctl,
        xmpp_service,
        auth_service,
        notifier,
        secrets_store,
        network_checker,
    );
    let app = custom_router(app_state)
        .await
        .map_err(|err| panic!("{err}"))
        .unwrap();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
