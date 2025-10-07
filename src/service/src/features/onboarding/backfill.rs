// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{
    invitations::InvitationRepository, members::UserRepository, models::DatabaseRwConnectionPools,
    network_checks::NetworkChecker, server_config, AppConfig,
};

/// If the database was created before this feature was introduced, some keys
/// could be missing while they should have been set. This function tries to
/// generate missing values from existing data.
#[tracing::instrument(level = "trace", skip_all)]
pub async fn backfill(
    db: &DatabaseRwConnectionPools,
    app_config: &AppConfig,
    network_checker: &NetworkChecker,
    invitation_repository: &InvitationRepository,
    user_repository: &UserRepository,
) {
    use tracing::warn;

    // Backfill `chosen_server_domain` if necessary.
    if let Err(err) = backfill_chosen_server_domain(db, app_config).await {
        warn!("Could not backfill `chosen_server_domain`: {err}");
    }

    // Backfill `all_dns_checks_passed_once` if necessary.
    if let Err(err) = backfill_all_dns_checks_passed_once(db, app_config, network_checker).await {
        warn!("Could not backfill `all_dns_checks_passed_once`: {err}");
    }

    // Backfill `at_least_one_invitation_sent` if necessary.
    if let Err(err) =
        backfill_at_least_one_invitation_sent(db, invitation_repository, user_repository).await
    {
        warn!("Could not backfill `at_least_one_invitation_sent`: {err}");
    }
}

async fn backfill_chosen_server_domain(
    db: &DatabaseRwConnectionPools,
    app_config: &AppConfig,
) -> anyhow::Result<()> {
    use tracing::trace;

    use super::chosen_server_domain as store;

    const KEY: &'static str = "chosen_server_domain";

    // Do not backfill if a value already exist.
    if store::get_opt(&db.read).await?.is_some() {
        trace!("Not backfilling `{KEY}`: Already set.");
        return Ok(());
    }

    trace!("Backfilling `{KEY}`…");

    let server_domain = app_config.server_domain().to_string();

    // If checks passed now, they passed once. If not, we can’t backfill
    // data as it might be temporary.
    trace!("Backfilling `{KEY}` to {server_domain}…");
    store::set(&db.write, server_domain).await
}

async fn backfill_all_dns_checks_passed_once(
    db: &DatabaseRwConnectionPools,
    app_config: &AppConfig,
    network_checker: &NetworkChecker,
) -> anyhow::Result<()> {
    use futures::{stream::FuturesOrdered, StreamExt};
    use tracing::trace;

    use crate::network_checks::PodNetworkConfig;
    use crate::network_checks::{NetworkCheck as _, RetryableNetworkCheckResult as _};

    use super::all_dns_checks_passed_once as store;

    const KEY: &'static str = "all_dns_checks_passed_once";

    // Do not backfill if a value already exist.
    if store::get_opt(&db.read).await?.is_some() {
        trace!("Not backfilling `{KEY}`: Already set.");
        return Ok(());
    }

    let federation_enabled = (server_config::federation_enabled::get_opt(&db.read).await?)
        .unwrap_or(app_config.server.defaults.federation_enabled);

    let pod_network_config = PodNetworkConfig::new(app_config, federation_enabled);

    trace!("Backfilling `{KEY}`…");

    let all_dns_checks_passed = (pod_network_config.dns_record_checks())
        .map(|check| async move { !check.run(network_checker).await.is_failure() })
        .collect::<FuturesOrdered<_>>()
        .all(|is_success| async move { is_success })
        .await;

    // If checks passed now, they passed once. If not, we can’t backfill
    // data as it might be temporary.
    if all_dns_checks_passed {
        trace!("Backfilling `{KEY}` to {all_dns_checks_passed}…");
        return store::set(&db.write, all_dns_checks_passed).await;
    }

    Ok(())
}

async fn backfill_at_least_one_invitation_sent(
    db: &DatabaseRwConnectionPools,
    invitation_repository: &InvitationRepository,
    user_repository: &UserRepository,
) -> anyhow::Result<()> {
    use super::at_least_one_invitation_sent as store;
    use tracing::trace;

    const KEY: &'static str = "at_least_one_invitation_sent";

    // Do not backfill if a value already exist.
    if store::get_opt(&db.read).await?.is_some() {
        trace!("Not backfilling `{KEY}`: Already set.");
        return Ok(());
    }

    trace!("Backfilling `{KEY}`…");

    /// Just a helper to do logging.
    async fn backfill(
        db: &DatabaseRwConnectionPools,
        at_least_one_invitation_sent: bool,
    ) -> anyhow::Result<()> {
        trace!("Backfilling `{KEY}` to {at_least_one_invitation_sent}…");
        store::set(&db.write, at_least_one_invitation_sent).await
    }

    // Mark true if an invitation is pending.
    let invitation_count = invitation_repository
        .account_invitations_stats(None)
        .await?
        .count;
    if invitation_count > 0 {
        return backfill(db, true).await;
    }

    // Mark true if more than one user exist.
    let user_count = user_repository.users_stats(None).await?.count;
    if user_count > 1 {
        return backfill(db, true).await;
    }

    // Otherwise, mark false.
    backfill(db, false).await
}
