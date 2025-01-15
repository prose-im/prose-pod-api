// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::http::StatusCode;
use axum_extra::{headers::ContentType, TypedHeader};
use service::workspace::WorkspaceService;

use crate::error::{self, Error, ErrorCode, HttpApiError, LogLevel};

use super::util::{prose_xmpp_vcard4_to_vcard4_vcard, vcard4_vcard_to_prose_xmpp_vcard4};

pub async fn get_workspace_vcard_route(
    workspace_service: WorkspaceService,
) -> Result<(TypedHeader<ContentType>, String), Error> {
    let vcard = workspace_service.get_workspace_vcard4_vcard().await?;
    Ok((
        TypedHeader(ContentType::from(mime::TEXT_VCARD)),
        vcard.to_string(),
    ))
}

pub async fn set_workspace_vcard_route(
    workspace_service: WorkspaceService,
    body: String,
) -> Result<(TypedHeader<ContentType>, String), Error> {
    let vcards = vcard4::parse(body).map_err(InvalidVCard::from)?;
    if vcards.len() > 1 {
        return Err(Error::from(InvalidVCard("Too many vCards.".to_owned())));
    }
    let vcard4_vcard = vcards
        .first()
        .ok_or(InvalidVCard("Not enough vCards.".to_owned()))?;
    let prose_xmpp_vcard4 =
        vcard4_vcard_to_prose_xmpp_vcard4(vcard4_vcard).map_err(InvalidVCard::from)?;

    workspace_service
        .set_workspace_vcard(&prose_xmpp_vcard4)
        .await?;

    let vcard4_vcard = workspace_service.get_workspace_vcard4_vcard().await?;

    Ok((
        TypedHeader(ContentType::from(mime::TEXT_VCARD)),
        vcard4_vcard.to_string(),
    ))
}

trait WorkspaceServiceExt {
    async fn get_workspace_vcard4_vcard(&self) -> Result<vcard4::Vcard, Error>;
}
impl WorkspaceServiceExt for WorkspaceService {
    async fn get_workspace_vcard4_vcard(&self) -> Result<vcard4::Vcard, Error> {
        let prose_xmpp_vcard4 = self.get_workspace_vcard().await?;
        let vcard4_vcard = prose_xmpp_vcard4_to_vcard4_vcard(prose_xmpp_vcard4).map_err(|err| {
            error::InternalServerError(format!(
                "Could not convert `prose_xmpp` `VCard4` to `vcard4::Vcard`: {err}"
            ))
        })?;
        Ok(vcard4_vcard)
    }
}

// ERRORS

impl ErrorCode {
    pub const INVALID_VCARD: Self = Self {
        value: "invalid_vcard",
        http_status: StatusCode::BAD_REQUEST,
        log_level: LogLevel::Info,
    };
}
#[derive(Debug, thiserror::Error)]
#[error("Invalid vCard4: {0}")]
pub struct InvalidVCard(String);
impl HttpApiError for InvalidVCard {
    fn code(&self) -> ErrorCode {
        ErrorCode::INVALID_VCARD
    }
}
impl From<vcard4::Error> for InvalidVCard {
    fn from(err: vcard4::Error) -> Self {
        Self(err.to_string())
    }
}
