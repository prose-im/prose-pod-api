// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod get_workspace;
mod guards;
pub mod util;
mod workspace_accent_color;
mod workspace_icon;
mod workspace_name;
mod workspace_vcard;

use axum::body::Body;
use axum::extract::{FromRequest, FromRequestParts, Request, State};
use axum::http::request::Parts;
use axum::middleware::from_extractor_with_state;
use axum::routing::{get, put, MethodRouter};
use axum::Json;
use axum_extra::either::Either;
use axum_extra::handler::HandlerCallWithExtractors as _;
use axum_extra::headers::ContentType;
use axum_extra::TypedHeader;
use mime::Mime;

use crate::error::{self, Error};
use crate::responders::Created;
use crate::util::content_type_or::{with_content_type, TextVCard};
use crate::AppState;

pub use self::get_workspace::*;
pub use self::workspace_accent_color::*;
pub use self::workspace_icon::*;
pub use self::workspace_name::*;
pub use self::workspace_vcard::*;

use super::auth::guards::IsAdmin;
use super::init::{init_workspace_route, InitWorkspaceResponse};

pub const WORKSPACE_ROUTE: &'static str = "/v1/workspace";

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .nest(
            WORKSPACE_ROUTE,
            axum::Router::new()
                .route(
                    "/",
                    MethodRouter::new()
                        .get(
                            with_content_type::<TextVCard, _>(get_workspace_vcard_route)
                                .or(get_workspace_route),
                        )
                        .put(put_workspace),
                )
                .route("/accent-color", get(get_workspace_accent_color_route))
                .route("/icon", get(get_workspace_icon_route))
                .route("/name", get(get_workspace_name_route))
                .merge(
                    axum::Router::new()
                        .route("/accent-color", put(set_workspace_accent_color_route))
                        .route("/icon", put(set_workspace_icon_route))
                        .route("/name", put(set_workspace_name_route))
                        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone())),
                ),
        )
        .with_state(app_state)
}

async fn put_workspace(
    init_service: service::init::InitService,
    app_config: service::AppConfig,
    secrets_store: service::secrets::SecretsStore,
    xmpp_service: service::xmpp::XmppServiceInner,
    server_config: service::server_config::ServerConfig,
    State(state): State<AppState>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    workspace_service: service::workspace::WorkspaceService,
    mut parts: Parts,
    req: Request<Body>,
) -> Result<Either<(TypedHeader<ContentType>, String), Created<InitWorkspaceResponse>>, Error> {
    if Mime::from(content_type) == mime::TEXT_VCARD {
        IsAdmin::from_request_parts(&mut parts, &state).await?;
        let body = String::from_request(req, &state)
            .await
            .map_err(|err| error::BadRequest {
                reason: err.to_string(),
            })?;
        set_workspace_vcard_route(workspace_service, body)
            .await
            .map(Either::E1)
    } else {
        let body = Json::from_request(req, &state)
            .await
            .map_err(|err| error::BadRequest {
                reason: err.to_string(),
            })?;
        init_workspace_route(
            init_service,
            app_config,
            secrets_store,
            xmpp_service,
            server_config,
            body,
        )
        .await
        .map(Either::E2)
    }
}
