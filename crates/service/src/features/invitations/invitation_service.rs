// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm::{DbConn, DbErr, TransactionTrait as _};
use secrecy::SecretString;

use crate::{
    invitations::{Invitation, InvitationRepository},
    members::{UnauthenticatedMemberService, UserCreateError},
    MutationError,
};

#[derive(Debug, Clone)]
pub struct InvitationService {
    member_service: UnauthenticatedMemberService,
}

impl InvitationService {
    pub fn new(member_service: UnauthenticatedMemberService) -> Self {
        Self { member_service }
    }

    pub async fn accept(
        &self,
        db: &DbConn,
        invitation: Invitation,
        password: &SecretString,
        nickname: &str,
    ) -> Result<(), InvitationAcceptError> {
        let txn = db.begin().await?;

        // Create the user
        self.member_service
            .create_user(
                &txn,
                &invitation.jid,
                &password,
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

pub type Error = InvitationServiceError;

#[derive(Debug, thiserror::Error)]
pub enum InvitationServiceError {
    #[error("Could not accept invitation: {0}")]
    CouldNotAcceptInvitation(#[from] InvitationAcceptError),
}

#[derive(Debug, thiserror::Error)]
pub enum InvitationAcceptError {
    #[error("Could not create user: {0}")]
    CouldNotCreateUser(#[from] UserCreateError),
    #[error("Invitation repository could not accept the inviation: {0}")]
    CouldNotAcceptInvitation(#[from] MutationError),
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
}
