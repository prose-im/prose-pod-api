// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;
use std::path::PathBuf;
use std::process::Output;
use std::str::{self, Utf8Error};
use std::{fmt, io};

use entity::model::{MemberRole, JID};
use entity::server_config;

use crate::config::Config;

pub struct ServerCtl {
    pub implem: Box<dyn ServerCtlImpl>,
}

impl ServerCtl {
    pub fn new(implem: Box<dyn ServerCtlImpl>) -> Self {
        Self { implem }
    }
}

impl Deref for ServerCtl {
    type Target = Box<dyn ServerCtlImpl>;

    fn deref(&self) -> &Self::Target {
        &self.implem
    }
}

/// Abstraction over `prosodyctl` in case we want to switch to another server.
/// Also facilitates testing.
pub trait ServerCtlImpl: Sync + Send {
    fn save_config(
        &self,
        server_config: &server_config::Model,
        app_config: &Config,
    ) -> Result<(), Error>;
    fn reload(&self) -> Result<(), Error>;

    fn add_user(&self, jid: &JID, password: &str) -> Result<(), Error>;
    fn remove_user(&self, jid: &JID) -> Result<(), Error>;
    fn set_user_role(&self, jid: &JID, role: &MemberRole) -> Result<(), Error>;
    fn add_user_with_role(
        &self,
        jid: &JID,
        password: &str,
        role: &MemberRole,
    ) -> Result<(), Error> {
        self.add_user(jid, password)
            .and_then(|_| self.set_user_role(jid, role))
    }
}

pub type Error = ServerCtlError;

#[derive(Debug, thiserror::Error)]
pub enum ServerCtlError {
    CannotOpenConfigFile(PathBuf, io::Error),
    CannotWriteConfigFile(PathBuf, io::Error),
    CommandFailed(Output),
    Utf8Error(Utf8Error),
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CannotOpenConfigFile(path, err) => write!(
                f,
                "Cannot create Prosody config file at path `{}`: {err}",
                path.display()
            ),
            Self::CannotWriteConfigFile(path, err) => write!(
                f,
                "Cannot write Prosody config file at path `{}`: {err}",
                path.display()
            ),
            Self::CommandFailed(output) => write!(
                f,
                "Command failed ({}):\nstdout: {}\nstderr: {}",
                output.status,
                str::from_utf8(&output.stdout).unwrap(),
                str::from_utf8(&output.stderr).unwrap(),
            ),
            Self::Utf8Error(err) => write!(f, "UTF-8 error: {err}"),
            Self::Other(err) => write!(f, "{err}"),
        }
    }
}

impl From<Utf8Error> for Error {
    fn from(value: Utf8Error) -> Self {
        Self::Utf8Error(value)
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Other(value.to_string())
    }
}
