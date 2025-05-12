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

use crate::network_checks::NetworkChecker;

#[async_trait]
pub trait NetworkCheck {
    type CheckId;
    type CheckResult;
    fn description(&self) -> String;
    fn check_type() -> &'static str;
    fn id(&self) -> Self::CheckId;
    async fn run(&self, network_checker: &NetworkChecker) -> Self::CheckResult;
}

pub trait RetryableNetworkCheckResult {
    fn is_failure(&self) -> bool;
    fn should_retry(&self) -> bool {
        self.is_failure()
    }
}
