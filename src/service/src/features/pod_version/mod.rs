// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod pod_version_controller;
mod pod_version_service;

use iso8601_timestamp::Timestamp;
use serdev::Serialize;

pub use pod_version_service::*;

#[derive(Debug, Clone)]
#[derive(Serialize, serdev::Deserialize)]
pub struct VersionInfo {
    /// E.g. `"v0.4.0 (2025-01-01)"`
    pub version: String,
    /// E.g. `"v0.4.0"`
    pub tag: String,
    /// E.g. `"2025-01-01"`
    pub build_date: Option<String>,
    /// E.g. `"2025-01-01T22:12:00Z"`
    pub build_timestamp: Option<String>,
    /// E.g. `"e3e6bbb"`
    pub commit_short: Option<String>,
    /// E.g. `"e3e6bbba82fa0d1934990f878c1db376fc35f7d8"`
    pub commit_long: Option<String>,
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
