// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod dns_records;
pub mod ip_connectivity;
pub mod ports_reachability;

pub use dns_records::*;
pub use ip_connectivity::*;
pub use ports_reachability::*;

use crate::services::network_checker::NetworkChecker;

pub trait NetworkCheck {
    type CheckResult;
    fn run(&self, network_checker: &NetworkChecker) -> Self::CheckResult;
}

pub trait RetryableNetworkCheckResult {
    fn should_retry(&self) -> bool;
}
