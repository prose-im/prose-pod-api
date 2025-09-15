// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serdev::Serialize;

use super::Url;

#[derive(Debug, Clone)]
#[derive(Serialize, serdev::Deserialize)]
#[serde(validate = "Self::validate")]
#[repr(transparent)]
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

impl std::ops::Deref for DashboardUrl {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// MARK: - Helpers

fn url_has_no_path(url: &Url) -> bool {
    // NOTE: `make_relative` when called on the same URL returns only the fragment and query.
    let relative_part = url.make_relative(&url);
    relative_part.is_some_and(|s| s.is_empty())
}
