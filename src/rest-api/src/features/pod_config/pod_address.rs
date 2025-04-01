// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::{Ipv4Addr, Ipv6Addr};

use axum::http::StatusCode;
use hickory_resolver::Name as DomainName;
use serde::{Deserialize, Serialize};
use service::pod_config::{PodAddress, PodAddressUpdateForm};

use crate::{
    error::{Error, ErrorCode, HttpApiError, LogLevel},
    pod_config_routes,
};

use super::{invalid_network_address, POD_CONFIG_ROUTE};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PatchPodAddressRequest {
    #[serde(default, deserialize_with = "crate::forms::deserialize_some")]
    pub ipv4: Option<Option<Ipv4Addr>>,
    #[serde(default, deserialize_with = "crate::forms::deserialize_some")]
    pub ipv6: Option<Option<Ipv6Addr>>,
    #[serde(default, deserialize_with = "crate::forms::deserialize_some")]
    pub hostname: Option<Option<DomainName>>,
}

fn check_processable_network_address(req: &PatchPodAddressRequest) -> Result<(), Error> {
    match (req.ipv4, req.ipv6, req.hostname.as_ref()) {
        (None, None, None) => Err(invalid_network_address()),
        _ => Ok(()),
    }
}

impl Into<PodAddressUpdateForm> for PatchPodAddressRequest {
    fn into(self) -> PodAddressUpdateForm {
        PodAddressUpdateForm {
            ipv4: self.ipv4,
            ipv6: self.ipv6,
            hostname: (self.hostname).map(|opt| opt.as_ref().map(ToString::to_string)),
        }
    }
}

pod_config_routes!(
    address,
    req: PatchPodAddressRequest, res: Option<PodAddress>,
    get: get_pod_address_route, get_fn: get_pod_address,
    set: set_pod_address_route, validate_set: {
        check_processable_network_address(&address)?;
    },
);

#[derive(Debug, thiserror::Error)]
#[error("Prose Pod address not initialized.")]
pub struct PodAddressNotInitialized;
impl ErrorCode {
    pub const POD_ADDRESS_NOT_INITIALIZED: Self = Self {
        value: "pod_address_not_initialized",
        http_status: StatusCode::PRECONDITION_FAILED,
        log_level: LogLevel::Warn,
    };
}
impl HttpApiError for PodAddressNotInitialized {
    fn code(&self) -> ErrorCode {
        ErrorCode::POD_ADDRESS_NOT_INITIALIZED
    }
    fn recovery_suggestions(&self) -> Vec<String> {
        vec![format!(
            "Call `PUT {POD_CONFIG_ROUTE}` to initialize it.",
        )]
    }
}
