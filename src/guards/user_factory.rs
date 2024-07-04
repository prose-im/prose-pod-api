// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_pod_core::prose_xmpp::BareJid;
use prose_pod_core::repositories::{
    Invitation, InvitationRepository, Member, MemberCreateForm, MemberRepository,
    ServerConfigRepository,
};
use prose_pod_core::sea_orm::{DatabaseTransaction, DbConn, TransactionTrait as _};
use prose_pod_core::MemberRole;
use prose_pod_core::{xmpp_service, AuthService, ServerCtl, XmppServiceContext, XmppServiceInner};
use rocket::outcome::try_outcome;
use rocket::request::Outcome;
use rocket::{Request, State};

use crate::error::{self, Error};

use super::jwt::JWT;
use super::{database_connection, LazyFromRequest};

pub struct UserFactory<'r> {
    server_ctl: &'r State<ServerCtl>,
    auth_service: &'r State<AuthService>,
    xmpp_service_inner: &'r State<XmppServiceInner>,
}

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for UserFactory<'r> {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let server_ctl =
            try_outcome!(req
                .guard::<&State<ServerCtl>>()
                .await
                .map_error(|(status, _)| (
                    status,
                    Error::InternalServerError {
                        reason: "Could not get a `&State<ServerCtl>` from a request.".to_string(),
                    }
                )));
        let auth_service =
            try_outcome!(req
                .guard::<&State<AuthService>>()
                .await
                .map_error(|(status, _)| (
                    status,
                    Error::InternalServerError {
                        reason: "Could not get a `&State<AuthService>` from a request.".to_string(),
                    }
                )));
        let xmpp_service_inner = try_outcome!(req
            .guard::<&State<XmppServiceInner>>()
            .await
            .map_error(|(status, _)| (
                status,
                Error::InternalServerError {
                    reason: "Could not get a `&State<XmppServiceInner>` from a request."
                        .to_string(),
                }
            )));

        // Make sure the Prose Pod is initialized, as we can't add or remove users otherwise.
        // TODO: Check that the Prose Pod is initialized another way (this doesn't cover all cases)
        let db = try_outcome!(database_connection(req).await);
        match ServerConfigRepository::get(db).await {
            Ok(Some(_)) => {}
            Ok(None) => return Error::ServerConfigNotInitialized.into(),
            Err(err) => return Error::DbErr(err).into(),
        }

        Outcome::Success(Self {
            server_ctl,
            auth_service,
            xmpp_service_inner,
        })
    }
}

impl<'r> UserFactory<'r> {
    pub(super) fn new(
        server_ctl: &'r State<ServerCtl>,
        auth_service: &'r State<AuthService>,
        xmpp_service_inner: &'r State<XmppServiceInner>,
    ) -> Self {
        Self {
            server_ctl,
            auth_service,
            xmpp_service_inner,
        }
    }

    pub async fn create_user<'a>(
        &self,
        txn: &DatabaseTransaction,
        jid: &BareJid,
        password: &str,
        nickname: &str,
        role: &Option<MemberRole>,
    ) -> Result<Member, Error> {
        // Create the user in database
        let member = MemberRepository::create(
            txn,
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
        let server_ctl = self.server_ctl;

        server_ctl.add_user(jid, password)?;
        if let Some(role) = role {
            server_ctl.set_user_role(jid, &role)?;
        }

        let jwt = self.auth_service.log_in(jid, password)?;
        let jwt =
            JWT::try_from(&jwt, self.auth_service).map_err(|e| Error::InternalServerError {
                reason: format!("The just-created JWT is invalid: {e}"),
            })?;
        let prosody_token = jwt.prosody_token()?;

        let ctx = XmppServiceContext {
            bare_jid: jid.to_owned(),
            prosody_token,
        };
        let xmpp_service = xmpp_service::XmppService::new(self.xmpp_service_inner.inner(), ctx);

        // TODO: Create the vCard using a display name instead of the nickname
        xmpp_service.create_own_vcard(nickname)?;
        // xmpp_service.set_own_nickname(nickname)?;

        Ok(member)
    }

    pub async fn accept_workspace_invitation(
        &self,
        db: &DbConn,
        invitation: Invitation,
        password: &str,
        nickname: &str,
    ) -> Result<(), Error> {
        let txn = db.begin().await?;

        // Create the user
        self.create_user(
            &txn,
            &invitation.jid,
            password,
            nickname,
            &Some(invitation.pre_assigned_role),
        )
        .await?;

        // Delete the invitation from database
        InvitationRepository::accept(&txn, invitation).await?;

        // Commit the transaction if everything went well
        txn.commit().await?;

        Ok(())
    }
}
