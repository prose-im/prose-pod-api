// prose-pod-api
//
// Copyright: 2026, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Context;
use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderName, Uri};
use axum::response::Response;
use axum::routing::any;

use crate::error::Error;
use crate::AppState;

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/cloud-api-proxy/{*path}", any(server_api_proxy))
        .route("/prose-files-proxy/{*path}", any(server_api_proxy))
        .with_state(app_state)
}

async fn server_api_proxy(
    State(ref app_state): State<AppState>,
    request_uri: Uri,
    request_method: axum::http::Method,
    request_headers: HeaderMap,
    request_body: Body,
) -> Result<Response, Error> {
    let server_api_url = app_state.app_config.server_api_url();

    proxy(
        server_api_url.as_str(),
        "/",
        app_state,
        request_uri,
        request_method,
        request_headers,
        request_body,
    )
    .await
}

async fn proxy(
    destination: &str,
    path_prefix: &'static str,
    app_state: &AppState,
    // NOTE: We have to use the full URI here instead of matching `{*path}`
    //   as it’s not present when this handler is called from
    //   `proxy_analytics_event`. To avoid discrepancies, let’s just read the
    //   URI all the time.
    request_uri: Uri,
    request_method: axum::http::Method,
    request_headers: HeaderMap,
    request_body: Body,
) -> Result<Response, Error> {
    // Construct upstream request.
    let upstream_request = {
        let upstream_url = {
            let request_path = request_uri
                .path_and_query()
                .expect("`proxy` shouldn’t be called from a route with no path")
                .path()
                .strip_prefix(path_prefix)
                .expect(&format!(
                    "`proxy` shouldn’t be called from a route not prefixed by `{path_prefix}`"
                ));
            assert!(!request_path.starts_with('/'));

            if destination.ends_with('/') {
                format!("{destination}{request_path}")
            } else {
                format!("{destination}/{request_path}")
            }
        };

        let mut req = app_state.http_client.request(request_method, &upstream_url);

        for (name, value) in request_headers.iter() {
            if should_forward_header(name) {
                req = req.header(name, value);
            }
        }

        tracing::debug!("Proxying `{request_uri}` to `{upstream_url}`: {req:#?}");

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
        name.as_str(),
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
    matches!(name.as_str(), "x-forwarded-for" | "x-real-ip")
}
