// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{ops::Deref, str::FromStr};

use cucumber::Parameter;
use rocket::http::Status;

#[derive(Debug, Parameter)]
#[param(name = "status", regex = r"\d{3}|.+")]
pub struct HTTPStatus(Status);

impl Deref for HTTPStatus {
    type Target = Status;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for HTTPStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(HTTPStatus(match s {
            "BadRequest" | "Bad Request" => Status::BadRequest,
            "Unauthorized" => Status::Unauthorized,
            "Forbidden" => Status::Forbidden,
            "Ok" | "OK" => Status::Ok,
            "Created" => Status::Created,
            "PartialContent" | "Partial Content" => Status::PartialContent,
            "NoContent" | "No Content" => Status::NoContent,
            "NotFound" | "Not Found" => Status::NotFound,
            "InternalServerError" | "Internal Server Error" => Status::InternalServerError,
            s => {
                if let Some(status) = s.parse::<u16>().ok().and_then(Status::from_code) {
                    status
                } else {
                    return Err(format!("Invalid `HTTPStatus`: {s}"));
                }
            }
        }))
    }
}
