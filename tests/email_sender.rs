// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use cucumber::then;

use crate::TestWorld;

#[then(expr = "{int} email(s) should be queued in the email sender")]
fn then_n_emails_queued(_world: &mut TestWorld, _n: usize) {
    todo!()
}
