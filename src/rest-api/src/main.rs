// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{net::SocketAddr, sync::Arc};

use prose_pod_api::{
    make_router, run_startup_actions,
    util::{database::db_conn, tracing_subscriber_ext},
    AppState,
};
use service::{
    auth::{AuthService, LiveAuthService},
    network_checks::{LiveNetworkChecker, NetworkChecker},
    notifications::{notifier::email::EmailNotifier, Notifier},
    prose_xmpp::UUIDProvider,
    prosody::{ProsodyAdminRest, ProsodyOAuth2},
    secrets::{LiveSecretsStore, SecretsStore},
    xmpp::{LiveServerCtl, LiveXmppService, ServerCtl, XmppServiceInner},
    AppConfig, HttpClient,
};
use tracing::{info, trace_span};

#[tokio::main]
async fn main() {
    let app_config = AppConfig::from_default_figment();
    if app_config.debug.log_config_at_startup {
        dbg!(&app_config);
    }

    let _tracing_guard = tracing_subscriber_ext::init_subscribers()
        .map_err(|err| panic!("Failed to init tracing for OpenTelemetry: {err}"))
        .unwrap();

    let startup_span = trace_span!("startup").entered();

    let init_dependencies_span = trace_span!("init_dependencies").entered();

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Could not install default crypto provider.");

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
    let email_notifier =
        Notifier::from_config::<EmailNotifier, _>(&app_config).unwrap_or_else(|e| panic!("{e}"));
    let network_checker = NetworkChecker::new(Arc::new(LiveNetworkChecker::default()));

    drop(init_dependencies_span);

    let addr = SocketAddr::new(app_config.address, app_config.port);

    let app_state = AppState::new(
        db,
        app_config,
        server_ctl,
        xmpp_service,
        auth_service,
        email_notifier,
        secrets_store,
        network_checker,
    );
    let app = run_startup_actions(make_router(&app_state), app_state)
        .await
        .map_err(|err| panic!("{err}"))
        .unwrap();

    drop(startup_span);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Serving the Prose Pod API on {addr}…");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

/// Source: [`axum/examples/graceful-shutdown/src/main.rs#L55-L77`](https://github.com/tokio-rs/axum/blob/ef0b99b6a01e083101fe2e78e6a9c17e3708bc3c/examples/graceful-shutdown/src/main.rs#L55-L77)
///
/// NOTE: Graceful shutdown will wait for outstanding requests to complete.
///   We have SSE routes so we can't add a timeout like suggested in
///   [`axum/examples/graceful-shutdown/src/main.rs#L40-L42`](https://github.com/tokio-rs/axum/blob/ef0b99b6a01e083101fe2e78e6a9c17e3708bc3c/examples/graceful-shutdown/src/main.rs#L40-L42).
///   We'll find a solution if it ever becomes a problem.
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
