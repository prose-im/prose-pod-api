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

use entity::model::MemberRole;
use entity::server_config;
use prose_xmpp::BareJid;
use secrecy::SecretString;

use crate::config::Config;

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
pub trait ServerCtlImpl: Debug + Sync + Send {
    fn save_config(
        &self,
        server_config: &server_config::Model,
        app_config: &Config,
    ) -> Result<(), Error>;
    fn reload(&self) -> Result<(), Error>;

    fn add_user(&self, jid: &BareJid, password: &SecretString) -> Result<(), Error>;
    fn remove_user(&self, jid: &BareJid) -> Result<(), Error>;
    fn set_user_role(&self, jid: &BareJid, role: &MemberRole) -> Result<(), Error>;
    fn add_user_with_role(
        &self,
        jid: &BareJid,
        password: &SecretString,
        role: &MemberRole,
    ) -> Result<(), Error> {
        self.add_user(jid, password)
            .and_then(|_| self.set_user_role(jid, role))
    }
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
