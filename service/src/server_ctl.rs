// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt;
use std::ops::Deref;
use std::process::Output;
use std::sync::{Arc, Mutex};

use entity::model::JID;

pub struct ServerCtl {
    pub implem: Arc<Mutex<dyn ServerCtlImpl>>,
}

impl ServerCtl {
    pub fn new(implem: Arc<Mutex<dyn ServerCtlImpl>>) -> Self {
        Self { implem }
    }
}

impl Deref for ServerCtl {
    type Target = Arc<Mutex<dyn ServerCtlImpl>>;

    fn deref(&self) -> &Self::Target {
        &self.implem
    }
}

/// Abstraction over ProsodyCtl in case we want to switch to another server.
/// Also facilitates testing.
pub trait ServerCtlImpl: Sync + Send {
    fn start(&self) -> Result<(), Error>;
    fn stop(&self) -> Result<(), Error>;
    fn restart(&self) -> Result<(), Error>;
    fn reload(&self) -> Result<(), Error>;
    fn status(&self) -> Result<(), Error>;

    fn add_user(&self, jid: &JID, password: &str) -> Result<(), Error>;
    fn remove_user(&self, jid: &JID) -> Result<(), Error>;
}

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    CommandFailed(Output),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(err) => write!(f, "IO error: {err}"),
            Self::CommandFailed(output) => write!(
                f,
                "Command failed (status: {}):\n{:?}",
                output.status, output.stderr,
            ),
        }
    }
}
