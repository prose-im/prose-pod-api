// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt::Debug, ops::Deref, sync::Arc};

use super::VersionInfo;

pub use self::{live::LivePodVersionService, r#static::StaticPodVersionService};

#[derive(Debug, Clone)]
pub struct PodVersionService {
    implem: Arc<dyn PodVersionServiceImpl>,
}

impl PodVersionService {
    pub fn new(implem: Arc<dyn PodVersionServiceImpl>) -> Self {
        Self { implem }
    }
}

impl Deref for PodVersionService {
    type Target = dyn PodVersionServiceImpl;

    fn deref(&self) -> &Self::Target {
        self.implem.as_ref()
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PodComponentsVersions {
    pub api: VersionInfo,
    pub server: VersionInfo,
}

#[async_trait::async_trait]
pub trait PodVersionServiceImpl: Debug + Send + Sync {
    fn get_api_version(&self) -> VersionInfo;
    async fn get_server_version(&self) -> Result<VersionInfo, anyhow::Error>;

    async fn get_pod_version(&self) -> Result<PodComponentsVersions, anyhow::Error> {
        Ok(PodComponentsVersions {
            api: self.get_api_version(),
            server: self.get_server_version().await?,
        })
    }
}

mod live {
    use anyhow::Context as _;
    use reqwest::Client as HttpClient;

    use crate::{prosody::ProsodyProseVersion, AppConfig};

    use super::{PodVersionServiceImpl, VersionInfo};

    #[derive(Debug, Clone)]
    pub struct LivePodVersionService {
        mod_prose_version: ProsodyProseVersion,
    }

    impl LivePodVersionService {
        pub fn from_config(config: &AppConfig, http_client: HttpClient) -> Self {
            Self {
                mod_prose_version: ProsodyProseVersion::from_config(config, http_client),
            }
        }
    }

    #[async_trait::async_trait]
    impl PodVersionServiceImpl for LivePodVersionService {
        fn get_api_version(&self) -> VersionInfo {
            super::r#static::API_VERSION.clone()
        }

        async fn get_server_version(&self) -> Result<VersionInfo, anyhow::Error> {
            (self.mod_prose_version.server_version())
                .await
                .context("Cannot get Server version")
        }
    }
}

mod r#static {
    use iso8601_timestamp::Timestamp;
    use lazy_static::lazy_static;

    use super::{PodVersionServiceImpl, VersionInfo};

    lazy_static! {
        pub(super) static ref API_VERSION: VersionInfo = VersionInfo::new(
            non_empty(include_str!("../../../static/api-version/VERSION").trim())
                .unwrap_or("unknown")
                .to_string(),
            non_empty(include_str!("../../../static/api-version/COMMIT").trim())
                .map(|s| s.to_string()),
            non_empty(include_str!("../../../static/api-version/BUILD_TIMESTAMP").trim())
                .and_then(|s| Timestamp::parse(s)),
        );
    }

    #[derive(Debug, Clone)]
    pub struct StaticPodVersionService;

    #[async_trait::async_trait]
    impl PodVersionServiceImpl for StaticPodVersionService {
        fn get_api_version(&self) -> VersionInfo {
            API_VERSION.clone()
        }

        async fn get_server_version(&self) -> Result<VersionInfo, anyhow::Error> {
            unreachable!()
        }
    }

    // MARK: - Utilities

    fn non_empty(s: &'static str) -> Option<&'static str> {
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }
}
