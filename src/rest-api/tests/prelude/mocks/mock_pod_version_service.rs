// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::{Arc, RwLock};

use service::pod_version::{PodVersionServiceImpl, StaticPodVersionService, VersionInfo};

#[derive(Debug, Clone, Default)]
pub struct MockPodVersionService {
    data: Arc<RwLock<MockPodVersionServiceData>>,
}

#[derive(Debug, Clone, Default)]
pub struct MockPodVersionServiceData {
    pub api: Option<VersionInfo>,
    pub server: Option<VersionInfo>,
}

impl MockPodVersionService {
    #[allow(unused)]
    pub fn set_api_version(&self, version: VersionInfo) {
        self.data.write().unwrap().api = Some(version);
    }
    #[allow(unused)]
    pub fn set_server_version(&self, version: VersionInfo) {
        self.data.write().unwrap().server = Some(version);
    }
}

#[async_trait::async_trait]
impl PodVersionServiceImpl for MockPodVersionService {
    fn get_api_version(&self) -> VersionInfo {
        (self.data.read().unwrap().api.clone())
            .unwrap_or_else(|| StaticPodVersionService.get_api_version())
    }
    async fn get_server_version(&self) -> Result<VersionInfo, anyhow::Error> {
        (self.data.read().unwrap().server.clone())
            .ok_or(anyhow::anyhow!("Mock Server version not set"))
    }
}
