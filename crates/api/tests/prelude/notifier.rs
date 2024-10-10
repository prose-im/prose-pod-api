// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use ::service::notifier::notification_message;
use cucumber::then;
use regex::Regex;

use crate::TestWorld;

#[then(expr = "{int} email(s) should have been sent")]
fn then_n_emails_sent(world: &mut TestWorld, n: usize) {
    assert_eq!(world.notifier_state().send_count, n);
}

#[then(expr = "the email body should match {string}")]
fn then_email_matches(world: &mut TestWorld, pattern: Regex) {
    let email = notification_message(&world.app_config.branding, &world.notifier_state().sent[0]);
    assert!(pattern.is_match(&email));
}
