// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    fmt::Debug,
    io,
    ops::Deref,
    path::PathBuf,
    process::Output,
    str::{self, Utf8Error},
    sync::Arc,
};

use prose_xmpp::BareJid;
use secrecy::SecretString;

use crate::{
    errors::UnexpectedHttpResponse, members::MemberRole, server_config::ServerConfig, AppConfig,
};

#[derive(Debug, Clone)]
pub struct ServerCtl {
    pub implem: Arc<dyn ServerCtlImpl>,
}

impl ServerCtl {
    pub fn new(implem: Arc<dyn ServerCtlImpl>) -> Self {
        Self { implem }
    }
}

impl Deref for ServerCtl {
    type Target = Arc<dyn ServerCtlImpl>;

    fn deref(&self) -> &Self::Target {
        &self.implem
    }
}

/// Abstraction over `prosodyctl` in case we want to switch to another server.
/// Also facilitates testing.
#[async_trait::async_trait]
pub trait ServerCtlImpl: Debug + Sync + Send {
    async fn wait_until_ready(&self) -> Result<(), Error>;

    async fn save_config(
        &self,
        server_config: &ServerConfig,
        app_config: &AppConfig,
    ) -> Result<(), Error>;
    async fn reset_config(&self, init_admin_password: &SecretString) -> Result<(), Error>;
    async fn reload(&self) -> Result<(), Error>;

    async fn add_user(&self, jid: &BareJid, password: &SecretString) -> Result<(), Error>;
    async fn remove_user(&self, jid: &BareJid) -> Result<(), Error>;

    async fn set_user_role(&self, jid: &BareJid, role: &MemberRole) -> Result<(), Error>;
    async fn set_user_password(&self, jid: &BareJid, password: &SecretString) -> Result<(), Error>;

    /// Add a user to everyone's roster.
    async fn add_team_member(&self, jid: &BareJid) -> Result<(), Error>;
    /// Remove a user from everyone's roster.
    async fn remove_team_member(&self, jid: &BareJid) -> Result<(), Error>;
    /// Rosters synchronization is debounced, but sometimes one needs to force
    /// a re-sync (e.g. after a termination).
    async fn force_rosters_sync(&self) -> Result<(), Error>;

    async fn delete_all_data(&self) -> Result<(), Error>;
}

pub type Error = ServerCtlError;

#[derive(Debug, thiserror::Error)]
pub enum ServerCtlError {
    #[error("Cannot create Prosody config file at path `{path}`: {1}", path = ._0.display())]
    CannotOpenConfigFile(PathBuf, io::Error),
    #[error("Cannot write Prosody config file at path `{path}`: {1}", path = ._0.display())]
    CannotWriteConfigFile(PathBuf, io::Error),
    #[error(
        "Command failed ({status}):\nstdout: {stdout}\nstderr: {stderr}",
        status = ._0.status,
        stdout = str::from_utf8(&._0.stdout).unwrap(),
        stderr = str::from_utf8(&._0.stderr).unwrap(),
    )]
    CommandFailed(Output),
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] Utf8Error),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Unexpected API response: {0}")]
    UnexpectedResponse(UnexpectedHttpResponse),
    #[error("{0}")]
    Other(String),
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::Other(format!("reqwest::Error: {error:?}"))
    }
}
