// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
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
    config::Config,
    model::{MemberRole, ServerConfig},
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
    async fn save_config(
        &self,
        server_config: &ServerConfig,
        app_config: &Config,
    ) -> Result<(), Error>;
    async fn reload(&self) -> Result<(), Error>;

    async fn add_user(&self, jid: &BareJid, password: &SecretString) -> Result<(), Error>;
    async fn remove_user(&self, jid: &BareJid) -> Result<(), Error>;

    async fn set_user_role(&self, jid: &BareJid, role: &MemberRole) -> Result<(), Error>;
    async fn set_user_password(&self, jid: &BareJid, password: &SecretString) -> Result<(), Error>;

    /// Add a user to everyone's roster.
    async fn add_team_member(&self, jid: &BareJid) -> Result<(), Error>;
    /// Remove a user from everyone's roster.
    async fn remove_team_member(&self, jid: &BareJid) -> Result<(), Error>;
}

pub type Error = ServerCtlError;

#[derive(Debug, thiserror::Error)]
pub enum ServerCtlError {
    #[error("Cannot create Prosody config file at path `{}`: {1}", ._0.display())]
    CannotOpenConfigFile(PathBuf, io::Error),
    #[error("Cannot write Prosody config file at path `{}`: {1}", ._0.display())]
    CannotWriteConfigFile(PathBuf, io::Error),
    #[error(
        "Command failed ({}):\nstdout: {}\nstderr: {}",
        ._0.status,
        str::from_utf8(&._0.stdout).unwrap(),
        str::from_utf8(&._0.stderr).unwrap(),
    )]
    CommandFailed(Output),
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] Utf8Error),
    #[error("{0}")]
    Other(String),
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Other(value.to_string())
    }
}
