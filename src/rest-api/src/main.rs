// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{net::SocketAddr, sync::Arc};

use axum::Router;
use prose_pod_api::{
    factory_reset_router, make_router, run_startup_actions,
    util::{
        database::db_conn,
        tracing_subscriber_ext::{self, init_subscribers, TracingReloadHandles},
        LifecycleManager,
    },
    AppState, MinimalAppState,
};
use service::{
    app_config::defaults,
    auth::{AuthService, LiveAuthService},
    licensing::{LicenseService, LiveLicenseService},
    network_checks::{LiveNetworkChecker, NetworkChecker},
    notifications::{notifier::email::EmailNotifier, Notifier},
    pod_version::{LivePodVersionService, PodVersionService, StaticPodVersionService},
    prose_xmpp::UUIDProvider,
    prosody::{ProsodyAdminRest, ProsodyOAuth2},
    secrets::{LiveSecretsStore, SecretsStore},
    xmpp::{LiveServerCtl, LiveXmppService, ServerCtl, XmppServiceInner},
    AppConfig, HttpClient,
};
use tracing::{info, instrument, trace, warn, Subscriber};
use tracing_subscriber::{registry::LookupSpan, EnvFilter, Layer};

#[tokio::main]
async fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Could not install default crypto provider.");

    // NOTE: Can only be called once.
    let (_tracing_guard, tracing_reload_handles) = init_subscribers()
        .map_err(|err| panic!("Failed to init tracing for OpenTelemetry: {err}"))
        .unwrap();

    let mut lifecycle_manager = LifecycleManager::new();
    let secrets_store = SecretsStore::new(Arc::new(LiveSecretsStore::default()));

    {
        let mut lifecycle_manager = lifecycle_manager.clone();
        tokio::task::spawn(async move { lifecycle_manager.listen_for_graceful_shutdown().await });
    }
    {
        let mut lifecycle_manager = lifecycle_manager.clone();
        tokio::task::spawn(async move { lifecycle_manager.listen_for_reload().await });
    }

    let mut starting = true;
    while starting || lifecycle_manager.should_restart().await {
        if starting {
            starting = false;
        } else {
            warn!("Restarting…");
        }

        {
            let minimal_app_state = MinimalAppState {
                lifecycle_manager: lifecycle_manager.clone(),
                secrets_store: secrets_store.clone(),
                static_pod_version_service: PodVersionService::new(Arc::new(
                    StaticPodVersionService,
                )),
            };
            let tracing_reload_handles = tracing_reload_handles.clone();
            tokio::task::spawn(async move {
                trace!("Starting an instance…");
                run(minimal_app_state, &tracing_reload_handles).await;
                trace!("Run finished.");
            });
        }

        lifecycle_manager = lifecycle_manager.rotate_instance();

        if lifecycle_manager.will_restart() {
            trace!("Waiting for next `restart_rx` signal…");
        } else {
            trace!("Not waiting for next `restart_rx` signal.");
        }
    }

    info!("Nothing else to do, exiting.");
}

async fn run(
    minimal_app_state: MinimalAppState,
    tracing_reload_handles: &TracingReloadHandles<
        EnvFilter,
        impl Subscriber + for<'a> LookupSpan<'a>,
        Box<dyn Layer<impl Subscriber + for<'a> LookupSpan<'a>> + Send + Sync>,
        impl Subscriber,
    >,
) {
    let lifecycle_manager = minimal_app_state.lifecycle_manager.clone();

    let (addr, app) = match AppConfig::from_default_figment() {
        Ok(app_config) => {
            let addr = SocketAddr::new(app_config.api.address, app_config.api.port);

            tracing_subscriber_ext::update_tracing_config(&app_config, tracing_reload_handles);

            let app = startup(app_config, minimal_app_state).await;

            (addr, app)
        }
        // Server a subset of routes if the API is broken while restarting.
        Err(_) if lifecycle_manager.is_restarting() => {
            warn!("The Prose Pod API is missing some static configuration. Serving only utility routes.");

            let addr = SocketAddr::new(defaults::api_address(), defaults::api_port());

            let app = factory_reset_router(&minimal_app_state);

            (addr, app)
        }
        // Panic if the API is just starting up.
        Err(err) => {
            // NOTE: `panic`s are unwound therefore we need to exit manually.
            tracing::error!("Startup error: {err:#}");
            std::process::exit(1);
        }
    };

    lifecycle_manager.stop_previous_instance().await;
    lifecycle_manager.set_restart_finished();

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let (cancellation_token, stopped) = lifecycle_manager.current_instance();
    info!("Serving the Prose Pod API on {addr}…");
    axum::serve(listener, app)
        .with_graceful_shutdown(async move { cancellation_token.cancelled().await })
        .await
        .unwrap();

    trace!("API instance stopped.");
    if lifecycle_manager.is_restarting() {
        trace!("Waiting for next instance to start…");
        stopped.wait().await;
    }
}

#[instrument(level = "trace", skip_all)]
async fn startup(app_config: AppConfig, minimal_app_state: MinimalAppState) -> Router {
    let app_state = init_dependencies(app_config, minimal_app_state).await;

    // NOTE: While we could have made `AppState` mutable by wrapping it in a `RwLock`
    //   and replaced all of it, we chose to recreate the `Router` to make sure Axum
    //   doesn’t keep caches which would leak data.
    match run_startup_actions(make_router(&app_state), app_state).await {
        Ok(router) => router,
        Err(err) => {
            // NOTE: `panic`s are unwound therefore we need to exit manually.
            tracing::error!("Startup error: {err:#}");
            std::process::exit(1);
        }
    }
}

#[instrument(level = "trace", skip_all)]
async fn init_dependencies(app_config: AppConfig, base: MinimalAppState) -> AppState {
    let db = db_conn(&app_config.api.databases.main)
        .await
        .expect("Could not connect to the database.");

    let license_service_impl = match LiveLicenseService::from_config(&app_config) {
        Ok(service) => service,
        Err(err) => {
            // NOTE: `panic`s are unwound therefore we need to exit manually.
            tracing::error!("Startup error: {err:#}");
            std::process::exit(1);
        }
    };

    // FIXME: Pass a dynamic `AppConfig` to dependencies?

    base.secrets_store.load_config(&app_config);
    let http_client = HttpClient::new();
    let prosody_admin_rest = Arc::new(ProsodyAdminRest::from_config(
        &app_config,
        http_client.clone(),
        base.secrets_store.clone(),
    ));
    let server_ctl = ServerCtl::new(Arc::new(LiveServerCtl::from_config(
        &app_config,
        prosody_admin_rest.clone(),
        db.clone(),
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
    let license_service = LicenseService::new(Arc::new(license_service_impl));
    let pod_version_service = PodVersionService::new(Arc::new(LivePodVersionService::from_config(
        &app_config,
        http_client.clone(),
    )));

    AppState::new(
        base,
        db,
        Arc::new(app_config),
        server_ctl,
        xmpp_service,
        auth_service,
        email_notifier,
        network_checker,
        license_service,
        pod_version_service,
    )
}
