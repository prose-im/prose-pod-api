// prosody-config
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt;

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

use super::{ArchivePolicy, RoomCreationRestriction};

// MARK: RoomCreationRestriction

impl Serialize for RoomCreationRestriction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // See <https://prosody.im/doc/chatrooms#creating_rooms>.
        match self {
            Self::NotRestricted => serializer.serialize_bool(false),
            Self::AdminsOnly => serializer.serialize_bool(true),
            Self::DomainOnly => serializer.serialize_str("local"),
        }
    }
}

struct RoomCreationRestrictionVisitor;

impl<'de> Visitor<'de> for RoomCreationRestrictionVisitor {
    type Value = RoomCreationRestriction;

    /// See <https://prosody.im/doc/chatrooms#creating_rooms>.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a boolean (true/false) or the string \"local\"")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(match value {
            false => RoomCreationRestriction::NotRestricted,
            true => RoomCreationRestriction::AdminsOnly,
        })
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            "local" => Ok(RoomCreationRestriction::DomainOnly),
            _ => Err(E::unknown_variant(
                value,
                &[
                    "true", "false", "local",
                ],
            )),
        }
    }
}

impl<'de> Deserialize<'de> for RoomCreationRestriction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(RoomCreationRestrictionVisitor)
    }
}

// MARK: ArchivePolicy

impl Serialize for ArchivePolicy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // See <https://prosody.im/doc/modules/mod_mam>.
        match self {
            Self::Always => serializer.serialize_bool(true),
            Self::OnlyIfUserEnabled => serializer.serialize_bool(false),
            Self::ContactsOnly => serializer.serialize_str("local"),
        }
    }
}

struct ArchivePolicyVisitor;

impl<'de> Visitor<'de> for ArchivePolicyVisitor {
    type Value = ArchivePolicy;

    /// See <https://prosody.im/doc/modules/mod_mam>.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a boolean (true/false) or the string \"roster\"")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(match value {
            true => ArchivePolicy::Always,
            false => ArchivePolicy::OnlyIfUserEnabled,
        })
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            "roster" => Ok(ArchivePolicy::ContactsOnly),
            _ => Err(E::unknown_variant(
                value,
                &[
                    "true", "false", "roster",
                ],
            )),
        }
    }
}

impl<'de> Deserialize<'de> for ArchivePolicy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ArchivePolicyVisitor)
    }
}
