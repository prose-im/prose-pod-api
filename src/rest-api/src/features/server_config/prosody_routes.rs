// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fs::File, io::Read};

use axum::{
    http::{header::CONTENT_TYPE, HeaderName, HeaderValue, StatusCode},
    response::NoContent,
    Json,
};
use axum_extra::either::Either;
use service::{prosody::ProsodyOverrides, xmpp::ServerManager, AppConfig};

use crate::{
    error::{self, Error},
    features::init::ServerConfigNotInitialized,
};

use super::guards::Lua;

pub async fn get_prosody_config_lua_route(
    app_config: AppConfig,
) -> Result<([(HeaderName, HeaderValue); 1], String), Error> {
    let config_file_path = app_config.prosody_ext.config_file_path;
    let mut file = File::open(&config_file_path).map_err(|e| error::HTTPStatus {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        body: format!(
            "Cannot open Prosody config file at `{path}`: {e}",
            path = &config_file_path.display(),
        ),
    })?;

    let mut prosody_config = String::new();
    file.read_to_string(&mut prosody_config)
        .map_err(|e| error::HTTPStatus {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: format!(
                "Cannot read Prosody config file at `{path}`: {e}",
                path = &config_file_path.display(),
            ),
        })?;

    return Ok((
        [(CONTENT_TYPE, HeaderValue::from_static("text/x-lua"))],
        prosody_config,
    ));
}

// PROSODY OVERRIDES (JSON)

pub async fn set_prosody_overrides_route(
    server_manager: ServerManager,
    Json(overrides): Json<ProsodyOverrides>,
) -> Result<Json<Option<ProsodyOverrides>>, Error> {
    let res = match server_manager.set_prosody_overrides(overrides).await {
        Ok(model) => Ok(Json(
            model
                .prosody_overrides
                // NOTE: It’s safe enough to force unwrap here as the
                //   JSON data should be exactly the user’s request.
                .map(|json| serde_json::from_value(json).unwrap()),
        )),
        Err(service::util::Either::E1(err)) => Err(Error::from(error::HTTPStatus {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: format!("Prosody overrides cannot be stored in database: {err}"),
        })),
        Err(service::util::Either::E2(err)) => Err(Error::from(err)),
    };

    server_manager.reload_current().await?;

    res
}

pub async fn get_prosody_overrides_route(
    server_manager: ServerManager,
) -> Result<Either<Json<ProsodyOverrides>, NoContent>, Error> {
    match server_manager.get_prosody_overrides().await {
        Ok(Some(Some(overrides))) => Ok(Either::E1(Json(overrides))),
        Ok(Some(None)) => Ok(Either::E2(NoContent)),
        Ok(None) => Err(Error::from(ServerConfigNotInitialized)),
        Err(service::util::Either::E1(err)) => Err(Error::from(err)),
        Err(service::util::Either::E2(err)) => Err(Error::from(error::HTTPStatus {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: format!("Prosody overrides stored in database cannot be read. To fix this, call `PUT /v1/server/config/prosody-overrides` with a new value. You can `GET /v1/server/config/prosody-overrides` first to see what the stored value was. Error: {err}"),
        })),
    }
}

pub async fn delete_prosody_overrides_route(
    server_manager: ServerManager,
) -> Result<NoContent, Error> {
    match server_manager.reset_prosody_overrides().await {
        Ok(_) => Ok(NoContent),
        Err(err) => Err(Error::from(err)),
    }
}

// PROSODY OVERRIDES (RAW)

pub async fn set_prosody_overrides_raw_route(
    server_manager: ServerManager,
    Lua(overrides): Lua,
) -> Result<Lua, Error> {
    let res = match server_manager.set_prosody_overrides_raw(overrides).await {
        Ok(model) => Ok(model
            .prosody_overrides_raw
            .map(Lua)
            // NOTE: It’s safe enough to force unwrap here as the
            //   Lua data should be exactly the user’s request.
            .unwrap()),
        Err(err) => Err(Error::from(err)),
    };

    server_manager.reload_current().await?;

    res
}

pub async fn get_prosody_overrides_raw_route(
    server_manager: ServerManager,
) -> Result<Either<Lua, NoContent>, Error> {
    match server_manager.get_prosody_overrides_raw().await {
        Ok(Some(overrides)) => Ok(Either::E1(Lua(overrides))),
        Ok(None) => Ok(Either::E2(NoContent)),
        Err(err) => Err(Error::from(err)),
    }
}

pub async fn delete_prosody_overrides_raw_route(
    server_manager: ServerManager,
) -> Result<NoContent, Error> {
    match server_manager.reset_prosody_overrides().await {
        Ok(_) => Ok(NoContent),
        Err(err) => Err(Error::from(err)),
    }
}
