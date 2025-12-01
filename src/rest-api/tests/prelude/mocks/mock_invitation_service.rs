// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::invitations::{invitation_service::prelude::*, InvitationRepositoryImpl as _};

use super::prelude::*;

#[derive(Debug)]
pub struct MockInvitationService {
    pub server: Arc<MockServerService>,
    pub mock_invitation_repository: Arc<MockInvitationRepository>,
    pub mock_user_repository: Arc<MockUserRepository>,
    pub mock_auth_service: MockAuthService,
}

#[async_trait]
impl InvitationApplicationServiceImpl for MockInvitationService {
    async fn register_with_token(
        &self,
        password: &Password,
        token: InvitationToken,
    ) -> Result<
        RegisterResponse,
        Either3<InvitationNotFoundForToken, MemberAlreadyExists, anyhow::Error>,
    > {
        self.server.check_online()?;

        let invitation = self
            .mock_invitation_repository
            .get_account_invitation_by_token(&token)
            .await?
            .ok_or(Either3::E1(InvitationNotFoundForToken))?;

        let email_address = match invitation.contact() {
            InvitationContact::Email { email_address } => email_address,
        };
        let account = UserAccount {
            jid: invitation.jid.clone(),
            password: password.clone(),
            role: invitation.pre_assigned_role.as_prosody(),
            joined_at: OffsetDateTime::now_utc(),
            email_address: Some(email_address.clone()),
        };
        self.mock_user_repository
            .add_user(
                &Nickname::from_string_unsafe(invitation.jid.expect_username().to_string()),
                account,
                &self.mock_auth_service,
            )
            .await?;

        self.mock_invitation_repository.delete_invitation_(token);

        Ok(RegisterResponse {
            jid: invitation.jid.clone(),
        })
    }

    async fn reject_invitation(&self, token: InvitationToken) -> Result<(), anyhow::Error> {
        self.server.check_online()?;

        self.mock_auth_service
            .state_mut()
            .password_reset_tokens
            .remove(&token);

        self.mock_invitation_repository.delete_invitation_(token);

        Ok(())
    }
}
