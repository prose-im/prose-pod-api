// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use base64::Engine;
use serdev::Serialize;

use super::LicenseService;

#[derive(Serialize)]
pub struct GetLicenseResponse {
    pub id: String,
    pub name: String,
    pub user_limit: u32,
    pub expiry: Option<iso8601_timestamp::Timestamp>,
    pub ttl_ms: Option<u128>,
}

pub async fn get_license(license_service: &LicenseService) -> GetLicenseResponse {
    use std::time::SystemTime;

    use base64::engine::general_purpose::STANDARD_NO_PAD as base64;
    use iso8601_timestamp::Timestamp as IsoTimestamp;

    let license = (license_service.installed_licenses().last())
        .expect("The Community license should always be installed")
        .to_owned();

    let expiry = license.expiry();

    GetLicenseResponse {
        id: base64.encode(license.id()),
        name: license.name().to_owned(),
        user_limit: license.user_limit(),
        expiry: expiry.map(IsoTimestamp::from),
        ttl_ms: expiry.map(|t| {
            t.duration_since(SystemTime::now())
                .unwrap_or_default()
                .as_millis()
        }),
    }
}
