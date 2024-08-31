// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::entity::pod_config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PodAddress {
    Static { ipv4: String, ipv6: Option<String> },
    Dynamic { hostname: String },
}

#[derive(Debug, thiserror::Error)]
pub enum PodAddressError {
    #[error("Prose Pod address not initialized.")]
    PodAddressNotInitialized,
}

impl TryFrom<pod_config::Model> for PodAddress {
    type Error = PodAddressError;

    fn try_from(pod_config: pod_config::Model) -> Result<Self, Self::Error> {
        if let Some(hostname) = pod_config.hostname {
            Ok(Self::Dynamic { hostname })
        } else if let Some(ipv4) = pod_config.ipv4 {
            Ok(Self::Static {
                ipv4,
                ipv6: pod_config.ipv6,
            })
        } else {
            Err(PodAddressError::PodAddressNotInitialized)
        }
    }
}
