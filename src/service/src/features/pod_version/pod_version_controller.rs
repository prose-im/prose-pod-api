// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::{PodComponentsVersions, PodVersionService, VersionInfo};

pub fn get_api_version(pod_version_service: &PodVersionService) -> VersionInfo {
    pod_version_service.get_api_version()
}

pub async fn get_pod_version(
    pod_version_service: &PodVersionService,
) -> Result<PodComponentsVersions, anyhow::Error> {
    pod_version_service.get_pod_version().await
}

pub async fn get_server_version(
    pod_version_service: &PodVersionService,
) -> Result<VersionInfo, anyhow::Error> {
    pod_version_service.get_server_version().await
}
