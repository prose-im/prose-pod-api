// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{net::SocketAddr, sync::Arc};

use arc_swap::ArcSwap;
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
    app_config::{pub_defaults as defaults, LogConfig},
    auth::{auth_service::OAuth2ClientState, AuthService, LiveAuthService},
    factory_reset::FactoryResetService,
    identity_provider::{IdentityProvider, LiveIdentityProvider},
    invitations::{
        invitation_service::{InvitationApplicationService, LiveInvitationApplicationService},
        InvitationRepository, LiveInvitationRepository,
    },
    licensing::{LicensingService, LiveLicensingService},
    members::{
        LiveUserApplicationService, LiveUserRepository, MemberService, UserApplicationService,
        UserRepository,
    },
    network_checks::{LiveNetworkChecker, NetworkChecker},
    notifications::{notifier::email::EmailNotifier, Notifier},
    pod_version::{LivePodVersionService, PodVersionService, StaticPodVersionService},
    prose_pod_server_api::ProsePodServerApi,
    prose_pod_server_service::{LiveProsePodServerService, ProsePodServerService},
    prose_xmpp::UUIDProvider,
    prosody::{ProsodyAdminRest, ProsodyInvitesRegisterApi},
    prosody_http::{
        admin_api::ProsodyAdminApi,
        oauth2::{self, ProsodyOAuth2},
        ProsodyHttpConfig,
    },
    workspace::{LiveWorkspaceService, WorkspaceService},
    xmpp::{LiveXmppService, XmppService},
    AppConfig, HttpClient,
};
use tracing::{info, instrument, trace, warn, Instrument as _, Subscriber};
use tracing_subscriber::{registry::LookupSpan, EnvFilter, Layer};

