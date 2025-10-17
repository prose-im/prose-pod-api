// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use service::{auth::auth_service::prelude::*, members::MemberRole};

use super::prelude::*;

/// Bypass token made for initializatin purposes.
pub static BYPASS_TOKEN: LazyLock<AuthToken> =
    LazyLock::new(|| AuthToken(SecretString::from("BYPASS")));

#[derive(Debug, Clone)]
pub struct MockAuthService {
    pub server: Arc<MockServerService>,
    pub state: Arc<RwLock<MockAuthServiceState>>,
    pub mock_user_repository: Arc<MockUserRepository>,
    pub password_reset_tokens_ttl: Duration,
    pub min_password_length: u8,
    pub server_domain: JidDomain,
}

#[derive(Debug, Default)]
pub struct MockAuthServiceState {
    pub sessions: HashMap<AuthToken, SessionInfo>,
    pub expired_sessions: HashMap<AuthToken, SessionInfo>,
    pub password_reset_tokens: LinkedHashMap<PasswordResetToken, PasswordResetRequestInfo>,
    pub expired_password_reset_tokens: LinkedHashMap<PasswordResetToken, PasswordResetRequestInfo>,
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub jid: BareJid,
    pub role: ProsodyRoleName,
}

impl MockAuthService {
    pub(crate) fn state(&self) -> RwLockReadGuard<'_, MockAuthServiceState> {
        self.state.read().unwrap()
    }
    pub(crate) fn state_mut(&self) -> RwLockWriteGuard<'_, MockAuthServiceState> {
        self.state.write().unwrap()
    }

    pub(crate) async fn log_in_unchecked(&self, jid: &BareJid) -> Result<AuthToken, anyhow::Error> {
        let role = self
            .mock_user_repository
            .raw_role(jid)
            .expect(&jid_missing(jid));
        let session = SessionInfo {
            jid: jid.to_owned(),
            role,
        };

        let token = AuthToken(random_secret(32));

        (self.state.write().unwrap())
            .sessions
            .insert(token.clone(), session);

        Ok(token)
    }

    pub(crate) fn password_reset_requests(&self, jid: &BareJid) -> Vec<PasswordResetRequestInfo> {
        self.state()
            .password_reset_tokens
            .values()
            .filter(|req| req.jid.eq(jid))
            .map(ToOwned::to_owned)
            .collect()
    }

    pub(crate) fn expired_password_reset_requests(
        &self,
        jid: &BareJid,
    ) -> Vec<PasswordResetRequestInfo> {
        self.state()
            .expired_password_reset_tokens
            .values()
            .filter(|req| req.jid.eq(jid))
            .map(ToOwned::to_owned)
            .collect()
    }

    #[allow(unused)]
    pub(crate) fn check_admin(
        &self,
        auth: &AuthToken,
    ) -> Result<(), Either<Forbidden, anyhow::Error>> {
        check_admin(&self.state, auth)
    }
}

pub(crate) fn check_admin(
    auth_service_state: &Arc<RwLock<MockAuthServiceState>>,
    auth: &AuthToken,
) -> Result<(), Either<Forbidden, anyhow::Error>> {
    if auth == BYPASS_TOKEN.deref() {
        return Ok(());
    }

    let state = auth_service_state
        .read()
        .expect("Auth service state poisoned");
    match state.sessions.get(auth) {
        Some(session) if MemberRole::try_from(&session.role) == Ok(MemberRole::Admin) => Ok(()),
        Some(session) => Err(Either::E1(Forbidden(format!(
            "'{}' is not an admin.",
            session.jid
        )))),
        None => Err(Either::E2(anyhow::Error::new(Unauthorized(
            "Not authenticated.".to_owned(),
        )))),
    }
}

