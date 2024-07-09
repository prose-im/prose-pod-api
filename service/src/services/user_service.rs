// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::model::MemberRole;
use prose_xmpp::BareJid;
use sea_orm::{ConnectionTrait, DbErr};

use crate::repositories::{Member, MemberCreateForm, MemberRepository};

use super::{
    auth_service::AuthService,
    server_ctl::{ServerCtl, ServerCtlError},
    xmpp_service::{XmppService, XmppServiceContext, XmppServiceError, XmppServiceInner},
};

#[derive(Debug, Clone)]
pub struct UserService<'r> {
    server_ctl: &'r ServerCtl,
    auth_service: &'r AuthService,
    xmpp_service_inner: &'r XmppServiceInner,
}

impl<'r> UserService<'r> {
    pub fn new(
        server_ctl: &'r ServerCtl,
        auth_service: &'r AuthService,
        xmpp_service_inner: &'r XmppServiceInner,
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
        password: &str,
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

        // NOTE: We can't rollback changes made to the XMPP server so let's do it
        //   after "rollbackable" DB changes in case they fail. It's not perfect
        //   but better than nothing.
        // TODO: Find a way to rollback XMPP server changes.
        let server_ctl = self.server_ctl.clone();

        server_ctl
            .add_user(jid, password)
            .map_err(UserCreateError::XmppServerCannotCreateUser)?;
        if let Some(role) = role {
            server_ctl
                .set_user_role(jid, &role)
                .map_err(UserCreateError::XmppServerCannotSetUserRole)?;
        }

        // NOTE: We need to log the user in to get a Prosody authentication token
        //   in order to set the user's vCard.
        let jwt = self
            .auth_service
            .log_in(jid, password)
            .expect("User was created with credentials which doesn't work.");
        let jwt = self
            .auth_service
            .verify(&jwt)
            .expect("The just-created JWT is invalid.");
        let prosody_token = jwt
            .prosody_token()
            .expect("The just-created JWT doesn't contain a Prosody token.");

        let ctx = XmppServiceContext {
            bare_jid: jid.to_owned(),
            prosody_token,
        };
        let xmpp_service = XmppService::new(&self.xmpp_service_inner, ctx);

        // TODO: Create the vCard using a display name instead of the nickname
        xmpp_service
            .create_own_vcard(nickname)
            .map_err(UserCreateError::CouldNotCreateVCard)?;
        // xmpp_service.set_own_nickname(nickname)?;

        Ok(member)
    }
}

pub type Error = UserServiceError;

#[derive(Debug, thiserror::Error)]
pub enum UserServiceError {
    #[error("Could not create user: {0}")]
    CouldNotCreateUser(#[from] UserCreateError),
}

#[derive(Debug, thiserror::Error)]
pub enum UserCreateError {
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
    #[error("Could not create user vCard: {0}")]
    CouldNotCreateVCard(XmppServiceError),
    #[error("XMPP server cannot create user: {0}")]
    XmppServerCannotCreateUser(ServerCtlError),
    #[error("XMPP server cannot set user role: {0}")]
    XmppServerCannotSetUserRole(ServerCtlError),
}
