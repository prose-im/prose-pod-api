// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use biscuit::macros::*;
use hickory_proto::rr::domain::Name as DomainName;
use service::licensing::{
    License, LicenseValidator, LicensingServiceImpl, NoValidLicense, ValidationError,
};

use super::prelude::*;

#[derive(Debug)]
pub struct MockLicensingService {
    installed_licenses: Arc<RwLock<Vec<License>>>,
    valid_licenses: Arc<RwLock<HashSet<Vec<u8>>>>,
    pub(crate) validator: LicenseValidator,
    pub(crate) license_signing_key: biscuit::KeyPair,
}

impl MockLicensingService {
    pub fn new(server_domain: DomainName) -> Self {
        let validator = LicenseValidator::new(server_domain);
        let license_signing_key = biscuit::KeyPair::new();

        let api_version = env!("CARGO_PKG_VERSION");
        let biscuit = biscuit!(
            r#"
            name({name});
            user_limit({user_limit});

            // Tie license to a specific version to prevent reuse.
            right("api_version", {api_version});
            check if api_version($version), right("api_version", $allowed), $version == $allowed;
            "#,
            name = format!("Community ({api_version})"),
            user_limit = 20i64,
        )
        .build(&license_signing_key)
        .unwrap()
        .seal()
        .unwrap();

        let license = License::new(biscuit, &validator).unwrap();

        let res = Self {
            installed_licenses: Default::default(),
            valid_licenses: Default::default(),
            validator,
            license_signing_key,
        };
        res.set_valid(&license);
        res.set_installed(vec![license]);

        res
    }
}

#[cfg(feature = "test")]
impl MockLicensingService {
    pub fn set_installed(&self, installed: Vec<License>) {
        *self.installed_licenses.write().unwrap() = installed;
    }
    pub fn add_installed(&self, license: License) {
        self.installed_licenses.write().unwrap().push(license);
    }
    pub fn set_valid(&self, license: &License) {
        (self.valid_licenses.write().unwrap()).insert(license.id().to_vec());
    }
    // pub fn set_invalid(&self, license: &License) {
    //     (self.valid_licenses.write().unwrap()).remove(license.id());
    // }
}

impl LicensingServiceImpl for MockLicensingService {
    fn installed_licenses(&self) -> Vec<License> {
        self.installed_licenses.read().unwrap().clone()
    }
    fn is_license_valid(&self, license: &License) -> bool {
        self.valid_licenses.read().unwrap().contains(license.id())
    }

    fn reload(&self) -> Result<(), NoValidLicense> {
        // Do nothing, act as if nothing had changed.
        Ok(())
    }

    fn deserialize_license_bytes(&self, _bytes: &[u8]) -> Result<License, ValidationError> {
        unimplemented!()
    }
    fn deserialize_license_base64(
        &self,
        _base64: &str,
    ) -> Result<License, Either<base64::DecodeError, ValidationError>> {
        unimplemented!()
    }
    fn install_license(&self, _license: License) -> Result<(), anyhow::Error> {
        unimplemented!()
    }
}
