// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::{fmt::Debug, ops::Deref};

use crate::AppConfig;

use super::{License, LicenseValidator};

const LICENSE_FILE_NAME: &'static str = "prose.lic";
const PROSE_CONFIG_DIR: &'static str = "/etc/prose";
const PROSE_DEFAULT_DATA_DIR: &'static str = "/usr/share/prose";

#[derive(Debug, Clone)]
pub struct LicenseService {
    implem: Arc<dyn LicenseServiceImpl>,
}

impl LicenseService {
    pub fn new(implem: Arc<dyn LicenseServiceImpl>) -> Self {
        Self { implem }
    }
}

impl Deref for LicenseService {
    type Target = dyn LicenseServiceImpl;

    fn deref(&self) -> &Self::Target {
        self.implem.as_ref()
    }
}

pub trait LicenseServiceImpl: Debug + Send + Sync {
    fn installed_licenses(&self) -> Vec<License>;
    fn is_license_valid(&self, license: &License) -> bool;
    /// Reload installed licenses.
    fn reload(&self) -> Result<(), NoValidLicense>;

    fn active_license(&self) -> Option<License> {
        for license in self.installed_licenses().into_iter().rev() {
            if self.is_license_valid(&license) {
                return Some(license);
            }
        }
        None
    }
    fn allows_user_count(&self, user_count: u32) -> bool {
        match self.active_license() {
            Some(license) => license.allows_user_count(user_count),
            None => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LiveLicenseService {
    validator: LicenseValidator,
    installed_licenses: Arc<RwLock<Vec<License>>>,
}

#[derive(Debug, Clone, Copy)]
#[derive(thiserror::Error)]
#[error("No valid license.")]
pub struct NoValidLicense;

impl LiveLicenseService {
    pub fn from_config(app_config: &AppConfig) -> Result<Self, NoValidLicense> {
        let validator = LicenseValidator::new(app_config.server_fqdn());

        let installed_licenses = Self::read_installed_licenses(&validator)?;

        Ok(Self {
            validator,
            installed_licenses: Arc::new(RwLock::new(installed_licenses)),
        })
    }

    fn read_installed_licenses(
        validator: &LicenseValidator,
    ) -> Result<Vec<License>, NoValidLicense> {
        fn read_license(
            path: &PathBuf,
            validator: &LicenseValidator,
            licenses: &mut Vec<License>,
            on_missing: impl FnOnce(&PathBuf) -> (),
        ) {
            // Check if file exists (for clearer errors).
            match std::fs::exists(path) {
                Ok(true) => {}
                Ok(false) => return on_missing(path),
                Err(err) => {
                    return tracing::error!(
                        "Could not check if license exists at '{path}': {err}",
                        path = path.display(),
                    )
                }
            }

            // Read the file.
            let bytes = match std::fs::read(path) {
                Ok(bytes) => bytes,
                Err(err) => {
                    return tracing::error!(
                        "Could not read license at '{path}': {err}",
                        path = path.display(),
                    )
                }
            };

            // Parse and validate the file.
            let license = match License::deserialize(bytes, &validator) {
                Ok(license) => license,
                Err(err) => {
                    return tracing::warn!(
                        "Invalid license at '{path}': {err:?}",
                        path = path.display(),
                    )
                }
            };

            // Store the valid license.
            licenses.push(license)
        }

        let custom_license_path = Path::new(PROSE_CONFIG_DIR).join(LICENSE_FILE_NAME);
        let packaged_license_path = Path::new(PROSE_DEFAULT_DATA_DIR).join(LICENSE_FILE_NAME);

        let mut res: Vec<License> = Vec::with_capacity(2);
        read_license(&custom_license_path, validator, &mut res, |_| {});
        read_license(&packaged_license_path, validator, &mut res, |path| {
            tracing::error!("Missing packaged license at path '{path}'. This is an unrecoverable internal error.", path = path.display())
        });

        match res.is_empty() {
            false => Ok(res),
            true => Err(NoValidLicense),
        }
    }
}

impl LicenseServiceImpl for LiveLicenseService {
    fn installed_licenses(&self) -> Vec<License> {
        self.installed_licenses.read().unwrap().clone()
    }
    fn is_license_valid(&self, license: &License) -> bool {
        license.is_valid(&self.validator)
    }
    fn reload(&self) -> Result<(), NoValidLicense> {
        let licenses = Self::read_installed_licenses(&self.validator)?;
        *self.installed_licenses.write().unwrap() = licenses;
        Ok(())
    }
}
