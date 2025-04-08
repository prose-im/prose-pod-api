// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::{Arc, RwLock, RwLockWriteGuard};

use jid::BareJid;
use linked_hash_set::LinkedHashSet;
use rand::{distributions::Alphanumeric, thread_rng, Rng as _};
use sea_orm::IntoActiveModel as _;
use secrecy::{ExposeSecret, SecretString};
use tracing::{debug, trace};

use crate::{
    auth::{auth_service, AuthService},
    models::{sea_orm::LinkedStringSet, DateLike, Duration, JidDomain, PossiblyInfinite},
    prosody::ProsodyOverrides,
    sea_orm::{ActiveModelTrait as _, DatabaseConnection, Set, TransactionTrait as _},
    secrets::{SecretsStore, ServiceAccountSecrets},
    server_config::{
        entities::server_config, ServerConfig, ServerConfigCreateForm, ServerConfigRepository,
        TlsProfile,
    },
    util::Either,
    AppConfig,
};

use super::{server_ctl, ServerCtl, ServerCtlError};

#[derive(Clone)]
pub struct ServerManager {
    db: Arc<DatabaseConnection>,
    app_config: Arc<AppConfig>,
    server_ctl: Arc<ServerCtl>,
    server_config: Arc<RwLock<server_config::Model>>,
}

impl ServerManager {
    pub fn new(
        db: Arc<DatabaseConnection>,
        app_config: Arc<AppConfig>,
        server_ctl: Arc<ServerCtl>,
        server_config: server_config::Model,
    ) -> Self {
        Self {
            db,
            app_config,
            server_ctl,
            server_config: Arc::new(RwLock::new(server_config)),
        }
    }

    fn server_config_mut(&self) -> RwLockWriteGuard<server_config::Model> {
        self.server_config
            .write()
            .expect("`server_config::Model` lock poisonned")
    }

    fn server_config(&self) -> server_config::Model {
        self.server_config
            .read()
            .expect("`server_config::Model` lock poisonned")
            .to_owned()
    }
}

impl ServerManager {
    async fn update<U>(&self, update: U) -> Result<ServerConfig, Error>
    where
        U: FnOnce(&mut server_config::ActiveModel) -> (),
    {
        let old_server_config = self.server_config();

        let mut active: server_config::ActiveModel = old_server_config.clone().into_active_model();
        update(&mut active);
        trace!("Updating config in database…");
        let new_server_config = active.update(self.db.as_ref()).await?;
        *self.server_config_mut() = new_server_config.clone();

        if new_server_config != old_server_config {
            trace!("Server config has changed, reloading…");
            self.reload(&new_server_config).await?;
        } else {
            trace!("Server config hasn't changed, no need to reload.");
        }

        Ok(new_server_config.with_default_values_from(&self.app_config))
    }

    /// Reload the XMPP server using the server configuration stored in `self`.
    pub async fn reload_current(&self) -> Result<(), Error> {
        self.reload(&self.server_config()).await
    }

    /// Reload the XMPP server using the server configuration passed as an argument.
    async fn reload(&self, server_config: &server_config::Model) -> Result<(), Error> {
        let server_ctl = self.server_ctl.as_ref();

        // Save new server config
        trace!("Saving server config…");
        server_ctl
            .save_config(
                &server_config.with_default_values_from(&self.app_config),
                &self.app_config,
            )
            .await?;
        // Reload server config
        trace!("Reloading XMPP server…");
        server_ctl.reload().await?;

        Ok(())
    }

    /// Generates a very strong random password.
    fn strong_random_password() -> SecretString {
        // NOTE: Code taken from <https://rust-lang-nursery.github.io/rust-cookbook/algorithms/randomness.html#create-random-passwords-from-a-set-of-alphanumeric-characters>.
        thread_rng()
            .sample_iter(&Alphanumeric)
            // 256 characters because why not
            .take(256)
            .map(char::from)
            .collect::<String>()
            .into()
    }
}

impl ServerManager {
    pub async fn init_server_config(
        db: &DatabaseConnection,
        server_ctl: &ServerCtl,
        app_config: &AppConfig,
        server_config: impl Into<ServerConfigCreateForm>,
    ) -> Result<ServerConfig, Error> {
        let None = ServerConfigRepository::get(db).await? else {
            return Err(Error::ServerConfigAlreadyInitialized);
        };

        let txn = db.begin().await?;

        // Initialize the server config in a transaction,
        // to rollback if subsequent operations fail.
        let model = ServerConfigRepository::create(&txn, server_config).await?;
        let server_config = model.with_default_values_from(app_config);

        // NOTE: We can't rollback changes made to the XMPP server so let's do it
        //   after "rollbackable" DB changes in case they fail. It's not perfect
        //   but better than nothing.
        // TODO: Find a way to rollback XMPP server changes.
        {
            server_ctl.save_config(&server_config, app_config).await?;
            server_ctl.reload().await?;
        }

        // Commit the transaction only if the admin user was
        // successfully created, to prevent inconsistent states.
        txn.commit().await?;

        Ok(server_config)
    }

