// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{get, put};

/// Get the results of all Compliance Suites executed on your server.
#[utoipa::path(
    tag = "Server / Compliance tests",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[get("/v1/server/compliance-tests")]
pub(super) fn get_compliance_tests_results() -> String {
    todo!()
}

/// Get the results of a Compliance Suite executed on your server.
#[utoipa::path(
    tag = "Server / Compliance tests",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[get("/v1/server/compliance-tests/<_xep_id>")]
pub(super) fn get_compliance_suite_results(_xep_id: String) -> String {
    todo!()
}

/// Configure which Compliance Suites to test on your server.
#[utoipa::path(
    tag = "Server / Compliance tests",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[put("/v1/server/compliance-tests")]
pub(super) fn set_compliance_test_suites() -> String {
    todo!()
}
