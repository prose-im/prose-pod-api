// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod backfill;

pub use self::backfill::backfill;
pub use self::kv_store::KvStore;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct OnboardingStepsStatuses {
    pub all_dns_checks_passed_once: bool,
    pub at_least_one_invitation_sent: bool,
}

#[tracing::instrument(level = "trace", skip_all)]
pub async fn get_steps_statuses(db: &impl sea_orm::ConnectionTrait) -> OnboardingStepsStatuses {
    OnboardingStepsStatuses {
        all_dns_checks_passed_once: all_dns_checks_passed_once::get(db).await,
        at_least_one_invitation_sent: at_least_one_invitation_sent::get(db).await,
    }
}

crate::gen_scoped_kv_store!("onboarding");

crate::gen_kv_store_get_set!(all_dns_checks_passed_once: bool);
crate::gen_kv_store_get_set!(at_least_one_invitation_sent: bool);
