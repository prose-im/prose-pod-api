// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod dns_record_check;
pub mod ip_connectivity_check;
pub mod port_reachability_check;

use async_trait::async_trait;
pub use dns_record_check::*;
pub use ip_connectivity_check::*;
pub use port_reachability_check::*;

use crate::features::network_checks::NetworkChecker;

#[async_trait]
pub trait NetworkCheck {
    type CheckResult;
    fn description(&self) -> String;
    async fn run(&self, network_checker: &NetworkChecker) -> Self::CheckResult;
}

pub trait RetryableNetworkCheckResult {
    fn should_retry(&self) -> bool;
}
