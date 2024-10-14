// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hickory_proto::rr::Name as DomainName;
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::entity::pod_config;

/// The Prose Pod configuration, almost as stored in the database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PodConfig {
    pub address: Option<PodAddress>,
}

impl From<pod_config::Model> for PodConfig {
    fn from(model: pod_config::Model) -> Self {
        Self {
            address: PodAddress::try_from(model).ok(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PodAddress {
    Static {
        ipv4: Option<Ipv4Addr>,
        ipv6: Option<Ipv6Addr>,
    },
    Dynamic {
        hostname: DomainName,
    },
}

impl TryFrom<pod_config::Model> for PodAddress {
    type Error = PodAddressError;

    fn try_from(pod_config: pod_config::Model) -> Result<Self, Self::Error> {
        match (pod_config.hostname, pod_config.ipv4, pod_config.ipv6) {
            (Some(hostname), _, _) => {
                let hostname = DomainName::from_str(&hostname).map_err(|err| {
                    PodAddressError::InvalidData(format!("Invalid hostname: {err}"))
                })?;
                Ok(Self::Dynamic { hostname })
            }
            (None, None, None) => Err(PodAddressError::PodAddressNotInitialized),
            (None, ipv4, ipv6) => {
                let ipv4 = ipv4
                    .map_or(Ok(None), |s| Ipv4Addr::from_str(&s).map(Some))
                    .map_err(|err| PodAddressError::InvalidData(format!("Invalid IPv4: {err}")))?;
                let ipv6 = ipv6
                    .map_or(Ok(None), |s| Ipv6Addr::from_str(&s).map(Some))
                    .map_err(|err| PodAddressError::InvalidData(format!("Invalid IPv6: {err}")))?;
                Ok(Self::Static { ipv4, ipv6 })
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PodAddressError {
    #[error("Prose Pod address not initialized.")]
    PodAddressNotInitialized,
    #[error("Invalid data stored: {0}.")]
    InvalidData(String),
}
