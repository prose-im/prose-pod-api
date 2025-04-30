// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    http::{header::ACCEPT, HeaderMap, HeaderValue},
    response::NoContent,
    Json,
};
use axum_extra::{either::Either, headers::ContentType, TypedHeader};
use base64::{engine::general_purpose, Engine as _};
use mime::Mime;
use service::{workspace::WorkspaceService, xmpp::xmpp_service::Avatar};

use crate::{
    error::{self, Error},
    util::detect_image_mime_type,
};

pub async fn get_workspace_icon_route(
    workspace_service: WorkspaceService,
) -> Result<Either<(TypedHeader<ContentType>, String), NoContent>, Error> {
    Ok(match workspace_service.get_workspace_icon().await? {
        Some(icon) => Either::E1((TypedHeader(ContentType::from(icon.mime)), icon.base64)),
        None => Either::E2(NoContent),
    })
}
pub async fn get_workspace_icon_json_route(
    workspace_service: WorkspaceService,
) -> Result<Json<Option<Avatar>>, Error> {
    let icon = workspace_service.get_workspace_icon().await?;
    Ok(Json(icon))
}

pub async fn set_workspace_icon_route(
    workspace_service: WorkspaceService,
    content_type: Option<TypedHeader<ContentType>>,
    headers: HeaderMap,
    base64: String,
) -> Result<Either<(TypedHeader<ContentType>, String), Json<Avatar>>, Error> {
    let mime = content_type.map(|TypedHeader(ct)| Mime::from(ct));

    let image_data =
        general_purpose::STANDARD
            .decode(&base64)
            .map_err(|err| error::BadRequest {
                reason: format!("Image data should be Base64-encoded. Error: {err}"),
            })?;

    let mime =
        detect_image_mime_type(&base64, mime).ok_or(Error::from(error::UnsupportedMediaType {
            comment: format!(
                "Supported content types: {}.",
                [
                    mime::IMAGE_PNG.to_string(),
                    mime::IMAGE_GIF.to_string(),
                    mime::IMAGE_JPEG.to_string(),
                ]
                .join(", ")
            ),
        }))?;

    workspace_service
        .set_workspace_icon(image_data, &mime)
        .await?;

    if headers.get(ACCEPT)
        == Some(&HeaderValue::from_str(&mime::APPLICATION_JSON.to_string()).unwrap())
    {
        Ok(Either::E2(Json(Avatar { base64, mime })))
    } else {
        Ok(Either::E1((TypedHeader(ContentType::from(mime)), base64)))
    }
}
