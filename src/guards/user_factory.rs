// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::model::{MemberRole, JID};
use entity::{member, workspace_invitation};
use rocket::outcome::try_outcome;
use rocket::request::Outcome;
use rocket::{Request, State};
use sea_orm_rocket::Connection;
use service::sea_orm::{DatabaseTransaction, DbConn, TransactionTrait as _};
use service::{Mutation, Query, ServerCtl};

use crate::error::{self, Error};

use super::{Db, LazyFromRequest};

pub struct UserFactory<'r> {
    server_ctl: &'r State<ServerCtl>,
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

        // Make sure the Prose Pod is initialized, as we can't add or remove users otherwise.
        // TODO: Check that the Prose Pod is initialized another way (this doesn't cover all cases)
        let db = try_outcome!(req
            .guard::<Connection<'_, Db>>()
            .await
            .map(|conn| conn.into_inner())
            .map_error(|(status, err)| {
                (status, err.map(Error::DbErr).unwrap_or(Error::UnknownDbErr))
            }));
        match Query::server_config(db).await {
            Ok(Some(_)) => {}
            Ok(None) => return Error::ServerConfigNotInitialized.into(),
            Err(err) => return Error::DbErr(err).into(),
        }

        Outcome::Success(Self { server_ctl })
    }
}

impl<'r> UserFactory<'r> {
    pub async fn create_user<'a>(
        &self,
        txn: &DatabaseTransaction,
        jid: &JID,
        password: &str,
        nickname: &str,
        role: &Option<MemberRole>,
    ) -> Result<member::Model, Error> {
        // Create the user in database
        let member = Mutation::create_user(txn, &jid, role).await?;

        // NOTE: We can't rollback changes made to the XMPP server so let's do it
        //   after "rollbackable" DB changes in case they fail. It's not perfect
        //   but better than nothing.
        // TODO: Find a way to rollback XMPP server changes.
        let server_ctl = self.server_ctl.lock().expect("Serverctl lock poisonned");

        server_ctl.add_user(jid, password)?;
        if let Some(role) = role {
            server_ctl.set_user_role(jid, &role)?;
        }
        // TODO: Create the vCard using a display name instead of the nickname
        server_ctl.create_vcard(jid, nickname)?;
        server_ctl.set_nickname(jid, nickname)?;

        Ok(member)
    }

    pub async fn accept_workspace_invitation(
        &self,
        db: &DbConn,
        invitation: workspace_invitation::Model,
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
        Mutation::accept_workspace_invitation(&txn, invitation).await?;

        // Commit the transaction if everything went well
        txn.commit().await?;

        Ok(())
    }
}
