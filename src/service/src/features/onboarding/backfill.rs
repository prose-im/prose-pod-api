// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm::DatabaseConnection;

use crate::{dependencies, network_checks::NetworkChecker, AppConfig};

use super::{all_dns_checks_passed_once, at_least_one_invitation_sent};

/// If the database was created before this feature was introduced, some keys
/// could be missing while they should have been set. This function tries to
/// generate missing values from existing data.
#[tracing::instrument(level = "trace", skip_all)]
pub async fn backfill(
    db: &DatabaseConnection,
    app_config: &AppConfig,
    network_checker: &NetworkChecker,
    uuid_gen: &dependencies::Uuid,
) {
    use tracing::warn;

    // Backfill `all_dns_checks_passed_once` if necessary.
    if all_dns_checks_passed_once::get_opt(db).await.is_none() {
        if let Err(err) = backfill_all_dns_checks_passed_once(db, app_config, network_checker).await
        {
            warn!("Could not backfill `all_dns_checks_passed_once`: {err}");
        }
    }

    // Backfill `at_least_one_invitation_sent` if necessary.
    if at_least_one_invitation_sent::get_opt(db).await.is_none() {
        if let Err(err) = backfill_at_least_one_invitation_sent(db, uuid_gen).await {
            warn!("Could not backfill `at_least_one_invitation_sent`: {err}");
        }
    }
}

async fn backfill_all_dns_checks_passed_once(
    db: &DatabaseConnection,
    app_config: &AppConfig,
    network_checker: &NetworkChecker,
) -> anyhow::Result<()> {
    use futures::{stream::FuturesOrdered, StreamExt};
    use tracing::{info, trace};

    use crate::network_checks::PodNetworkConfig;
    use crate::network_checks::{NetworkCheck as _, RetryableNetworkCheckResult as _};
    use crate::pod_config::PodConfigRepository;
    use crate::server_config::ServerConfigRepository;

    const KEY: &'static str = "all_dns_checks_passed_once";

    let server_domain = match ServerConfigRepository::get(db).await? {
        Some(config) => config.domain,
        None => {
            info!("Not backfilling `{KEY}`: Server config not initalized.");
            return Ok(());
        }
    };

    let pod_address = match (PodConfigRepository::get(db).await?)
        .map(|config| config.network_address())
        .flatten()
    {
        Some(address) => address,
        None => {
            info!("Not backfilling `{KEY}`: Pod address not initalized.");
            return Ok(());
        }
    };

    let federation_enabled = match ServerConfigRepository::get(db).await? {
        Some(model) => {
            let server_config = model.with_default_values_from(app_config);
            server_config.federation_enabled
        }
        None => {
            info!("Not backfilling `{KEY}`: Server config not initalized.");
            return Ok(());
        }
    };

    let pod_network_config = PodNetworkConfig {
        server_domain,
        pod_address,
        federation_enabled,
    };

    trace!("Backfilling `{KEY}`…");

    let all_dns_checks_passed = pod_network_config
        .dns_record_checks()
        .map(|check| async move { !check.run(network_checker).await.is_failure() })
        .collect::<FuturesOrdered<_>>()
        .all(|is_success| async move { is_success })
        .await;

    // If checks passed now, they passed once. If not, we can’t backfill
    // data as it might be temporary.
    if all_dns_checks_passed {
        trace!("Backfilling `{KEY}` to {all_dns_checks_passed}…");
        return self::all_dns_checks_passed_once::set(db, all_dns_checks_passed).await;
    }

    Ok(())
}

async fn backfill_at_least_one_invitation_sent(
    db: &DatabaseConnection,
    uuid_gen: &dependencies::Uuid,
) -> anyhow::Result<()> {
    use std::str::FromStr as _;

    use jid::BareJid;
    use sea_orm::TransactionTrait as _;
    use tracing::trace;

    use crate::invitations::{InvitationContact, InvitationCreateForm, InvitationRepository};
    use crate::models::EmailAddress;

    const KEY: &'static str = "at_least_one_invitation_sent";

    trace!("Backfilling `{KEY}`…");

    // Inviations are deleted once they are accepted, for privacy reasons,
    // but one way to know if one has been created in the past is to create
    // a new one in a transaction and check the auto-incrementing ID (which
    // don’t get reused, as stated in https://sqlite.org/autoinc.html). If
    // it’s greater than 1, it means a row existed once in the past.
    let transaction = db.begin().await.unwrap();

    let invitation = InvitationRepository::create(
        &transaction,
        InvitationCreateForm {
            jid: BareJid::new("foo@example.org").unwrap(),
            pre_assigned_role: None,
            contact: InvitationContact::Email {
                email_address: EmailAddress::from_str("foo@example.org").unwrap(),
            },
            created_at: None,
        },
        uuid_gen,
    )
    .await?;
    // NOTE: Invitation IDs start at 1 (see https://sqlite.org/autoinc.html).
    let at_least_one_invitation_sent = invitation.id > 1;

    // Explicitly roll back the transaction to ensure it’s not committed.
    transaction.rollback().await?;

    trace!("Backfilling `{KEY}` to {at_least_one_invitation_sent}…");
    self::at_least_one_invitation_sent::set(db, at_least_one_invitation_sent).await
}
