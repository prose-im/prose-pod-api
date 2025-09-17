// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm::{ConnectionTrait, DbErr, TransactionTrait};
use secrecy::SecretString;
use tracing::instrument;

use crate::{
    auth::AuthService,
    licensing::LicenseService,
    models::EmailAddress,
    xmpp::{
        BareJid, ServerCtl, ServerCtlError, XmppService, XmppServiceContext, XmppServiceError,
        XmppServiceInner,
    },
};

use super::{entities::member, Member, MemberCreateForm, MemberRepository, MemberRole};

#[derive(Debug, Clone)]
pub struct UnauthenticatedMemberService {
    server_ctl: ServerCtl,
    auth_service: AuthService,
    license_service: LicenseService,
    xmpp_service_inner: XmppServiceInner,
}

impl UnauthenticatedMemberService {
    pub fn new(
        server_ctl: ServerCtl,
        auth_service: AuthService,
        license_service: LicenseService,
        xmpp_service_inner: XmppServiceInner,
    ) -> Self {
        Self {
            server_ctl,
            auth_service,
            license_service,
            xmpp_service_inner,
        }
    }
}

impl UnauthenticatedMemberService {
    #[instrument(
        level = "trace",
        skip_all, fields(jid = jid.to_string()),
        err,
    )]
    pub async fn exists<Db: ConnectionTrait>(&self, db: &Db, jid: &BareJid) -> Result<bool, DbErr> {
        MemberRepository::exists(db, jid).await
    }

    #[instrument(
        level = "trace",
        skip_all, fields(
            jid = jid.to_string(),
            role = role.as_ref().map(ToString::to_string),
        ),
        err,
    )]
    pub async fn create_user<Db: ConnectionTrait + TransactionTrait>(
        &self,
        db: &Db,
        jid: &BareJid,
        password: &SecretString,
        nickname: &str,
        role: &Option<MemberRole>,
        email_address: Option<EmailAddress>,
    ) -> Result<Member, UserCreateError> {
        // Create the user in the Pod API database.
        let member = self.create_user_local(db, jid, role, email_address).await?;

        // NOTE: We can't rollback changes made to the XMPP server so we do it
        //   after "rollbackable" DB changes in case they fail. It's not perfect
        //   but better than nothing.
        // TODO: Find a way to rollback XMPP server changes.
        let server_ctl = self.server_ctl.clone();

        // Create the user
        (server_ctl.add_user(jid, &password).await)
            .map_err(UserCreateError::XmppServerCannotCreateUser)?;
        // FIXME: Re-enable.
        let disabled = true;
        // Add the user to everyone's roster
        // (server_ctl.add_team_member(jid).await)
        //     .map_err(UserCreateError::XmppServerCannotAddTeamMember)?;
        if let Some(role) = role {
            // Set the user's role for servers which support it
            (server_ctl.set_user_role(jid, &role).await)
                .map_err(UserCreateError::XmppServerCannotSetUserRole)?;
        }

        // NOTE: We need to log the user in to get a Prosody authentication token
        //   in order to set the user's vCard.
        let auth_token = (self.auth_service.log_in(jid, &password).await)
            .expect("User was created with credentials which don’t work.");

        let ctx = XmppServiceContext {
            bare_jid: jid.to_owned(),
            prosody_token: auth_token.clone(),
        };
        let xmpp_service = XmppService::new(self.xmpp_service_inner.clone(), ctx);

        // TODO: Create the vCard using a display name instead of the nickname
        (xmpp_service.create_own_vcard(nickname).await)
            .map_err(UserCreateError::CouldNotCreateVCard)?;
        // xmpp_service.set_own_nickname(nickname)?;

        Ok(member.into())
    }

    /// Creates a user in the local database, but not on the XMPP server.
    /// Useful when reconciliating data from the XMPP server to the Pod API.
    #[instrument(
        level = "trace",
        skip_all, fields(
            jid = jid.to_string(),
            role = role.as_ref().map(ToString::to_string),
        ),
        err,
    )]
    pub async fn create_user_local<Db: ConnectionTrait + TransactionTrait>(
        &self,
        db: &Db,
        jid: &BareJid,
        role: &Option<MemberRole>,
        email_address: Option<EmailAddress>,
    ) -> Result<member::Model, UserCreateError> {
        // Start a database transaction so we can roll back
        // if the member limit is reached.
        let txn = db.begin().await?;

        // Create the user in database
        let member = MemberRepository::create(
            &txn,
            MemberCreateForm {
                jid: jid.to_owned(),
                role: role.to_owned(),
                joined_at: None,
                email_address,
            },
        )
        .await?;

        // Check if the member limit is reached.
        let count: u64 = MemberRepository::count(&txn).await?;
        let count: u32 = count.clamp(u32::MIN as u64, u32::MAX as u64) as u32;
        if !self.license_service.allows_user_count(count) {
            txn.rollback().await?;
            return Err(UserCreateError::LimitReached);
        }

        // Try to commit the transaction before performing
        // non-transactional XMPP server changes.
        txn.commit().await?;

        Ok(member)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UserCreateError {
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
    #[error("Could not create user vCard: {0}")]
    CouldNotCreateVCard(XmppServiceError),
    #[error("XMPP server cannot create user: {0}")]
    XmppServerCannotCreateUser(ServerCtlError),
    #[error("XMPP server cannot add team member: {0}")]
    XmppServerCannotAddTeamMember(ServerCtlError),
    #[error("XMPP server cannot set user role: {0}")]
    XmppServerCannotSetUserRole(ServerCtlError),
    #[error("Member limit reached.")]
    LimitReached,
}