#[tokio::main]
async fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Could not install default crypto provider.");

    let todo = "Try to run make-demo-scenarios";
    let todo = "Update scenarios";

    // NOTE: Can only be called once.
    let (_tracing_guard, tracing_reload_handles) = {
        let figment = AppConfig::figment();
        let ref log_config = match figment.extract_inner::<LogConfig>("log") {
            Ok(config) => config,
            Err(err) if err.missing() => {
                unreachable!("Defaults should be provided by `AppConfig::figment`")
            }
            Err(err) => panic!("Invalid `log` config: {err:?}"),
        };
        init_subscribers(log_config)
            .map_err(|err| panic!("Failed to init tracing for OpenTelemetry: {err}"))
            .unwrap()
    };

    let mut lifecycle_manager = LifecycleManager::new();

    {
        let mut lifecycle_manager = lifecycle_manager.clone();
        tokio::spawn(
            async move { lifecycle_manager.listen_for_graceful_shutdown().await }.in_current_span(),
        );
    }
    {
        let mut lifecycle_manager = lifecycle_manager.clone();
        tokio::spawn(async move { lifecycle_manager.listen_for_reload().await }.in_current_span());
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
                static_pod_version_service: PodVersionService::new(Arc::new(
                    StaticPodVersionService,
                )),
            };
            let tracing_reload_handles = tracing_reload_handles.clone();
            tokio::spawn(
                async move {
                    trace!("Starting an instance…");
                    run(minimal_app_state, &tracing_reload_handles).await;
                    trace!("Run finished.");
                }
                .in_current_span(),
            );
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

            let addr = SocketAddr::new(defaults::API_ADDRESS, defaults::API_PORT);

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
        .with_graceful_shutdown(cancellation_token.cancelled_owned())
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
    let db = db_conn(
        &app_config.api.databases.main_read,
        &app_config.api.databases.main_write,
    )
    .await
    .expect("Could not connect to the database.");

    let licensing_service_impl = match LiveLicensingService::from_config(&app_config) {
        Ok(service) => service,
        Err(err) => {
            // NOTE: `panic`s are unwound therefore we need to exit manually.
            tracing::error!("Startup error: {err:#}");
            std::process::exit(1);
        }
    };

    // FIXME: Pass a dynamic `AppConfig` to dependencies?

    let http_client = HttpClient::new();

    let prosody_admin_rest = Arc::new(ProsodyAdminRest::from_config(
        &app_config,
        http_client.clone(),
    ));
    let prosody_http_config = Arc::new(ProsodyHttpConfig {
        url: app_config.server.http_url(),
    });
    let prosody_http_admin_api = Arc::new(ProsodyAdminApi::new(Arc::clone(&prosody_http_config)));
    let prosody_oauth2 = Arc::new(ProsodyOAuth2::new(Arc::clone(&prosody_http_config)));
    let server_api = ProsePodServerApi::from_config(&app_config, http_client.clone());
    let prosody_invites_register_api =
        ProsodyInvitesRegisterApi::from_config(&app_config, http_client.clone());

    let api_version = base.static_pod_version_service.get_api_version();

    let oauth2_client_config = oauth2::ClientConfig {
        client_name: "Prose Pod API".to_owned(),
        client_uri: "https://prose-pod-api:8080".to_owned(),
        redirect_uris: vec!["https://prose-pod-api:8080/redirect".to_owned()],
        grant_types: vec![
            "authorization_code".to_owned(),
            "refresh_token".to_owned(),
            "password".to_owned(),
        ],
        software_version: api_version.commit_long,
        ..Default::default()
    };
    let auth_service = AuthService {
        implem: Arc::new(LiveAuthService {
            oauth2: prosody_oauth2.clone(),
            oauth2_client: ArcSwap::from_pointee(OAuth2ClientState::Unregistered(oauth2_client_config)),
            server_api: server_api.clone(),
            admin_api: prosody_http_admin_api.clone(),
            invites_register_api: prosody_invites_register_api.clone(),
            password_reset_token_ttl: app_config.auth.password_reset_token_ttl.to_std()
                .expect("`app_config.auth.password_reset_token_ttl` contains years or months. Not supported.")
                .try_into()
                .expect("`app_config.auth.password_reset_token_ttl` out of range."),
            min_password_length: app_config.auth.min_password_length,
        }),
    };

    let live_xmpp_service = Arc::new(LiveXmppService::from_config(
        &app_config,
        http_client.clone(),
        prosody_http_admin_api.clone(),
        Arc::new(UUIDProvider::new()),
    ));

    let user_repository = UserRepository {
        implem: Arc::new(LiveUserRepository {
            server_api: server_api.clone(),
            admin_rest: prosody_admin_rest.clone(),
            admin_api: prosody_http_admin_api.clone(),
            auth_service: auth_service.clone(),
            xmpp_service: Arc::clone(&live_xmpp_service),
            workspace_jid: app_config.workspace_jid(),
        }),
    };
    let invitation_repository = InvitationRepository {
        implem: Arc::new(LiveInvitationRepository {
            server_api: server_api.clone(),
            admin_api: prosody_http_admin_api.clone(),
            invites_register_api: prosody_invites_register_api.clone(),
        }),
    };

    let user_application_service = UserApplicationService {
        implem: Arc::new(LiveUserApplicationService {
            server_api: server_api.clone(),
        }),
    };
    let invitation_application_service = InvitationApplicationService {
        implem: Arc::new(LiveInvitationApplicationService {
            invites_register_api: prosody_invites_register_api.clone(),
        }),
    };

    let xmpp_service = XmppService {
        implem: live_xmpp_service,
    };
    let email_notifier = Notifier::from_config::<EmailNotifier, _>(&app_config)
        .inspect_err(|err| warn!("Could not create email notifier: {err}"))
        .ok();
    let network_checker = NetworkChecker::new(Arc::new(LiveNetworkChecker::from_config(
        &app_config.api.network_checks,
    )));
    let licensing_service = LicensingService::new(Arc::new(licensing_service_impl));
    let pod_version_service = PodVersionService::new(Arc::new(LivePodVersionService::from_config(
        &app_config,
        http_client.clone(),
    )));
    let factory_reset_service = FactoryResetService::default();
    let member_service = MemberService::new(
        user_repository.clone(),
        user_application_service.clone(),
        app_config.server_domain().to_owned(),
        xmpp_service.clone(),
        auth_service.clone(),
        None,
        &app_config.api.member_enriching,
    );
    let identity_provider = IdentityProvider::new(Arc::new(LiveIdentityProvider {
        db: db.clone(),
        member_service: member_service.clone(),
        admin_api: prosody_http_admin_api.clone(),
    }));
    let prose_pod_server_service = ProsePodServerService(Arc::new(LiveProsePodServerService {
        config_file_path: app_config.prosody_ext.config_file_path.clone(),
        server_api: server_api.clone(),
        admin_rest: prosody_admin_rest.clone(),
        admin_api: prosody_http_admin_api.clone(),
        auth_service: auth_service.clone(),
        xmpp_service: xmpp_service.clone(),
        oauth2: prosody_oauth2.clone(),
        invites_register_api: prosody_invites_register_api.clone(),
        db: db.clone(),
    }));
    let workspace_service = WorkspaceService {
        implem: Arc::new(LiveWorkspaceService {
            server_api: server_api.clone(),
            workspace_username: app_config
                .service_accounts
                .prose_workspace
                .xmpp_node
                .clone(),
        }),
    };

    AppState {
        base,
        db,
        app_config: Arc::new(app_config),
        user_repository,
        invitation_repository,
        xmpp_service,
        auth_service,
        email_notifier,
        member_service,
        network_checker,
        workspace_service,
        licensing_service,
        pod_version_service,
        factory_reset_service,
        prose_pod_server_service,
        identity_provider,
        user_application_service,
        invitation_application_service,
    }
}
