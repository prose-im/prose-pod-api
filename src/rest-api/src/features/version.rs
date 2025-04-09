// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{routing::get, Json};
use iso8601_timestamp::Timestamp;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref API_VERSION: VersionInfo = VersionInfo::new(
        non_empty(include_str!("../../static/api-version/VERSION").trim())
            .unwrap_or("unknown")
            .to_string(),
        non_empty(include_str!("../../static/api-version/COMMIT").trim()).map(|s| s.to_string()),
        non_empty(include_str!("../../static/api-version/BUILD_TIMESTAMP").trim())
            .and_then(|s| Timestamp::parse(s)),
    );
}

pub(super) fn router() -> axum::Router {
    axum::Router::new()
        .route("/version", get(get_api_version_route))
        .route("/v1/version", get(get_api_version_route))
        .route("/pod/version", get(get_pod_version_route))
        .route("/v1/pod/version", get(get_pod_version_route))
}

async fn get_api_version_route() -> Json<VersionInfo> {
    Json(API_VERSION.clone())
}

async fn get_pod_version_route() -> Json<PodComponentsVersions> {
    Json(PodComponentsVersions {
        api: API_VERSION.clone(),
    })
}

#[derive(Serialize, Deserialize)]
struct PodComponentsVersions {
    api: VersionInfo,
}

#[derive(Clone, Serialize, Deserialize)]
struct VersionInfo {
    /// E.g. `"v0.4.0 (2025-01-01)"`
    version: String,
    /// E.g. `"v0.4.0"`
    tag: String,
    /// E.g. `"2025-01-01"`
    build_date: Option<String>,
    /// E.g. `"2025-01-01T22:12:00Z"`
    build_timestamp: Option<String>,
    /// E.g. `"e3e6bbb"`
    commit_short: Option<String>,
    /// E.g. `"e3e6bbba82fa0d1934990f878c1db376fc35f7d8"`
    commit_long: Option<String>,
}

impl VersionInfo {
    fn new(tag: String, commit_hash: Option<String>, build_timestamp: Option<Timestamp>) -> Self {
        let build_date = build_timestamp.map(|timestamp| timestamp.date().to_string());
        let commit_short = commit_hash.as_ref().map(|hash| hash[..7].to_string());
        let version = if let Some(ref date) = build_date {
            format!("{tag} ({date})")
        } else {
            format!("{tag}")
        };
        Self {
            version,
            tag,
            build_date,
            build_timestamp: build_timestamp.as_ref().map(Timestamp::to_string),
            commit_short,
            commit_long: commit_hash,
        }
    }
}

// UTILITIES

fn non_empty(s: &'static str) -> Option<&'static str> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
