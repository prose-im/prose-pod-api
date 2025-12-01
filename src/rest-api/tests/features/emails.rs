// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use cucumber::then;
use regex::Regex;

use crate::TestWorld;

#[then(expr = "{int} email(s) should have been sent")]
pub fn then_n_emails_sent(world: &mut TestWorld, n: usize) {
    assert_eq!(world.email_notifier_state().send_count, n);
}

#[then(expr = "the email body should match {string}")]
pub fn then_email_matches(world: &mut TestWorld, pattern: Regex) {
    let email = world.email_notifier_state().sent[0].message_plain.clone();
    assert!(pattern.is_match(&email), "email:\n{email:#}");
}
