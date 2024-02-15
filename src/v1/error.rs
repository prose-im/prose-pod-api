// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt, io::Cursor};

use log::info;
use migration::sea_orm;
use rocket::http::{ContentType, Status};
use rocket::response::{self, Responder};
use rocket::{Request, Response};
use serde_json::json;

#[derive(Debug)]
pub enum Error {
    /// Unknown database error.
    UnknownDbErr,
    /// SeaORM database error.
    DbErr(sea_orm::DbErr),
    /// Prose Pod not yet initialized.
    PodNotInitialized,
    /// Prose Pod already initialized.
    PodAlreadyInitialized,
}

impl Error {
    /// HTTP status to return for this error.
    pub fn http_status(&self) -> Status {
        match self {
            Error::UnknownDbErr => Status::InternalServerError,
            Error::DbErr(_) => Status::InternalServerError,
            Error::PodNotInitialized => Status::BadRequest,
            Error::PodAlreadyInitialized => Status::Conflict,
        }
    }

    /// User-facing error code (a string for easier understanding).
    pub fn code(&self) -> &str {
        match self {
            Error::UnknownDbErr => "database_error",
            Error::DbErr(_) => "database_error",
            Error::PodNotInitialized => "pod_not_initialized",
            Error::PodAlreadyInitialized => "pod_already_initialized",
        }
    }
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(
        self,
        _: &'r Request<'_>,
    ) -> response::Result<'static> {
        // Log error
        info!("{self}");

        // Construct response
        let body = json!({
            "reason": self.code(),
        })
        .to_string();
        Response::build()
            .status(self.http_status())
            .header(ContentType::JSON)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}

impl fmt::Display for Error {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Error::UnknownDbErr => write!(f, "Unknown database error"),
            Error::DbErr(err) => write!(f, "Database error: {err}"),
            Error::PodNotInitialized => write!(
                f,
                "Prose Pod not initialized. Call `POST {}` to initialize it.",
                uri!(super::init)
            ),
            Error::PodAlreadyInitialized => write!(f, "Prose Pod already initialized."),
        }
    }
}

impl std::error::Error for Error {}
