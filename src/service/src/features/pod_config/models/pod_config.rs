// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hickory_proto::rr::Name as DomainName;
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::{models::Url, pod_config::entities::pod_config};

/// The Prose Pod configuration, almost as stored in the database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PodConfig {
    pub address: Option<NetworkAddress>,
    pub dashboard_url: Option<Url>,
}

#[derive(Debug, Clone, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum PodConfigField {
    PodAddress,
    DashboardUrl,
}

impl From<pod_config::Model> for PodConfig {
    fn from(model: pod_config::Model) -> Self {
        Self {
            address: model.pod_address(),
            dashboard_url: model.dashboard_url,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NetworkAddress {
    Static {
        ipv4: Option<Ipv4Addr>,
        ipv6: Option<Ipv6Addr>,
    },
    Dynamic {
        hostname: DomainName,
    },
}

impl NetworkAddress {
    pub fn try_from(
        hostname: Option<&String>,
        ipv4: Option<&String>,
        ipv6: Option<&String>,
    ) -> Result<Option<Self>, InvalidNetworkAddress> {
        match (hostname, ipv4, ipv6) {
            (Some(hostname), _, _) => {
                let hostname = DomainName::from_str(&hostname)
                    .map_err(|err| InvalidNetworkAddress(format!("Invalid hostname: {err}")))?;
                Ok(Some(Self::Dynamic { hostname }))
            }
            (None, None, None) => Ok(None),
            (None, ipv4, ipv6) => {
                let ipv4 = ipv4
                    .as_ref()
                    .map_or(Ok(None), |s| Ipv4Addr::from_str(&s).map(Some))
                    .map_err(|err| InvalidNetworkAddress(format!("Invalid IPv4: {err}")))?;
                let ipv6 = ipv6
                    .as_ref()
                    .map_or(Ok(None), |s| Ipv6Addr::from_str(&s).map(Some))
                    .map_err(|err| InvalidNetworkAddress(format!("Invalid IPv6: {err}")))?;
                Ok(Some(Self::Static { ipv4, ipv6 }))
            }
        }
    }

    #[inline]
    pub fn try_from_or_warn(
        hostname: Option<&String>,
        ipv4: Option<&String>,
        ipv6: Option<&String>,
        warning: &'static str,
    ) -> Option<Self> {
        match Self::try_from(hostname, ipv4, ipv6) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("{warning}: {e}");
                None
            }
        }
    }
}

impl ToString for NetworkAddress {
    fn to_string(&self) -> String {
        match self {
            Self::Static { ipv4, ipv6 } => {
                ipv6.as_ref().map(Ipv6Addr::to_string).unwrap_or_else(|| {
                    ipv4.as_ref()
                        .map(Ipv4Addr::to_string)
                        .expect("IPv4 or IPv6 must be defined by this point.")
                })
            }
            Self::Dynamic { hostname } => hostname.to_string(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid network address: {0}.")]
pub struct InvalidNetworkAddress(String);
