// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use base64::{prelude::BASE64_STANDARD, Engine};
use serdev::Serialize;

use crate::members::UserRepository;

use super::LicensingService;

#[derive(Debug)]
#[derive(Serialize)]
pub struct GetLicenseResponse {
    pub id: String,
    pub name: String,
    pub user_limit: u32,
    pub expiry: Option<iso8601_timestamp::Timestamp>,
    pub ttl_ms: Option<u128>,
}

pub async fn get_license(licensing_service: &LicensingService) -> GetLicenseResponse {
    use std::time::SystemTime;

    use iso8601_timestamp::Timestamp as IsoTimestamp;

    let license = (licensing_service.installed_licenses().last())
        .expect("The Community license should always be installed")
        .to_owned();

    let expiry = license.expiry();

    GetLicenseResponse {
        id: BASE64_STANDARD.encode(license.id()),
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

#[derive(Debug)]
#[derive(Serialize)]
pub struct GetLicensingStatusResponse {
    pub license: GetLicenseResponse,
    pub user_count: u32,
    pub remaining_seats: u32,
}

pub async fn get_licensing_status(
    licensing_service: &LicensingService,
    user_repository: &UserRepository,
) -> Result<GetLicensingStatusResponse, anyhow::Error> {
    let license = self::get_license(licensing_service).await;
    let user_count = user_repository.users_stats(None).await?.count as u32;
    let remaining_seats = (license.user_limit)
        .checked_sub(user_count)
        .unwrap_or_default();

    Ok(GetLicensingStatusResponse {
        license,
        user_count,
        remaining_seats,
    })
}
