// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{extract::Request, http::uri};

/// The Prose Pod API should support repeated query parameters using
/// non-indexed brackets (e.g. `jids[]=foo@example.org&jids[]=bar@example.org`).
/// Both [`axum::extract::Query`] and [`axum_extra::extract::Query`] don't support it,
/// but by removing the brackets, we can leverage [`axum_extra::extract::Query`]'s
/// support for multi-value items and achieve the desired behavior.
pub fn rename_bracketed_query_param_names(req: Request) -> Request {
    let (mut head, body) = req.into_parts();
    let uri = head.uri.clone();

    let query = uri.query().unwrap_or_default();
    let new_query = query.replace("[]=", "=");

    head.uri = uri::Builder::from(uri.clone())
        .path_and_query(format!("{path}?{new_query}", path = uri.clone().path()))
        .build()
        .unwrap();

    Request::from_parts(head, body)
}
