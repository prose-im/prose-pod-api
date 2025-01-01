// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use sea_orm::{ConnectionTrait, DbErr};
use secrecy::SecretString;

use crate::{
    auth::AuthService,
    xmpp::{
        BareJid, ServerCtl, ServerCtlError, XmppService, XmppServiceContext, XmppServiceError,
        XmppServiceInner,
    },
};

use super::{Member, MemberCreateForm, MemberRepository, MemberRole};

#[derive(Debug, Clone)]
pub struct MemberService {
    server_ctl: Arc<ServerCtl>,
    auth_service: Arc<AuthService>,
    xmpp_service_inner: Arc<XmppServiceInner>,
}

impl MemberService {
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

    pub async fn create_user(
        &self,
        db: &impl ConnectionTrait,
        jid: &BareJid,
        password: &SecretString,
        nickname: &str,
        role: &Option<MemberRole>,
    ) -> Result<Member, UserCreateError> {
        // Create the user in database
        let member = MemberRepository::create(
            db,
            MemberCreateForm {
                jid: jid.to_owned(),
                role: role.to_owned(),
                joined_at: None,
            },
        )
        .await?;

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
    pub async fn delete_user(
        &self,
        db: &impl ConnectionTrait,
        jid: &BareJid,
    ) -> Result<(), UserDeleteError> {
        // Delete the user from database.
        MemberRepository::delete(db, jid).await?;

        // NOTE: We can't rollback changes made to the XMPP server so we do it
        //   after "rollbackable" DB changes in case they fail. It's not perfect
        //   but better than nothing.
        // TODO: Find a way to rollback XMPP server changes.
        let server_ctl = self.server_ctl.clone();

        // Delete the user from the XMPP server.
        server_ctl
            .remove_user(jid)
            .await
            .map_err(UserDeleteError::XmppServerCannotDeleteUser)?;

        Ok(())
    }
}

pub type Error = MemberServiceError;

#[derive(Debug, thiserror::Error)]
pub enum MemberServiceError {
    #[error("Could not create user: {0}")]
    CouldNotCreateUser(#[from] UserCreateError),
    #[error("Could not delete user: {0}")]
    CouldNotDeleteUser(#[from] UserDeleteError),
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
}

#[derive(Debug, thiserror::Error)]
pub enum UserDeleteError {
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
    #[error("XMPP server cannot delete user: {0}")]
    XmppServerCannotDeleteUser(ServerCtlError),
}
