// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fs::File, io::Read};

use axum::http::{header::CONTENT_TYPE, HeaderName, HeaderValue, StatusCode};
use service::AppConfig;

use crate::error::{self, Error};

pub async fn get_prosody_config_lua(
    app_config: AppConfig,
) -> Result<([(HeaderName, HeaderValue); 1], String), Error> {
    let config_file_path = app_config.server.prosody_config_file_path;
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
