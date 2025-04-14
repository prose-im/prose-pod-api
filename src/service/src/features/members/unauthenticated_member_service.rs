// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use sea_orm::{ConnectionTrait, DbErr, TransactionTrait};
use secrecy::SecretString;
use tracing::instrument;

use crate::{
    auth::AuthService,
    xmpp::{
        BareJid, ServerCtl, ServerCtlError, XmppService, XmppServiceContext, XmppServiceError,
        XmppServiceInner,
    },
};

use super::{Member, MemberCreateForm, MemberRepository, MemberRole};

#[derive(Debug, Clone)]
pub struct UnauthenticatedMemberService {
    server_ctl: Arc<ServerCtl>,
    auth_service: Arc<AuthService>,
    xmpp_service_inner: Arc<XmppServiceInner>,
}

impl UnauthenticatedMemberService {
    pub fn new(
        server_ctl: Arc<ServerCtl>,
        auth_service: Arc<AuthService>,
        xmpp_service_inner: Arc<XmppServiceInner>,
    ) -> Self {
        Self {
            server_ctl,
            auth_service,
            xmpp_service_inner,
        }
    }
}

impl UnauthenticatedMemberService {
    #[instrument(
        level = "trace",
        skip_all, fields(
            jid = jid.to_string(),
            role = role.as_ref().map(ToString::to_string),
        ),
        err,
    )]
    pub async fn create_user<DB: ConnectionTrait + TransactionTrait>(
        &self,
        db: &DB,
        jid: &BareJid,
        password: &SecretString,
        nickname: &str,
        role: &Option<MemberRole>,
    ) -> Result<Member, UserCreateError> {
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
            },
        )
        .await?;

        // Check if the member limit is reached.
        let count = MemberRepository::count(&txn).await?;
        if count > MEMBER_LIMIT.as_u64() {
            txn.rollback().await?;
            return Err(UserCreateError::LimitReached);
        }

        // Try to commit the transaction before performing
        // non-transactional XMPP server changes.
        txn.commit().await?;

        // NOTE: We can't rollback changes made to the XMPP server so we do it
        //   after "rollbackable" DB changes in case they fail. It's not perfect
        //   but better than nothing.
        // TODO: Find a way to rollback XMPP server changes.
        let server_ctl = self.server_ctl.clone();

        // Create the user
        server_ctl
            .add_user(jid, &password)
            .await
            .map_err(UserCreateError::XmppServerCannotCreateUser)?;
        // Add the user to everyone's roster
        server_ctl
            .add_team_member(jid)
            .await
            .map_err(UserCreateError::XmppServerCannotAddTeamMember)?;
        if let Some(role) = role {
            // Set the user's role for servers which support it
            server_ctl
                .set_user_role(jid, &role)
                .await
                .map_err(UserCreateError::XmppServerCannotSetUserRole)?;
        }

        // NOTE: We need to log the user in to get a Prosody authentication token
        //   in order to set the user's vCard.
        let auth_token = self
            .auth_service
            .log_in(jid, &password)
            .await
            .expect("User was created with credentials which doesn't work.");

        let ctx = XmppServiceContext {
            bare_jid: jid.to_owned(),
            prosody_token: auth_token.clone(),
        };
        let xmpp_service = XmppService::new(self.xmpp_service_inner.clone(), ctx);

        // TODO: Create the vCard using a display name instead of the nickname
        xmpp_service
            .create_own_vcard(nickname)
            .await
            .map_err(UserCreateError::CouldNotCreateVCard)?;
        // xmpp_service.set_own_nickname(nickname)?;

        Ok(member)
    }
}

#[cfg(not(feature = "test"))]
#[doc(hidden)]
#[repr(transparent)]
struct MemberLimit(u32);

#[cfg(not(feature = "test"))]
lazy_static::lazy_static! {
    /// WARNING: Any attempt to reverse-engineer the Prose Pod API
    ///   for the purpose of bypassing or removing the member limit,
    ///   is strictly prohibited and may expose you to legal action.
    #[doc(hidden)]
    static ref MEMBER_LIMIT: MemberLimit = {
        const MARKER: u128 = u128::from_be(0x128A21444f5f4e4f545f4d4f44494659u128);
        // Prevent `100` from appearing in the binary to prevent tampering.
        static VALUE: u128 = 100u128 | MARKER;
        // NOTE: `read_volatile` prevents `MARKER` from being optimized away.
        let value: u128 = unsafe { std::ptr::read_volatile(&VALUE as *const u128) };
        // Remove `MARKER`.
        MemberLimit((value ^ MARKER) as u32)
    };
}

#[cfg(not(feature = "test"))]
impl MemberLimit {
    fn as_u64(&self) -> u64 {
        self.0 as u64
    }
}

#[cfg(feature = "test")]
#[doc(hidden)]
#[repr(transparent)]
pub struct MemberLimit(pub std::sync::atomic::AtomicU32);

#[cfg(feature = "test")]
lazy_static::lazy_static! {
    #[doc(hidden)]
    pub static ref MEMBER_LIMIT: MemberLimit = {
        MemberLimit(std::sync::atomic::AtomicU32::new(100))
    };
}

#[cfg(feature = "test")]
impl MemberLimit {
    fn as_u64(&self) -> u64 {
        self.0.load(std::sync::atomic::Ordering::Relaxed) as u64
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
