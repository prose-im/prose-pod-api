// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::anyhow;
use base64::{prelude::BASE64_STANDARD, Engine as _};
use secrecy::{ExposeSecret as _, SecretString};
use service::{
    auth::{
        errors::{InvalidAuthToken, InvalidCredentials},
        AuthServiceImpl, AuthToken, UserInfo,
    },
    members::MemberRole,
    models::BareJid,
    sea_orm::DatabaseConnection,
    util::either::Either,
};
use tracing::debug;

use std::sync::{Arc, RwLock};

use super::mock_server_ctl::MockServerCtlState;

#[derive(Debug)]
pub struct MockAuthService {
    pub(crate) state: Arc<RwLock<MockAuthServiceState>>,
    pub mock_server_ctl_state: Arc<RwLock<MockServerCtlState>>,
}

#[derive(Debug)]
pub struct MockAuthServiceState {
    pub online: bool,
}

impl MockAuthService {
    pub fn new(
        state: Arc<RwLock<MockAuthServiceState>>,
        mock_server_ctl_state: Arc<RwLock<MockServerCtlState>>,
    ) -> Self {
        Self {
            state,
            mock_server_ctl_state,
        }
    }

    fn check_online(&self) -> Result<(), anyhow::Error> {
        if self.state.read().unwrap().online {
            Ok(())
        } else {
            Err(anyhow!("XMPP server offline"))
        }
    }

    pub async fn log_in_unchecked(&self, jid: &BareJid) -> Result<AuthToken, anyhow::Error> {
        let role = {
            let state = self.mock_server_ctl_state.read().unwrap();
            (state.users.get(jid)).map_or(MemberRole::Member, |member| match member.role.as_str() {
                "prosody:admin" => MemberRole::Admin,
                "prosody:member" => MemberRole::Member,
                role => unreachable!("role={role}"),
            })
        };
        let json = serde_json::to_string(&UserInfo {
            jid: jid.to_owned(),
            role,
        })
        .unwrap();
        let base64 = BASE64_STANDARD.encode(json);
        let token = SecretString::from(base64);

        Ok(AuthToken(token))
    }
}

impl Default for MockAuthServiceState {
    fn default() -> Self {
        Self { online: true }
    }
}

#[async_trait::async_trait]
impl AuthServiceImpl for MockAuthService {
    async fn log_in(
        &self,
        jid: &BareJid,
        password: &SecretString,
    ) -> Result<AuthToken, Either<InvalidCredentials, anyhow::Error>> {
        self.check_online()?;

        let valid_credentials = {
            let state = self.mock_server_ctl_state.read().unwrap();
            (state.users.get(jid))
                .map(|user| user.password.expose_secret() == password.expose_secret())
                .expect("User must be created first")
        };

        if !valid_credentials {
            return Err(Either::E1(InvalidCredentials));
        }

        let token = self.log_in_unchecked(jid).await?;
        Ok(token)
    }

    async fn get_user_info(
        &self,
        token: AuthToken,
        _db: &DatabaseConnection,
    ) -> Result<UserInfo, Either<InvalidAuthToken, anyhow::Error>> {
        self.check_online()?;

        let base64 = token.expose_secret();
        let json = (BASE64_STANDARD.decode(base64)).map_err(|err| {
            debug!("Could not Base64-decode test token: {err}");
            Either::E1(InvalidAuthToken)
        })?;
        let json = String::from_utf8(json).map_err(|err| {
            debug!("Test token is not valid UTF-8: {err}");
            Either::E1(InvalidAuthToken)
        })?;
        serde_json::from_str(&json).map_err(|err| {
            debug!("Could not parse data from test token: {err}");
            Either::E1(InvalidAuthToken)
        })
    }

    async fn register_oauth2_client(&self) -> Result<(), anyhow::Error> {
        self.check_online()?;

        // NOTE: Nothing to do in tests
        Ok(())
    }
}
