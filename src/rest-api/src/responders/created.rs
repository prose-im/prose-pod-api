// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    http::{header::LOCATION, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};

pub struct Created<T> {
    pub location: HeaderValue,
    pub body: T,
}

impl<T: serde::Serialize> IntoResponse for Created<T> {
    fn into_response(self) -> Response {
        IntoResponse::into_response((
            StatusCode::CREATED,
            [(LOCATION, self.location)],
            axum::Json(self.body),
        ))
    }
}
