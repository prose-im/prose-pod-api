// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{ops::Deref, str::FromStr};

use axum::http::StatusCode;
use cucumber::Parameter;

#[derive(Debug, Parameter)]
#[param(name = "status", regex = r"\d{3}|.+")]
pub struct HTTPStatus(StatusCode);

impl Deref for HTTPStatus {
    type Target = StatusCode;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for HTTPStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(HTTPStatus(match s {
            "BadRequest" | "Bad Request" => StatusCode::BAD_REQUEST,
            "PreconditionFailed" | "Precondition Failed" => StatusCode::PRECONDITION_FAILED,
            "Unauthorized" => StatusCode::UNAUTHORIZED,
            "Forbidden" => StatusCode::FORBIDDEN,
            "Ok" | "OK" => StatusCode::OK,
            "Created" => StatusCode::CREATED,
            "PartialContent" | "Partial Content" => StatusCode::PARTIAL_CONTENT,
            "NoContent" | "No Content" => StatusCode::NO_CONTENT,
            "NotFound" | "Not Found" => StatusCode::NOT_FOUND,
            "InternalServerError" | "Internal Server Error" => StatusCode::INTERNAL_SERVER_ERROR,
            "UnprocessableEntity" | "Unprocessable Entity" => StatusCode::UNPROCESSABLE_ENTITY,
            s => {
                let code = s.parse::<u16>().map_err(|_| {
                    format!("Invalid `HTTPStatus` (use a number if unsupported): {s}")
                })?;
                let status = StatusCode::from_u16(code)
                    .map_err(|_| format!("Invalid `HTTPStatus` (unknown status code): {s}"))?;
                status
            }
        }))
    }
}
