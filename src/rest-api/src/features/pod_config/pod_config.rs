// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::{Ipv4Addr, Ipv6Addr};

use axum::{
    extract::State,
    http::{header::IF_MATCH, HeaderValue, StatusCode},
    Json,
};
use axum_extra::{headers::IfMatch, TypedHeader};
use hickory_resolver::Name as DomainName;
use serde::{Deserialize, Serialize};
use service::{
    models::Url,
    pod_config::{NetworkAddressCreateForm, PodConfig, PodConfigCreateForm, PodConfigRepository},
};

use crate::{
    error::{self, Error, ErrorCode, HttpApiError, LogLevel, PreconditionRequired},
    responders::Created,
    AppState,
};

use super::POD_CONFIG_ROUTE;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct InitPodAddressRequest {
    pub ipv4: Option<Ipv4Addr>,
    pub ipv6: Option<Ipv6Addr>,
    pub hostname: Option<DomainName>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct InitPodConfigRequest {
    pub address: InitPodAddressRequest,
    pub dashboard_url: Option<Url>,
}

impl Into<PodConfigCreateForm> for InitPodConfigRequest {
    fn into(self) -> PodConfigCreateForm {
        PodConfigCreateForm {
            address: NetworkAddressCreateForm {
                ipv4: self.address.ipv4,
                ipv6: self.address.ipv6,
                hostname: self.address.hostname.as_ref().map(ToString::to_string),
            },
            dashboard_url: self.dashboard_url,
        }
    }
}

pub fn invalid_network_address() -> Error {
    Error::from(error::HTTPStatus {
        status: StatusCode::UNPROCESSABLE_ENTITY,
        body: "You must pass either an IPv4, an IPv6 or a hostname.".to_string(),
    })
}
fn check_processable_network_address(req: &InitPodAddressRequest) -> Result<(), Error> {
    match (req.ipv4, req.ipv6, req.hostname.as_ref()) {
        (None, None, None) => Err(invalid_network_address()),
        _ => Ok(()),
    }
}
pub(super) fn check_url_has_no_path(url: &Option<Url>) -> Result<(), Error> {
    if let Some(url) = url {
        // NOTE: `make_relative` when called on the same URL returns only the fragment and query.
        let relative_part = url.make_relative(&url);
        if relative_part.is_none_or(|s| !s.is_empty()) {
            return Err(Error::from(error::HTTPStatus {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                body: "The URL you passed contains a fragment or query.".to_string(),
            }));
        }
    }
    Ok(())
}

pub async fn init_pod_config_route(
    State(AppState { db, .. }): State<AppState>,
    Json(req): Json<InitPodConfigRequest>,
) -> Result<Created<PodConfig>, Error> {
    check_processable_network_address(&req.address)?;
    check_url_has_no_path(&req.dashboard_url)?;

    if PodConfigRepository::get(&db).await?.is_some() {
        Err(Error::from(PodConfigAlreadyInitialized))
    } else {
        let model = PodConfigRepository::create(&db, req).await?;
        Ok(Created {
            location: HeaderValue::from_static(POD_CONFIG_ROUTE),
            body: PodConfig::from(model),
        })
    }
}

pub async fn is_pod_config_initialized_route(
    State(AppState { ref db, .. }): State<AppState>,
    TypedHeader(if_match): TypedHeader<IfMatch>,
) -> Result<StatusCode, Error> {
    if if_match != IfMatch::any() {
        Err(Error::from(PreconditionRequired {
            comment: format!("Missing header: '{IF_MATCH}'."),
        }))
    } else if PodConfigRepository::is_initialized(db).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::PRECONDITION_FAILED)
    }
}

impl ErrorCode {
    const POD_CONFIG_ALREADY_INITIALIZED: Self = Self {
        value: "pod_config_already_initialized",
        http_status: StatusCode::CONFLICT,
        log_level: LogLevel::Info,
    };
}
#[derive(Debug, thiserror::Error)]
#[error("Pod config already initialized.")]
pub struct PodConfigAlreadyInitialized;
impl HttpApiError for PodConfigAlreadyInitialized {
    fn code(&self) -> ErrorCode {
        ErrorCode::POD_CONFIG_ALREADY_INITIALIZED
    }
}

impl ErrorCode {
    pub const POD_CONFIG_NOT_INITIALIZED: Self = Self {
        value: "pod_config_not_initialized",
        http_status: StatusCode::PRECONDITION_FAILED,
        log_level: LogLevel::Warn,
    };
}
#[derive(Debug, thiserror::Error)]
#[error("Pod config not initialized.")]
pub struct PodConfigNotInitialized;
impl HttpApiError for PodConfigNotInitialized {
    fn code(&self) -> ErrorCode {
        ErrorCode::POD_CONFIG_NOT_INITIALIZED
    }
}

pub async fn get_pod_config_route(
    State(AppState { db, .. }): State<AppState>,
) -> Result<Json<PodConfig>, Error> {
    let model = PodConfigRepository::get(&db).await?;
    let res = model.map(PodConfig::from).unwrap_or_default();
    Ok(Json(res))
}
