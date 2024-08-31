// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::{Deserialize, Serialize};

use crate::entity::pod_config;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PodConfig {
    pub address: PodAddress,
}

// TODO: Replace `String` by parsed types (ensuring format and preventing bad user input).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PodAddress {
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
    pub hostname: Option<String>,
}

impl From<pod_config::Model> for PodConfig {
    fn from(model: pod_config::Model) -> Self {
        Self {
            address: PodAddress {
                ipv4: model.ipv4,
                ipv6: model.ipv6,
                hostname: model.hostname,
            },
        }
    }
}
