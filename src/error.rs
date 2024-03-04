// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt, io::Cursor};

use log::info;
use rocket::http::{ContentType, Header, Status};
use rocket::response::{self, Responder};
use rocket::{Request, Response};
use serde_json::json;
use service::sea_orm;

use crate::server_ctl;

#[derive(Debug)]
pub enum Error {
    /// Feature not implemented yet.
    NotImplemented { feature: String },
    /// Internal server error.
    /// Use it only when a nearly-impossible code path is taken.
    InternalServerError { reason: String },
    /// Unauthorized
    Unauthorized,
    /// Unknown database error.
    UnknownDbErr,
    /// SeaORM database error.
    DbErr(sea_orm::DbErr),
    /// Prose Pod not yet initialized.
    PodNotInitialized,
    /// Prose Pod already initialized.
    PodAlreadyInitialized,
    /// ServerCtl fail (e.g. execution of `prosodyctl` failed).
    ServerCtlErr(server_ctl::Error),
}

impl Error {
    /// HTTP status to return for this error.
    pub fn http_status(&self) -> Status {
        match self {
            Self::NotImplemented { .. } => Status::NotImplemented,
            Self::InternalServerError { .. } => Status::InternalServerError,
            Self::Unauthorized => Status::Unauthorized,
            Self::UnknownDbErr => Status::InternalServerError,
            Self::DbErr(_) => Status::InternalServerError,
            Self::PodNotInitialized => Status::BadRequest,
            Self::PodAlreadyInitialized => Status::Conflict,
            Self::ServerCtlErr(_) => Status::InternalServerError,
        }
    }

    /// User-facing error code (a string for easier understanding).
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotImplemented { .. } => "not_implemented",
            Self::InternalServerError { .. } => "internal_server_error",
            Self::Unauthorized => "unauthorized",
            Self::UnknownDbErr => "database_error",
            Self::DbErr(_) => "database_error",
            Self::PodNotInitialized => "pod_not_initialized",
            Self::PodAlreadyInitialized => "pod_already_initialized",
            Self::ServerCtlErr(_) => "internal_server_error",
        }
    }

    pub fn add_headers(&self, response: &mut Response<'_>) {
        match self {
            Self::Unauthorized => {
                response.set_header(Header::new("WWW-Authenticate", "value"));
            }
            _ => {}
        }
    }
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        // Log error
        info!("{self}");

        // Construct response
        let body = json!({
            "reason": self.code(),
        })
        .to_string();
        let mut response = Response::build()
            .status(self.http_status())
            .header(ContentType::JSON)
            .sized_body(body.len(), Cursor::new(body))
            .ok()?;

        self.add_headers(&mut response);

        Ok(response)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotImplemented { feature } => write!(f, "Feature not implemented: {feature}"),
            Self::InternalServerError { reason } => write!(f, "Internal server error: {reason}"),
            Self::Unauthorized => write!(f, "Unauthorized"),
            Self::UnknownDbErr => write!(f, "Unknown database error"),
            Self::DbErr(err) => write!(f, "Database error: {err}"),
            Self::PodNotInitialized => write!(
                f,
                "Prose Pod not initialized. Call `POST {}` to initialize it.",
                uri!(crate::v1::init)
            ),
            Self::PodAlreadyInitialized => write!(f, "Prose Pod already initialized."),
            Self::ServerCtlErr(err) => write!(f, "ServerCtl error: {err}"),
        }
    }
}

impl std::error::Error for Error {}
