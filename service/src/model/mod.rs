// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod durations;
mod email_address;
pub(crate) mod invitations;
mod jid_node;
pub mod members;
pub mod server_config;

pub use durations::*;
pub use email_address::*;
pub use invitations::*;
use jid::{DomainPart, DomainRef, NodePart, NodeRef};
pub use jid_node::*;
pub use members::*;
pub use server_config::*;

use sea_orm::entity::prelude::*;
use sea_orm::sea_query::{self, ArrayType, ValueTypeErr};
use sea_orm::TryGetError;
use serde::{Deserialize, Serialize};

use std::fmt::{Debug, Display};
use std::ops::Deref;
use std::str::FromStr;

pub use crate::entity::{
    member::Model as Member, server_config::Model as ServerConfig, workspace::Model as Workspace,
    workspace_invitation::Model as Invitation,
};

// ===== JID =====

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct JID(jid::BareJid);

impl Deref for JID {
    type Target = jid::BareJid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl JID {
    pub fn new<S1: ToString, S2: ToString>(node: S1, domain: S2) -> Result<Self, jid::Error> {
        Ok(Self(jid::BareJid::from_parts(
            Some(&NodePart::from_str(node.to_string().as_str())?),
            &DomainPart::from_str(domain.to_string().as_str())?,
        )))
    }
    pub fn from_parts(node: Option<&NodeRef>, domain: &DomainRef) -> Self {
        Self(jid::BareJid::from_parts(node, domain))
    }
}

impl Display for JID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl FromStr for JID {
    type Err = <<Self as Deref>::Target as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <Self as Deref>::Target::from_str(s).map(Self)
    }
}

impl From<jid::BareJid> for JID {
    fn from(bare_jid: jid::BareJid) -> Self {
        Self(bare_jid)
    }
}

impl From<EmailAddress> for JID {
    fn from(email_address: EmailAddress) -> Self {
        // NOTE: Email adresses are already parsed, and they are equivalent to a JID.
        Self::from_str(email_address.as_str()).unwrap()
    }
}

impl TryFrom<String> for JID {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl Serialize for JID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Serialize::serialize(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for JID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        jid::BareJid::deserialize(deserializer).map(Self)
    }
}

impl From<JID> for sea_query::Value {
    fn from(jid: JID) -> Self {
        Self::String(Some(Box::new(jid.to_string())))
    }
}

impl sea_orm::TryGetable for JID {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::prelude::QueryResult,
        index: I,
    ) -> Result<Self, TryGetError> {
        let value: String = res.try_get_by(index).map_err(TryGetError::DbErr)?;
        Self::try_from(value)
            // Technically, the value is not `null`, but we wouldn't want to unsafely unwrap here.
            .map_err(|e| TryGetError::Null(format!("{:?}", e)))
    }
}

impl sea_query::ValueType for JID {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::String(Some(value)) => (*value).try_into().map_err(|_| ValueTypeErr),
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(JID).to_string()
    }

    fn array_type() -> ArrayType {
        ArrayType::String
    }

    fn column_type() -> ColumnType {
        ColumnType::string(None)
    }
}

impl sea_query::Nullable for JID {
    fn null() -> Value {
        Value::String(None)
    }
}
