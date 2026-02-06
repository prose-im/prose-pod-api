// prose-pod-api
//
// Copyright: 2026, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Context;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderName};
use axum::response::Response;
use axum::routing::any;

use crate::error::Error;
use crate::AppState;

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/cloud-api-proxy/{*path}", any(server_api_proxy))
        .with_state(app_state)
}

async fn server_api_proxy(
    State(app_state): State<AppState>,
    Path(request_path): Path<String>,
    request_method: axum::http::Method,
    request_headers: HeaderMap,
    request_body: Body,
) -> Result<Response, Error> {
    // Construct upstream request.
    let upstream_request = {
        let upstream_url = {
            assert!(!request_path.ends_with('/'));

            let ref server_api_url = app_state.app_config.server_api_url();
            assert!(!server_api_url.ends_with('/'));

            format!("{server_api_url}/cloud-api-proxy/{request_path}")
        };

        let mut req = app_state.http_client.request(request_method, upstream_url);

        for (name, value) in request_headers.iter() {
            if should_forward_header(name) {
                req = req.header(name, value);
            }
        }

        req.body(reqwest::Body::wrap_stream(request_body.into_data_stream()))
    };

    let upstream_response = upstream_request
        .send()
        .await
        .context("Failed proxying to Prose Pod Server")?;

    let response = {
        let response_status = upstream_response.status();
        let response_headers = upstream_response.headers().to_owned();

        let stream = upstream_response.bytes_stream();

        let body = Body::from_stream(stream);

        let mut res = Response::new(body);
        *res.status_mut() = response_status;

        for (name, value) in response_headers.iter() {
            if should_forward_header(name) {
                res.headers_mut().append(name, value.clone());
            }
        }

        res
    };

    Ok(response)
}

fn should_forward_header(name: &HeaderName) -> bool {
    !is_hop_by_hop(name) && !has_source_ip(name)
}

/// Whether or not a header is a hop-by-hop header.
///
/// Hop-by-hop headers, as defined by [RFC 2616, section 13.5.1], should not be
/// forwarded by proxies. Otherwise, attackers can abuse those headers
/// (see [“hop-by-hop headers” on HackTricks]).
///
/// [RFC 2616, section 13.5.1]: https://datatracker.ietf.org/doc/html/rfc2616#section-13.5.1
/// [“hop-by-hop headers” on HackTricks]: https://book.hacktricks.wiki/en/pentesting-web/abusing-hop-by-hop-headers.html
fn is_hop_by_hop(name: &HeaderName) -> bool {
    matches!(
        name.as_str().to_ascii_lowercase().as_str(),
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailers"
            | "transfer-encoding"
            | "upgrade"
    )
}

fn has_source_ip(name: &HeaderName) -> bool {
    matches!(
        name.as_str().to_ascii_lowercase().as_str(),
        "x-forwarded-for" | "x-real-ip"
    )
}
