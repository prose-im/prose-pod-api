// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use service::{
    dependencies,
    licensing::LicenseService,
    secrets::{LiveSecretsStore, SecretsStore},
    AppConfig,
};

use super::{
    mocks::{MockLicenseService, MockSecretsStore},
    test_world::CONFIG_PATH,
};

pub fn reload_config(world: &mut crate::TestWorld) {
    let figment =
        AppConfig::figment_at_path(CONFIG_PATH.as_path()).merge(world.config_overrides.clone());
    let config = AppConfig::from_figment(figment)
        .expect(&format!("Invalid config file at {}", CONFIG_PATH.display()));

    let mock_license_service = Arc::new(MockLicenseService::new(config.server_fqdn()));
    let mock_secrets_store = Arc::new(MockSecretsStore::new(LiveSecretsStore::default(), &config));
    let uuid_gen = dependencies::Uuid::from_config(&config);

    world.app_config = Some(Arc::new(config));
    world.license_service = Some(LicenseService::new(mock_license_service.clone()));
    world.mock_license_service = Some(mock_license_service);
    world.secrets_store = Some(SecretsStore::new(mock_secrets_store.clone()));
    world.mock_secrets_store = Some(mock_secrets_store);
    world.uuid_gen = Some(uuid_gen);
}
