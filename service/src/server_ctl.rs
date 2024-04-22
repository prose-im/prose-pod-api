// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;
use std::process::Output;
use std::str::{self, Utf8Error};
use std::sync::{Arc, Mutex};
use std::{fmt, io};

use entity::model::{MemberRole, JID};
use vcard_parser::error::VcardError;
use vcard_parser::vcard::property::property_nickname::PropertyNickNameData;
use vcard_parser::vcard::property::Property;
use vcard_parser::vcard::Vcard;

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

    fn test_user_password(&self, jid: &JID, password: &str) -> Result<bool, Error>;

    fn get_vcard(&self, jid: &JID) -> Result<Option<Vcard>, Error>;
    fn set_vcard(&self, jid: &JID, vcard: &Vcard) -> Result<(), Error>;

    fn create_vcard(&self, jid: &JID, name: &str) -> Result<(), Error> {
        let vcard = Vcard::new(name);
        self.set_vcard(jid, &vcard)
    }
    fn set_nickname(&self, jid: &JID, nickname: &str) -> Result<(), Error> {
        let mut vcard = self.get_vcard(jid)?.unwrap_or(Vcard::new(nickname));

        vcard.set_property(
            &PropertyNickNameData::try_from((None, nickname, vec![]))
                .map(Property::PropertyNickName)?,
        )?;

        self.set_vcard(jid, &vcard)
    }
}

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    CommandFailed(Output),
    Utf8Error(Utf8Error),
    VcardError(VcardError),
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(err) => write!(f, "IO error: {err}"),
            Self::CommandFailed(output) => write!(
                f,
                "Command failed ({}):\nstdout: {}\nstderr: {}",
                output.status,
                str::from_utf8(&output.stdout).unwrap(),
                str::from_utf8(&output.stderr).unwrap(),
            ),
            Self::Utf8Error(err) => write!(f, "UTF-8 error: {err}"),
            Self::VcardError(err) => write!(f, "vCard error: {err}"),
            Self::Other(err) => write!(f, "{err}"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<Utf8Error> for Error {
    fn from(value: Utf8Error) -> Self {
        Self::Utf8Error(value)
    }
}

impl From<VcardError> for Error {
    fn from(value: VcardError) -> Self {
        Self::VcardError(value)
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Other(value.to_string())
    }
}
