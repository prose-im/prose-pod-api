// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::models::Url;

use super::PodAddress;

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PodConfig {
    pub address: PodAddress,
    pub dashboard_url: DashboardUrl,
}

impl PodConfig {
    pub fn dashboard_url(&self) -> Url {
        self.dashboard_url.0.clone()
    }
}

// MARK: - Dashboard URL

#[derive(Debug, Clone)]
#[derive(serdev::Serialize, serdev::Deserialize)]
#[serde(validate = "Self::validate")]
pub struct DashboardUrl(Url);

impl DashboardUrl {
    pub fn new(url: Url) -> anyhow::Result<Self> {
        let res = Self(url);
        res.validate().map_err(|str| anyhow::Error::msg(str))?;
        Ok(res)
    }

    fn validate(&self) -> Result<(), &'static str> {
        if url_has_no_path(&self.0) {
            Ok(())
        } else {
            Err("The Dashboard URL contains a fragment or query.")
        }
    }
}

// MARK: - Helpers

fn url_has_no_path(url: &Url) -> bool {
    // NOTE: `make_relative` when called on the same URL returns only the fragment and query.
    let relative_part = url.make_relative(&url);
    relative_part.is_some_and(|s| s.is_empty())
}
