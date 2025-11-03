// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::invitations::invitation_repository::prelude::*;

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct MockInvitationRepository {
    pub state: Arc<RwLock<MockInvitationRepositoryState>>,
    pub mock_server_state: Arc<RwLock<MockServerServiceState>>,
    pub mock_auth_service_state: Arc<RwLock<MockAuthServiceState>>,
    pub server_domain: JidDomain,
    pub invitations_ttl: Duration,
}

#[derive(Debug, Default)]
pub struct MockInvitationRepositoryState {
    pub invitations: HashMap<EmailAddress, Invitation>,
}

#[async_trait]
impl InvitationRepositoryImpl for MockInvitationRepository {
    async fn create_account_invitation(
        &self,
        CreateAccountInvitationCommand {
            username,
            role,
            email_address,
            ttl,
        }: CreateAccountInvitationCommand,
        auth: &AuthToken,
    ) -> Result<Invitation, Either<Forbidden, anyhow::Error>> {
        check_online(&self.mock_server_state)?;
        check_admin(&self.mock_auth_service_state, auth)?;

        let jid = BareJid::from_parts(Some(&username), &self.server_domain);
        let token = InvitationToken::from(random_secret(32));
        let created_at = OffsetDateTime::now_utc();
        let expires_at = created_at.saturating_add(ttl.unwrap_or(self.invitations_ttl));
        let invitation = Invitation {
            id: token.clone(),
            created_at,
            jid,
            pre_assigned_role: role,
            email_address: email_address.clone(),
            accept_token_expires_at: expires_at,
            reject_token: token.clone(),
            accept_token: token,
        };

        self.state_mut()
            .invitations
            .insert(email_address, invitation.clone());

        Ok(invitation)
    }

    async fn list_account_invitations(
        &self,
        auth: &AuthToken,
    ) -> Result<Vec<Invitation>, Either<Forbidden, anyhow::Error>> {
        check_online(&self.mock_server_state)?;
        check_admin(&self.mock_auth_service_state, auth)?;

        let invitations = (self.state().invitations)
            .values()
            .map(ToOwned::to_owned)
            .collect();

        Ok(invitations)
    }

    async fn account_invitations_stats(
        &self,
        auth: Option<&AuthToken>,
    ) -> Result<InvitationsStats, anyhow::Error> {
        check_online(&self.mock_server_state)?;
        if let Some(auth) = auth {
            check_admin(&self.mock_auth_service_state, auth)?;
        }

        Ok(InvitationsStats {
            count: self.state().invitations.len(),
        })
    }

    async fn get_account_invitation_by_username(
        &self,
        username: &NodeRef,
        auth: &AuthToken,
    ) -> Result<Option<Invitation>, Either<Forbidden, anyhow::Error>> {
        check_online(&self.mock_server_state)?;
        check_admin(&self.mock_auth_service_state, auth)?;

        let invitation = (self.state().invitations)
            .values()
            .find(|invitation| invitation.jid.node() == Some(username))
            .map(ToOwned::to_owned);

        Ok(invitation)
    }

    async fn get_account_invitation_by_id(
        &self,
        id: &InvitationId,
        auth: &AuthToken,
    ) -> Result<Option<Invitation>, Either<Forbidden, anyhow::Error>> {
        check_online(&self.mock_server_state)?;
        check_admin(&self.mock_auth_service_state, auth)?;

        let invitation = (self.state().invitations)
            .values()
            .find(|invitation| invitation.id.eq(id))
            .map(ToOwned::to_owned);

        Ok(invitation)
    }

    async fn get_account_invitation_by_token(
        &self,
        token: &InvitationToken,
    ) -> Result<Option<Invitation>, anyhow::Error> {
        check_online(&self.mock_server_state)?;

        let invitation = (self.state().invitations)
            .values()
            .find(|invitation| invitation.accept_token.eq(token))
            .map(ToOwned::to_owned);

        match invitation.as_ref() {
            Some(invitation) if invitation.is_expired() => Ok(None),
            Some(_) => Ok(invitation),
            None => Ok(None),
        }
    }

    async fn delete_invitation(
        &self,
        token: InvitationToken,
        auth: &AuthToken,
    ) -> Result<(), Either<Forbidden, anyhow::Error>> {
        check_online(&self.mock_server_state)?;
        check_admin(&self.mock_auth_service_state, auth)?;

        self.delete_invitation_(token);

        Ok(())
    }
}

// MARK: - Boilerplate

impl MockInvitationRepository {
    #[inline]
    pub(crate) fn state(&self) -> RwLockReadGuard<'_, MockInvitationRepositoryState> {
        self.state.read().unwrap()
    }

    #[inline]
    pub(crate) fn state_mut(&self) -> RwLockWriteGuard<'_, MockInvitationRepositoryState> {
        self.state.write().unwrap()
    }

    #[inline]
    pub(crate) fn invitation_for_email(&self, email_address: &EmailAddress) -> Option<Invitation> {
        self.state().invitations.get(email_address).cloned()
    }

    pub(crate) fn delete_invitation_(&self, token: InvitationToken) {
        let key = (self.state().invitations)
            .iter()
            .find_map(|(key, invitation)| {
                if invitation.accept_token == token {
                    Some(key.clone())
                } else {
                    None
                }
            });
        if let Some(key) = key {
            self.state_mut().invitations.remove(&key);
        }
    }
}
