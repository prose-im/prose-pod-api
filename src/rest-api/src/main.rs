// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use axum::Router;
use prose_pod_api::{
    make_router, run_startup_actions,
    util::{database::db_conn, tracing_subscriber_ext, LifecycleManager},
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
use tokio_util::sync::CancellationToken;
use tracing::{info, instrument, trace, warn};

#[tokio::main]
async fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Could not install default crypto provider.");

    let _tracing_guard = tracing_subscriber_ext::init_subscribers()
        .map_err(|err| panic!("Failed to init tracing for OpenTelemetry: {err}"))
        .unwrap();

    let starting = AtomicBool::new(true);
    let mut lifecycle_manager = LifecycleManager::new();

    lifecycle_manager.restart_rx_mut().mark_changed();
    while lifecycle_manager.restart_rx_mut().changed().await.is_ok() {
        let (starting, restarting) = (
            starting.swap(false, Ordering::Relaxed),
            lifecycle_manager.is_restarting(),
        );
        if restarting {
            warn!("Restarting…");
        }
        if starting || restarting {
            {
                let lifecycle_manager = lifecycle_manager.clone();
                tokio::task::spawn(async move {
                    trace!("Starting an instance…");
                    run(&lifecycle_manager).await;
                    trace!("Instance stopped.");
                });
            }

            lifecycle_manager = lifecycle_manager.rotate_instance();
        }
    }
    warn!("Nothing else to do, exiting.");
}

async fn run(lifecycle_manager: &LifecycleManager) {
    let app_config = AppConfig::from_default_figment();
    if app_config.debug.log_config_at_startup {
        dbg!(&app_config);
    }
    let addr = SocketAddr::new(app_config.address, app_config.port);

    let app = startup(app_config, lifecycle_manager).await;
    lifecycle_manager.set_restart_finished();

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let (cancellation_token, stopped) = lifecycle_manager.current_instance();
    info!("Serving the Prose Pod API on {addr}…");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(cancellation_token))
        .await
        .unwrap();

    trace!("API instance stopped. Waiting for next one to start…");
    stopped.wait().await;
}

#[instrument(level = "trace", skip_all)]
async fn startup(app_config: AppConfig, lifecycle_manager: &LifecycleManager) -> Router {
    let app_state = init_dependencies(app_config, lifecycle_manager).await;

    let app = run_startup_actions(make_router(&app_state), app_state)
        .await
        .map_err(|err| panic!("{err}"))
        .unwrap();

    // NOTE: While we could have made `AppState` mutable by wrapping it in a `RwLock`
    //   and replaced all of it, we chose to recreate the `Router` to make sure Axum
    //   doesn’t keep caches which would leak data.
    lifecycle_manager.stop_previous_instance().await;

    app
}

#[instrument(level = "trace", skip_all)]
async fn init_dependencies(
    app_config: AppConfig,
    lifecycle_manager: &LifecycleManager,
) -> AppState {
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
    let email_notifier = Notifier::from_config::<EmailNotifier, _>(&app_config)
        .inspect_err(|err| warn!("Could not create email notifier: {err}"))
        .ok();
    let network_checker = NetworkChecker::new(Arc::new(LiveNetworkChecker::default()));

    AppState::new(
        db,
        app_config,
        server_ctl,
        xmpp_service,
        auth_service,
        email_notifier,
        secrets_store,
        network_checker,
        lifecycle_manager.clone(),
    )
}

/// Source: [`axum/examples/graceful-shutdown/src/main.rs#L55-L77`](https://github.com/tokio-rs/axum/blob/ef0b99b6a01e083101fe2e78e6a9c17e3708bc3c/examples/graceful-shutdown/src/main.rs#L55-L77)
///
/// NOTE: Graceful shutdown will wait for outstanding requests to complete.
///   We have SSE routes so we can't add a timeout like suggested in
///   [`axum/examples/graceful-shutdown/src/main.rs#L40-L42`](https://github.com/tokio-rs/axum/blob/ef0b99b6a01e083101fe2e78e6a9c17e3708bc3c/examples/graceful-shutdown/src/main.rs#L40-L42).
///   We'll find a solution if it ever becomes a problem.
async fn shutdown_signal(cancellation_token: CancellationToken) {
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
        _ = ctrl_c => {
            warn!("Received Ctrl+C.")
        },
        _ = terminate => {
            warn!("Process terminated.")
        },
        _ = cancellation_token.cancelled() => {
            warn!("API run cancelled.")
        },
    }
}
