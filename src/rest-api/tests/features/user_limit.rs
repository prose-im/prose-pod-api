// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use biscuit::macros::*;
use service::licensing::License;

use crate::prelude::mocks::LICENSE_SIGNING_KEY;

use super::prelude::*;

#[given(expr = "the user limit is {int}")]
async fn given_user_limit(world: &mut TestWorld, limit: u32) -> Result<(), Error> {
    let ref validator = world.mock_license_service.validator;

    let domain = world.app_config().server_fqdn();
    let biscuit = biscuit!(
        r#"
        name({name});
        user_limit({user_limit});

        // Tie to a test-only domain to prevent reuse.
        right("domain", {domain});
        check if domain($domain), right("domain", $allowed), $domain == $allowed;
        "#,
        name = format!("Testing ({domain})"),
        user_limit = limit as i64,
        domain = domain.to_string(),
    )
    .build(&LICENSE_SIGNING_KEY)
    .unwrap()
    .seal()
    .unwrap();

    let license = License::new(biscuit, validator).unwrap();
    world.mock_license_service.set_valid(&license);
    world.mock_license_service.add_installed(license);

    Ok(())
}
