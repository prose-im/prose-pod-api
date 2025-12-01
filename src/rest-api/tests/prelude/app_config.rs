// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use service::{
    auth::AuthService,
    identity_provider::IdentityProvider,
    invitations::InvitationRepository,
    licensing::LicensingService,
    members::{UserApplicationService, UserRepository},
    prose_pod_server_service::ProsePodServerService,
    workspace::WorkspaceService,
    xmpp::XmppService,
    AppConfig,
};

use crate::prelude::mocks::{
    MockAuthService, MockIdentityProvider, MockInvitationRepository, MockServerService,
    MockUserRepository, MockUserService, MockWorkspaceService, MockXmppService,
};

use super::{mocks::MockLicensingService, test_world::CONFIG_PATH};

pub fn reload_config(world: &mut crate::TestWorld) {
    let figment =
        AppConfig::figment_at_path(CONFIG_PATH.as_path()).merge(world.config_overrides.clone());
    let config = Arc::new(
        AppConfig::from_figment(figment)
            .expect(&format!("Invalid config file at {}", CONFIG_PATH.display())),
    );
    world.app_config = Some(config.clone());

    let mock_licensing_service = Arc::new(MockLicensingService::new(config.server_fqdn()));

    let mock_xmpp_service = Arc::new(MockXmppService {
        state: world.mock_xmpp_service_state.clone(),
        mock_server_state: world.mock_server_state.clone(),
    });

    let mock_user_repository = Arc::new(MockUserRepository {
        state: world.mock_user_repository_state.clone(),
        mock_server_state: world.mock_server_state.clone(),
        mock_auth_service_state: world.mock_auth_service_state.clone(),
        mock_xmpp_service: mock_xmpp_service.clone(),
        server_domain: config.server_domain().clone(),
    });
    let mock_invitation_repository = Arc::new(MockInvitationRepository {
        state: Default::default(),
        mock_server_state: world.mock_server_state.clone(),
        mock_auth_service_state: world.mock_auth_service_state.clone(),
        server_domain: config.server_domain().clone(),
        invitations_ttl: (config.auth.invitation_ttl.to_std().unwrap())
            .try_into()
            .unwrap(),
    });

    let mock_server_service = Arc::new(MockServerService {
        state: world.mock_server_state.clone(),
        db: world.db.clone(),
        mock_auth_service_state: world.mock_auth_service_state.clone(),
        mock_user_repository: mock_user_repository.clone(),
    });
    let mock_auth_service = Arc::new(MockAuthService {
        state: world.mock_auth_service_state.clone(),
        server: mock_server_service.clone(),
        mock_user_repository: mock_user_repository.clone(),
        password_reset_tokens_ttl: (config.auth.password_reset_token_ttl.to_std().unwrap())
            .try_into()
            .unwrap(),
        min_password_length: config.auth.min_password_length,
        server_domain: config.server_domain().clone(),
    });
    let mock_user_application_service = Arc::new(MockUserService {
        server: mock_server_service.clone(),
        mock_user_repository: mock_user_repository.clone(),
        mock_auth_service: mock_auth_service.clone(),
        server_domain: config.server_domain().clone(),
    });
    let mock_workspace_service = Arc::new(MockWorkspaceService {
        state: world.mock_workspace_service_state().clone(),
        mock_auth_service: mock_auth_service.clone(),
    });
    let mock_identity_provider = Arc::new(MockIdentityProvider {
        state: world.mock_identity_provider_state.clone(),
        mock_user_repository_state: world.mock_user_repository_state.clone(),
    });

    world.licensing_service = Some(LicensingService::new(mock_licensing_service.clone()));
    world.mock_licensing_service = Some(mock_licensing_service);

    world.user_repository = Some(UserRepository {
        implem: mock_user_repository.clone(),
    });
    world.mock_user_repository = Some(mock_user_repository);

    world.invitation_repository = Some(InvitationRepository {
        implem: mock_invitation_repository.clone(),
    });
    world.mock_invitation_repository = Some(mock_invitation_repository);

    world.server_service = Some(ProsePodServerService(mock_server_service.clone()));
    world.mock_server_service = Some(mock_server_service);

    world.auth_service = Some(AuthService {
        implem: mock_auth_service.clone(),
    });
    world.mock_auth_service = Some(mock_auth_service);

    world.user_application_service = Some(UserApplicationService {
        implem: mock_user_application_service.clone(),
    });
    world.mock_user_application_service = Some(mock_user_application_service);

    world.xmpp_service = Some(XmppService {
        implem: mock_xmpp_service.clone(),
    });
    world.mock_xmpp_service = Some(mock_xmpp_service);

    world.workspace_service = Some(WorkspaceService {
        implem: mock_workspace_service.clone(),
    });
    world.mock_workspace_service = Some(mock_workspace_service);

    world.identity_provider = Some(IdentityProvider {
        implem: mock_identity_provider.clone(),
    });
    world.mock_identity_provider = Some(mock_identity_provider);
}