    async fn set_api_xmpp_password(
        server_ctl: &ServerCtl,
        app_config: &AppConfig,
        secrets_store: &SecretsStore,
        password: SecretString,
    ) -> Result<(), ServerCtlError> {
        let api_jid = app_config.api_jid();

        server_ctl.set_user_password(&api_jid, &password).await?;
        secrets_store.set_prose_pod_api_xmpp_password(password);

        Ok(())
    }

    pub async fn reset_server_config(
        db: &DatabaseConnection,
        server_ctl: &ServerCtl,
        app_config: &AppConfig,
        secrets_store: &SecretsStore,
    ) -> Result<(), Error> {
        ServerConfigRepository::reset(db).await?;

        // Write the bootstrap configuration.
        let password = Self::strong_random_password();
        server_ctl
            .reset_config(&password)
            .await
            .map_err(Error::from)?;

        // Update the API user password to match the new one specified in the bootstrap configuration.
        Self::set_api_xmpp_password(server_ctl, app_config, secrets_store, password.clone())
            .await?;

        // Store the new password in the environment variables to the next API instance
        // can access it (the screts store will be dropped efore next run).
        std::env::set_var(
            "PROSE_BOOTSTRAP__PROSE_POD_API_XMPP_PASSWORD",
            password.expose_secret(),
        );

        // Apply the bootstrap configuration.
        server_ctl.reload().await.map_err(Error::from)?;

        Ok(())
    }

    pub async fn rotate_api_xmpp_password(
        server_ctl: &ServerCtl,
        app_config: &AppConfig,
        secrets_store: &SecretsStore,
    ) -> Result<(), ServerCtlError> {
        Self::set_api_xmpp_password(
            server_ctl,
            app_config,
            secrets_store,
            Self::strong_random_password(),
        )
        .await
    }

    /// NOTE: Used only in tests.
    #[cfg(debug_assertions)]
    pub async fn set_domain(&self, domain: &JidDomain) -> Result<ServerConfig, Error> {
        trace!("Setting XMPP server domain to {domain}…");
        self.update(|active| {
            active.domain = Set(domain.to_owned());
        })
        .await
    }
}

macro_rules! reset_fn {
    ($fn:ident, $display:literal, $($field:ident),*,) => {
        pub async fn $fn(&self) -> Result<ServerConfig, Error> {
            trace!(concat!("Resetting ", $display, "…"));
            let model = self
                .update(|active| {
                    $( active.$field = Set(None); )*
                })
                .await?;
            Ok(model)
        }
    };
}

impl ServerManager {
    reset_fn!(
        reset_messaging_config,
        "messaging configuration",
        message_archive_enabled,
        message_archive_retention,
    );
    reset_fn!(
        reset_files_config,
        "files configuration",
        file_upload_allowed,
        file_storage_encryption_scheme,
        file_storage_retention,
    );
    reset_fn!(
        reset_push_notifications_config,
        "notifications configuration",
        push_notification_with_body,
        push_notification_with_sender,
    );
    reset_fn!(
        reset_network_encryption_config,
        "network encryption configuration",
        tls_profile,
    );
    reset_fn!(
        reset_server_federation_config,
        "server federation configuration",
        federation_enabled,
        federation_whitelist_enabled,
        federation_friendly_servers,
    );
}

impl ServerManager {
    pub async fn create_service_accounts(
        domain: &JidDomain,
        server_ctl: &ServerCtl,
        app_config: &AppConfig,
        auth_service: &AuthService,
        secrets_store: &SecretsStore,
    ) -> Result<(), CreateServiceAccountError> {
        // NOTE: No need to create Prose Pod API's XMPP account as it's already created
        //   automatically when the XMPP server starts (using `mod_init_admin` in Prosody).

        // Create workspace XMPP account
        Self::create_service_account(
            app_config.workspace_jid(domain),
            server_ctl,
            auth_service,
            secrets_store,
        )
        .await?;

        Ok(())
    }

