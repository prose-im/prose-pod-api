// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{get, put, response::status::NoContent};

use crate::error::Error;

/// Get the results of all Compliance Suites executed on your server.
#[get("/v1/server/compliance-tests")]
pub(super) fn get_compliance_tests_results() -> Result<NoContent, Error> {
    Err(Error::NotImplemented("Get all compliance suites results"))
}

/// Get the results of a Compliance Suite executed on your server.
#[get("/v1/server/compliance-tests/<_>")]
pub(super) fn get_compliance_suite_results() -> Result<NoContent, Error> {
    Err(Error::NotImplemented("Get compliance suite results"))
}

/// Configure which Compliance Suites to test on your server.
#[put("/v1/server/compliance-tests")]
pub(super) fn set_compliance_test_suites() -> Result<NoContent, Error> {
    Err(Error::NotImplemented("Set compliance test suites"))
}
