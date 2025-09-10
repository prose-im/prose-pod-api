// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{body::Bytes, response::IntoResponse};
use axum_extra::{headers::ContentType, TypedHeader};
use service::models::AvatarOwned;

pub struct Avatar(pub AvatarOwned);

impl IntoResponse for Avatar {
    fn into_response(self) -> axum::response::Response {
        (
            TypedHeader(ContentType::from(self.0.media_type().to_owned())),
            Bytes::copy_from_slice(self.0.as_bytes()),
        )
            .into_response()
    }
}
