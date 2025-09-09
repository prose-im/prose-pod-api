// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use jid::DomainRef;
use sea_orm::{DatabaseConnection, DbErr, TransactionTrait as _};
use secrecy::SecretString;
use tracing::instrument;

use crate::{
    members::{
        Member, MemberRepository, MemberRole, Nickname, UnauthenticatedMemberService,
        UserCreateError,
    },
    models::JidNode,
    util::bare_jid_from_username,
};

pub struct InitService {
    pub db: Arc<DatabaseConnection>,
}

impl InitService {
    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn init_first_account(
        &self,
        server_domain: &DomainRef,
        member_service: &UnauthenticatedMemberService,
        form: impl Into<InitFirstAccountForm>,
    ) -> Result<Member, InitFirstAccountError> {
        let form = form.into();
        let jid = bare_jid_from_username(&form.username, server_domain);

        if MemberRepository::count(self.db.as_ref()).await? > 0 {
            return Err(InitFirstAccountError::FirstAccountAlreadyCreated);
        }

        let txn = self.db.begin().await?;
        let member = member_service
            .create_user(
                &txn,
                &jid,
                &form.password,
                &form.nickname,
                &Some(MemberRole::Admin),
                // TODO: See [First admin account has no email address · Issue #256 · prose-im/prose-pod-api](https://github.com/prose-im/prose-pod-api/issues/256).
                None,
            )
            .await
            .map_err(InitFirstAccountError::CouldNotCreateFirstAccount)?;
        txn.commit().await?;

        Ok(member)
    }
}

#[derive(Debug)]
pub struct InitFirstAccountForm {
    pub username: JidNode,
    pub password: SecretString,
    pub nickname: Nickname,
}

#[derive(Debug, thiserror::Error)]
pub enum InitFirstAccountError {
    #[error("First account already created.")]
    FirstAccountAlreadyCreated,
    #[error("Could not create first account: {0}")]
    CouldNotCreateFirstAccount(UserCreateError),
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
}
