// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use base64::{prelude::BASE64_STANDARD, Engine};
use sea_orm::DatabaseConnection;
use serdev::Serialize;

use crate::members::MemberRepository;

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

    use iso8601_timestamp::Timestamp as IsoTimestamp;

    let license = (license_service.installed_licenses().last())
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

#[derive(Serialize)]
pub struct GetLicensingStatusResponse {
    pub license: GetLicenseResponse,
    pub user_count: u64,
    pub remaining_seats: u64,
}

pub async fn get_licensing_status(
    license_service: &LicenseService,
    db: &DatabaseConnection,
) -> Result<GetLicensingStatusResponse, anyhow::Error> {
    let license = self::get_license(license_service).await;
    let user_count = MemberRepository::count(db).await?;
    let remaining_seats = (license.user_limit as u64)
        .checked_sub(user_count)
        .unwrap_or_default();

    Ok(GetLicensingStatusResponse {
        license,
        user_count,
        remaining_seats,
    })
}
