// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    body::{to_bytes, Body},
    http::header::CONTENT_TYPE,
    response::{IntoResponse as _, Response},
};
use tokio::runtime::Handle;

use crate::error::{self, Error};

pub fn error_catcher(res: Response) -> Response {
    if !res.status().is_success() {
        if let Some(content_type) = res.headers().get(CONTENT_TYPE) {
            // NOTE: Axum errors are encoded as `text/plain; charset=utf-8`.
            if content_type.to_str().unwrap().starts_with("text/plain") {
                fn body_to_string(body: Body) -> String {
                    tokio::task::block_in_place(move || {
                        Handle::current().block_on(async move {
                            let bytes =
                                to_bytes(body, usize::MAX).await.unwrap_or("<error>".into());
                            String::from_utf8_lossy(&bytes).to_string()
                        })
                    })
                }
                return Error::from(error::HTTPStatus {
                    status: res.status(),
                    body: body_to_string(res.into_body()),
                })
                .into_response();
            }
        }
    }
    res
}
