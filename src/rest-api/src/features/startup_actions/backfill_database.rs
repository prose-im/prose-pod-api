// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    invitations::{InvitationContact, InvitationRepository},
    members::{MemberRole, UnauthenticatedMemberService, UserCreateError},
    sea_orm::TransactionTrait as _,
};
use tracing::{info, warn};

use crate::AppState;

#[tracing::instrument(level = "trace", skip_all, err)]
pub async fn backfill_database(app_state: &AppState) -> Result<(), String> {
    tracing::debug!("Backfilling database…");

    // Backfill onboarding steps status.
    backfill_onboarding_steps(app_state).await?;

    // Backfill users manually created in the XMPP server.
    backfill_xmpp_users(app_state).await?;

    Ok(())
}

pub async fn backfill_onboarding_steps(
    AppState {
        db,
        app_config,
        network_checker,
        uuid_gen,
        ..
    }: &AppState,
) -> Result<(), String> {
    let ref app_config = app_config.read().unwrap().clone();
    service::onboarding::backfill(db, app_config, network_checker, uuid_gen).await;
    Ok(())
}

pub async fn backfill_xmpp_users(
    AppState {
        db,
        server_ctl,
        auth_service,
        xmpp_service,
        license_service,
        ..
    }: &AppState,
) -> Result<(), String> {
    // Load users from the XMPP server (in case of a manual migration).
    let users = (server_ctl.list_users().await)
        .map_err(|err| format!("Could not list XMPP accounts: {err}"))?;

    let member_service = UnauthenticatedMemberService::new(
        server_ctl.clone(),
        auth_service.clone(),
        license_service.clone(),
        xmpp_service.clone(),
    );

    for user in users.iter() {
        // Map XMPP server roles to Prose roles.
        let Some(role) = Option::<MemberRole>::from(&user.role) else {
            continue;
        };

        // Create member in database if it doesn’t exist already.
        match member_service.exists(db, &user.jid).await {
            Ok(false) => {
                let txn = (db.begin().await)
                    .map_err(|err| format!("Could not start transaction: {err}"))?;

                // Try to find email address in an invitation,
                // and delete the invitation if one exists.
                let email_address = match InvitationRepository::get_by_jid(&txn, &user.jid).await {
                    Ok(Some(invitation)) => {
                        // Read the email address.
                        let email_address = match invitation.contact() {
                            InvitationContact::Email { email_address } => Some(email_address),
                        };

                        // Delete the invitation.
                        if let Err(err) =
                            InvitationRepository::delete_by_id(&txn, invitation.id).await
                        {
                            warn!(
                                "Could not delete invitation for {jid}: {err}",
                                jid = user.jid
                            );
                        }

                        email_address
                    }
                    Ok(None) => None,
                    Err(err) => {
                        warn!("Could not find invitation for {jid}: {err}", jid = user.jid);
                        None
                    }
                };
                // TODO: Try to read the email address from the user’s vCard if `None`?

                // Create the user in the Pod API database.
                // NOTE: Prosody doesn’t store the “joined at” timestamp.
                //   We can’t pass it down to `MemberRepository::create`.
                member_service
                    .create_user_local(&txn, &user.jid, &Some(role), email_address)
                    .await
                    .map_err(|err| format!("Could not create user {jid}: {err}", jid = user.jid))?;

                (txn.commit().await)
                    .map_err(|err| format!("Could not create user {jid}: {err}", jid = user.jid))?;

                info!(
                    "Created user {jid} (found in XMPP server, was missing in API database).",
                    jid = user.jid,
                );

                // Add the user to everyone's roster (if needed).
                server_ctl.add_team_member(&user.jid).await.map_err(|err| {
                    UserCreateError::XmppServerCannotAddTeamMember(err).to_string()
                })?;
            }
            Ok(true) => {
                // User already exists, do nothing.
            }
            Err(_) => {
                // Database in faulty state, can’t create members anyway.
            }
        }
    }

    Ok(())
}