#[async_trait]
impl AuthServiceImpl for MockAuthService {
    async fn log_in(
        &self,
        jid: &BareJid,
        password: &SecretString,
    ) -> Result<AuthToken, Either<InvalidCredentials, anyhow::Error>> {
        self.server.check_online()?;

        let valid_credentials = {
            use secrecy::ExposeSecret as _;
            let user_password = self.mock_user_repository.password(jid).expect(USER_MISSING);
            password.expose_secret() == user_password.expose_secret()
        };
        if !valid_credentials {
            return Err(Either::E1(InvalidCredentials));
        }

        let token = self.log_in_unchecked(jid).await?;

        Ok(token)
    }

    async fn get_user_info(
        &self,
        token: &AuthToken,
    ) -> Result<UserInfo, Either<Unauthorized, anyhow::Error>> {
        self.server.check_online()?;

        match self.state().sessions.get(&token) {
            Some(session) => Ok(UserInfo::try_from(session).context("Not a user account")?),
            None => Err(Either::E1(Unauthorized("Bad auth token.".to_owned()))),
        }
    }

    async fn revoke(&self, token: AuthToken) -> Result<(), anyhow::Error> {
        self.server.check_online()?;

        {
            let mut state = self.state_mut();
            if let Some(info) = state.sessions.remove(&token) {
                state.expired_sessions.insert(token, info);
            };
        }

        Ok(())
    }

    async fn create_password_reset_token(
        &self,
        username: &NodeRef,
        ttl: Option<Duration>,
        _contact: &InvitationContact,
        auth: &AuthToken,
    ) -> Result<PasswordResetRequestInfo, Either<Forbidden, anyhow::Error>> {
        self.server.check_online()?;

        match self.get_user_info(auth).await {
            Ok(caller) => {
                if !caller.is_admin() {
                    return Err(Either::E1(Forbidden("Not an admin.".to_owned())));
                }
            }
            Err(Either::E1(Unauthorized(msg))) => {
                return Err(Either::E1(Forbidden(msg)));
            }
            Err(Either::E2(err)) => {
                return Err(Either::E2(err));
            }
        }

        let token = PasswordResetToken::from(random_secret(32));

        let jid = BareJid::from_parts(Some(username), &self.server_domain);
        let created_at = Utc::now();
        let ttl = ttl.unwrap_or(self.password_reset_tokens_ttl.clone());
        let expires_at = created_at
            .checked_add_signed(TimeDelta::from_std(ttl).unwrap())
            .unwrap();
        let info = PasswordResetRequestInfo {
            jid,
            token: token.clone(),
            created_at,
            expires_at,
        };

        self.state_mut()
            .password_reset_tokens
            .insert(token, info.clone());

        Ok(info)
    }

    fn validate_password(&self, password: &Password) -> Result<(), PasswordValidationError> {
        auth_service::validate_password(password, self.min_password_length)
    }

    async fn reset_password(
        &self,
        token: PasswordResetToken,
        password: &Password,
    ) -> Result<
        (),
        Either4<PasswordValidationError, PasswordResetTokenExpired, Forbidden, anyhow::Error>,
    > {
        self.server.check_online()?;
        self.validate_password(password).map_err(Either4::E1)?;

        let jid = {
            let state = self.state();
            let reset_request = state.password_reset_tokens.get(&token);

            let Some(reset_request) = reset_request else {
                return Err(Either4::E2(PasswordResetTokenExpired));
            };

            if reset_request.is_expired() {
                return Err(Either4::E2(PasswordResetTokenExpired));
            }

            reset_request.jid.to_owned()
        };

        self.mock_user_repository.set_password(&jid, password);

        {
            let mut state = self.state_mut();
            if let Some(info) = state.password_reset_tokens.remove(&token) {
                state.expired_password_reset_tokens.insert(token, info);
            };
        }

        Ok(())
    }
}

// MARK: - Boilerplate

impl TryFrom<&SessionInfo> for UserInfo {
    type Error = service::prosody::UnsupportedProsodyRole;

    fn try_from(session: &SessionInfo) -> Result<Self, Self::Error> {
        let role = MemberRole::try_from(&session.role)?;

        Ok(UserInfo {
            jid: session.jid.clone(),
            primary_role: role,
        })
    }
}