    async fn create_service_account(
        jid: BareJid,
        server_ctl: &ServerCtl,
        auth_service: &AuthService,
        secrets_store: &SecretsStore,
    ) -> Result<(), CreateServiceAccountError> {
        debug!("Creating service account '{jid}'…");

        // Create the XMPP user account
        let password = Self::strong_random_password();
        server_ctl.add_user(&jid, &password).await?;

        // Log in as the service account (to get a JWT with access tokens)
        let auth_token = auth_service.log_in(&jid, &password).await?;

        // Store the secrets
        let secrets = ServiceAccountSecrets {
            password,
            prosody_token: auth_token.clone(),
        };
        secrets_store.set_service_account_secrets(jid, secrets);

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateServiceAccountError {
    #[error("Could not create XMPP account: {0}")]
    CouldNotCreateXmppAccount(#[from] server_ctl::Error),
    #[error("Could not log in: {0}")]
    CouldNotLogIn(#[from] auth_service::Error),
}

macro_rules! set {
    ($t:ty $(as $db_type:ty)?, $fn:ident, $var:ident $(,)?) => {
        pub async fn $fn(&self, new_state: $t) -> Result<ServerConfig, Error> {
            $(let new_state = <$db_type>::from(new_state);)?
            trace!("Setting {} to {new_state}…", stringify!($var));
            self.update(|active| active.$var = Set(Some(new_state)))
                .await
        }
    };
}
macro_rules! get {
    ($t:ty $(as $db_type:ty)?, $fn:ident, $var:ident $(,)?) => {
        pub async fn $fn(&self) -> Result<Option<$t>, Error> {
            trace!("Getting {}…", stringify!($var));
            let $var = self.server_config().$var;
            $(let $var = <$t>::from(<$var as $db_type>);)?
            Ok($var)
        }
    };
}
macro_rules! reset {
    ($fn:ident, $var:ident $(,)?) => {
        pub async fn $fn(&self) -> Result<ServerConfig, Error> {
            trace!("Resetting {}…", stringify!($var));
            self.update(|active| active.$var = Set(None)).await
        }
    };
}
macro_rules! property_helpers {
    (
        $var:ident, type: $t:ty
        $(, set: $set_fn:ident)?
        $(, get: $get_fn:ident)?
        $(, reset: $reset_fn:ident)?
        $(,)?
    ) => {
        $(set!($t, $set_fn, $var);)?
        $(get!($t, $get_fn, $var);)?
        $(reset!($reset_fn, $var);)?
    };
    (
        $var:ident, type: $t:ty as $db_type:ty
        $(, set: $set_fn:ident)?
        $(, get: $get_fn:ident)?
        $(, reset: $reset_fn:ident)?
        $(,)?
    ) => {
        $(set!($t as $db_type, $set_fn, $var);)?
        $(get!($t as $db_type, $get_fn, $var);)?
        $(reset!($reset_fn, $var);)?
    };
}

impl ServerManager {
    // File upload
    property_helpers!(
        file_upload_allowed, type: bool,
        set: set_file_upload_allowed,
        reset: reset_file_upload_allowed,
    );
    property_helpers!(
        file_storage_retention, type: PossiblyInfinite<Duration<DateLike>>,
        set: set_file_storage_retention,
        reset: reset_file_storage_retention,
    );

    // Message archive
    property_helpers!(
        message_archive_enabled, type: bool,
        set: set_message_archive_enabled,
        reset: reset_message_archive_enabled,
    );
    property_helpers!(
        message_archive_retention, type: PossiblyInfinite<Duration<DateLike>>,
        set: set_message_archive_retention,
        reset: reset_message_archive_retention,
    );

    // Push notifications
    property_helpers!(
        push_notification_with_body, type: bool,
        set: set_push_notification_with_body,
        reset: reset_push_notification_with_body,
    );
    property_helpers!(
        push_notification_with_sender, type: bool,
        set: set_push_notification_with_sender,
        reset: reset_push_notification_with_sender,
    );

    // Network encryption
    property_helpers!(
        tls_profile, type: TlsProfile,
        set: set_tls_profile,
        reset: reset_tls_profile,
    );

    // Server federation
    property_helpers!(
        federation_enabled, type: bool,
        set: set_federation_enabled,
        reset: reset_federation_enabled,
    );
    property_helpers!(
        federation_whitelist_enabled, type: bool,
        set: set_federation_whitelist_enabled,
        reset: reset_federation_whitelist_enabled,
    );
    property_helpers!(
        federation_friendly_servers, type: LinkedHashSet<String> as LinkedStringSet,
        set: set_federation_friendly_servers,
        reset: reset_federation_friendly_servers,
    );
}

impl ServerManager {
    pub async fn set_prosody_overrides(
        &self,
        new_state: ProsodyOverrides,
    ) -> Result<ServerConfig, Either<serde_json::Error, Error>> {
        let new_state = serde_json::to_value(new_state).map_err(Either::Left)?;
        trace!("Setting prosody_overrides to {new_state}…");
        self.update(|active| active.prosody_overrides = Set(Some(new_state)))
            .await
            .map_err(Either::Right)
    }

    /// - `Ok(Some(None))` => Server config initialized, no value
    /// - `Ok(None)` => Server config not initialized
    pub async fn get_prosody_overrides(
        &self,
    ) -> Result<Option<Option<ProsodyOverrides>>, Either<sea_orm::DbErr, serde_json::Error>> {
        trace!("Getting prosody_overrides…");
        ServerConfigRepository::get_prosody_overrides(self.db.as_ref()).await
    }

    reset!(reset_prosody_overrides, prosody_overrides);
}

impl ServerManager {
    property_helpers!(
        prosody_overrides_raw, type: String,
        set: set_prosody_overrides_raw,
        get: get_prosody_overrides_raw,
        reset: reset_prosody_overrides_raw,
    );
}

pub type Error = ServerManagerError;

#[derive(Debug, thiserror::Error)]
pub enum ServerManagerError {
    #[error("XMPP server already initialized.")]
    ServerConfigAlreadyInitialized,
    #[error("`ServerCtl` error: {0}")]
    ServerCtl(#[from] server_ctl::Error),
    #[error("Database error: {0}")]
    DbErr(#[from] sea_orm::DbErr),
}
