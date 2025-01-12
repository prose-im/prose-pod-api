// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::{Arc, RwLock, RwLockWriteGuard};

use jid::BareJid;
use rand::{distributions::Alphanumeric, thread_rng, Rng as _};
use sea_orm::IntoActiveModel as _;
use secrecy::SecretString;
use tracing::{debug, trace};

use crate::{
    auth::{auth_service, AuthService},
    models::{DateLike, Duration, JidDomain, PossiblyInfinite},
    sea_orm::{ActiveModelTrait as _, DatabaseConnection, Set, TransactionTrait as _},
    secrets::{SecretsStore, ServiceAccountSecrets},
    server_config::{
        entities::server_config, ServerConfig, ServerConfigCreateForm, ServerConfigRepository,
        TlsProfile,
    },
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

    pub async fn rotate_api_xmpp_password(
        server_ctl: &ServerCtl,
        app_config: &AppConfig,
        secrets_store: &SecretsStore,
    ) -> Result<(), ServerCtlError> {
        let api_jid = app_config.api_jid();
        let password = Self::strong_random_password();

        server_ctl.set_user_password(&api_jid, &password).await?;
        secrets_store.set_prose_pod_api_xmpp_password(password);

        Ok(())
    }

    pub async fn set_domain(&self, domain: &JidDomain) -> Result<ServerConfig, Error> {
        trace!("Setting XMPP server domain to {domain}…");
        self.update(|active| {
            active.domain = Set(domain.to_owned());
        })
        .await
    }

    pub async fn reset_messaging_config(&self) -> Result<ServerConfig, Error> {
        trace!("Resetting messaging configuration…");
        let model = self
            .update(|active| {
                active.message_archive_enabled = Set(None);
                active.message_archive_retention = Set(None);
            })
            .await?;
        Ok(model)
    }

    pub async fn reset_files_config(&self) -> Result<ServerConfig, Error> {
        trace!("Resetting files configuration…");
        let model = self
            .update(|active| {
                active.file_upload_allowed = Set(None);
                active.file_storage_encryption_scheme = Set(None);
                active.file_storage_retention = Set(None);
            })
            .await?;
        Ok(model)
    }

    pub async fn reset_push_notifications_config(&self) -> Result<ServerConfig, Error> {
        trace!("Resetting push notifications configuration…");
        let model = self
            .update(|active| {
                active.push_notification_with_body = Set(None);
                active.push_notification_with_sender = Set(None);
            })
            .await?;
        Ok(model)
    }

    pub async fn reset_network_encryption_config(&self) -> Result<ServerConfig, Error> {
        trace!("Resetting network encryption configuration…");
        let model = self
            .update(|active| {
                active.tls_profile = Set(None);
            })
            .await?;
        Ok(model)
    }
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

macro_rules! set_bool {
    ($fn:ident, $var:ident) => {
        pub async fn $fn(&self, new_state: bool) -> Result<ServerConfig, Error> {
            trace!(
                "Turning {} {}…",
                stringify!($var),
                if new_state { "on" } else { "off" },
            );
            self.update(|active| active.$var = Set(Some(new_state)))
                .await
        }
    };
}
macro_rules! set {
    ($t:ty, $fn:ident, $var:ident) => {
        pub async fn $fn(&self, new_state: $t) -> Result<ServerConfig, Error> {
            trace!("Setting {} to {new_state}…", stringify!($var));
            self.update(|active| active.$var = Set(Some(new_state)))
                .await
        }
    };
}
macro_rules! reset {
    ($fn:ident, $var:ident) => {
        pub async fn $fn(&self) -> Result<ServerConfig, Error> {
            trace!("Resetting {}…", stringify!($var));
            self.update(|active| active.$var = Set(None)).await
        }
    };
}

impl ServerManager {
    set_bool!(set_message_archive_enabled, message_archive_enabled);

    set!(
        PossiblyInfinite<Duration<DateLike>>,
        set_message_archive_retention,
        message_archive_retention
    );
    reset!(reset_message_archive_retention, message_archive_retention);

    set_bool!(set_file_upload_allowed, file_upload_allowed);
    set!(
        PossiblyInfinite<Duration<DateLike>>,
        set_file_storage_retention,
        file_storage_retention
    );

    // Push notifications
    set_bool!(set_push_notification_with_body, push_notification_with_body);
    reset!(
        reset_push_notification_with_body,
        push_notification_with_body
    );
    set_bool!(
        set_push_notification_with_sender,
        push_notification_with_sender
    );
    reset!(
        reset_push_notification_with_sender,
        push_notification_with_sender
    );

    // Network encryption
    set!(TlsProfile, set_tls_profile, tls_profile);
    reset!(reset_tls_profile, tls_profile);
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
