// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    members::{user_repository::prelude::*, Nickname},
    prosody::UnsupportedProsodyRole,
    xmpp::{XmppServiceContext, XmppServiceImpl as _},
};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct MockUserRepository {
    pub state: Arc<RwLock<MockUserRepositoryState>>,
    pub mock_server_state: Arc<RwLock<MockServerServiceState>>,
    pub mock_auth_service_state: Arc<RwLock<MockAuthServiceState>>,
    pub mock_xmpp_service: Arc<MockXmppService>,
    pub server_domain: JidDomain,
}

#[derive(Debug, Default)]
pub struct MockUserRepositoryState {
    pub users: LinkedHashMap<BareJid, UserAccount>,
}

#[derive(Debug, Clone)]
pub struct UserAccount {
    pub jid: BareJid,
    pub password: Password,
    pub role: ProsodyRoleName,
    pub joined_at: DateTime<Utc>,
    pub email_address: Option<EmailAddress>,
}

impl UserAccount {
    pub fn member(jid: BareJid) -> Self {
        Self {
            password: Password::from("password"),
            role: MemberRole::Member.as_prosody(),
            joined_at: Utc::now(),
            email_address: Some(EmailAddress::from(&jid)),
            jid,
        }
    }

    pub fn admin(jid: BareJid) -> Self {
        Self {
            password: Password::from("password"),
            role: MemberRole::Admin.as_prosody(),
            joined_at: Utc::now(),
            email_address: Some(EmailAddress::from(&jid)),
            jid,
        }
    }
}

#[async_trait]
impl UserRepositoryImpl for MockUserRepository {
    async fn list_users(
        &self,
        auth: &AuthToken,
    ) -> Result<Vec<Member>, Either3<Unauthorized, Forbidden, anyhow::Error>> {
        check_online(&self.mock_server_state)?;
        check_admin(&self.mock_auth_service_state, auth)?;

        let members = self
            .state()
            .users
            .values()
            .flat_map(|account| Member::try_from(account).ok())
            .collect();

        Ok(members)
    }

    async fn get_user_by_username(
        &self,
        username: &NodeRef,
        auth: &AuthToken,
    ) -> Result<Option<Member>, Either<Forbidden, anyhow::Error>> {
        check_online(&self.mock_server_state)?;
        check_admin(&self.mock_auth_service_state, auth)?;

        let jid = BareJid::from_parts(Some(username), &self.server_domain);
        let user_opt = self
            .state()
            .users
            .get(&jid)
            .and_then(|account| Member::try_from(account).ok());

        Ok(user_opt)
    }

    async fn users_stats(&self, _auth: Option<&AuthToken>) -> Result<UsersStats, anyhow::Error> {
        check_online(&self.mock_server_state)?;

        let count = self.user_count();

        Ok(UsersStats { count })
    }

    async fn set_user_role(
        &self,
        username: &NodeRef,
        role: &MemberRole,
        auth: &AuthToken,
    ) -> Result<(), Either<Forbidden, anyhow::Error>> {
        check_online(&self.mock_server_state)?;
        check_admin(&self.mock_auth_service_state, auth)?;

        let jid = BareJid::from_parts(Some(username), &self.server_domain);

        let mut state = self.state.write().unwrap();
        state
            .users
            .get_mut(&jid)
            .expect(&format!(
                "Cannot set <{jid}>'s password: User must be created first."
            ))
            .role = role.as_prosody();

        Ok(())
    }

    async fn delete_user(
        &self,
        username: &NodeRef,
        auth: &AuthToken,
    ) -> Result<(), Either3<MemberNotFound, Forbidden, anyhow::Error>> {
        check_online(&self.mock_server_state)?;
        check_admin(&self.mock_auth_service_state, auth)?;

        let jid = BareJid::from_parts(Some(username), &self.server_domain);

        let mut state = self.state.write().unwrap();
        if state.users.remove(&jid).is_none() {
            return Err(Either3::E1(MemberNotFound(username.to_string())));
        };

        Ok(())
    }
}

// MARK: - Boilerplate

impl MockUserRepository {
    #[inline]
    pub(crate) fn state(&self) -> RwLockReadGuard<'_, MockUserRepositoryState> {
        self.state.read().unwrap()
    }

    #[inline]
    pub(crate) fn state_mut(&self) -> RwLockWriteGuard<'_, MockUserRepositoryState> {
        self.state.write().unwrap()
    }

    pub(crate) fn raw_role(&self, jid: &BareJid) -> Option<ProsodyRoleName> {
        self.state()
            .users
            .get(jid)
            .map(|member| member.role.clone())
    }

    pub(crate) fn role(&self, jid: &BareJid) -> Option<MemberRole> {
        self.state()
            .users
            .get(jid)
            .and_then(|member| MemberRole::try_from(&member.role).ok())
    }

    pub(crate) fn password(&self, jid: &BareJid) -> Option<Password> {
        self.state()
            .users
            .get(jid)
            .map(|member| member.password.clone())
    }

    pub(crate) fn set_password(&self, jid: &BareJid, password: &Password) {
        self.state_mut()
            .users
            .get_mut(jid)
            .expect(&jid_missing(jid))
            .password = password.clone().into();
    }

    pub(crate) async fn add_user(
        &self,
        nickname: &Nickname,
        account: UserAccount,
        mock_auth_service: &MockAuthService,
    ) -> Result<(), anyhow::Error> {
        let email_address = account.email_address.clone();

        // Create the user account.
        let jid = account.jid.clone();
        self.state_mut().users.insert(jid.clone(), account.clone());

        // Create a token for the user to simplify other steps.
        let token = mock_auth_service.log_in_unchecked(&jid).await?;
        mock_auth_service
            .state_mut()
            .sessions
            .insert(token, SessionInfo::from(account));

        // Creates the user’s vCard.
        let ctx = XmppServiceContext {
            bare_jid: jid.to_owned(),
            auth_token: BYPASS_TOKEN.clone(),
        };
        self.mock_xmpp_service
            .create_own_vcard(&ctx, nickname, email_address)
            .await
            .context("Could not create user vCard4")?;

        Ok(())
    }

    pub(crate) async fn add_service_account(
        &self,
        account: UserAccount,
    ) -> Result<(), anyhow::Error> {
        // Create the user account.
        let jid = account.jid.clone();
        self.state_mut().users.insert(jid.clone(), account.clone());

        Ok(())
    }

    pub(crate) fn user_count(&self) -> usize {
        let ref all_users = self.state().users;
        // I.e. not service accounts.
        let regular_users = all_users
            .iter()
            .filter(|(_, account)| account.role.as_str() != ProsodyRoleName::REGISTERED_RAW);
        regular_users.count()
    }
}

impl From<UserAccount> for SessionInfo {
    fn from(account: UserAccount) -> Self {
        Self {
            jid: account.jid,
            role: account.role,
        }
    }
}

impl TryFrom<&UserAccount> for Member {
    type Error = UnsupportedProsodyRole;

    fn try_from(account: &UserAccount) -> Result<Self, Self::Error> {
        Ok(Self {
            jid: account.jid.clone(),
            role: MemberRole::try_from(&account.role)?,
        })
    }
}
