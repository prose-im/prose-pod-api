// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod backfill;

use serdev::Serialize;

pub use self::backfill::backfill;
pub use self::kv_store::{get_bool, set_bool};

#[derive(Debug)]
#[derive(Serialize)]
pub struct OnboardingStepsStatuses {
    pub all_dns_checks_passed_once: bool,
    pub at_least_one_invitation_sent: bool,
}

#[tracing::instrument(level = "trace", skip_all)]
pub async fn get_steps_statuses(db: &impl sea_orm::ConnectionTrait) -> OnboardingStepsStatuses {
    OnboardingStepsStatuses {
        all_dns_checks_passed_once: all_dns_checks_passed_once::get_or_default(db).await,
        at_least_one_invitation_sent: at_least_one_invitation_sent::get_or_default(db).await,
    }
}

crate::gen_scoped_kv_store!(pub(super) onboarding; get/set: bool);

// TODO: Remove `pub` once network checks logic has been moved to `service`.
crate::gen_kv_store_scoped_get_set!(pub all_dns_checks_passed_once: bool [+default]);
crate::gen_kv_store_scoped_get_set!(pub at_least_one_invitation_sent: bool [+default]);
