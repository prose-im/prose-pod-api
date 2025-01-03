// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use base64::engine::{general_purpose::STANDARD_NO_PAD as Base64, Engine as _};
use secrecy::{ExposeSecret as _, SecretString};
use service::{
    auth::{auth_service, AuthServiceImpl, AuthToken, UserInfo},
    models::BareJid,
};

use std::sync::{Arc, RwLock};

use super::mock_server_ctl::MockServerCtlState;

#[derive(Debug, Clone)]
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

    fn check_online(&self) -> Result<(), auth_service::Error> {
        if self.state.read().unwrap().online {
            Ok(())
        } else {
            Err(auth_service::Error::Other("XMPP server offline".to_owned()))?
        }
    }

    pub fn log_in_unchecked(&self, jid: &BareJid) -> Result<AuthToken, auth_service::Error> {
        let json = serde_json::to_string(&UserInfo { jid: jid.clone() }).unwrap();
        let base64 = Base64.encode(json);
        let token = SecretString::new(base64);

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
    ) -> Result<AuthToken, auth_service::Error> {
        self.check_online()?;

        let state = self.mock_server_ctl_state.read().unwrap();
        let valid_credentials = state
            .users
            .get(jid)
            .map(|user| user.password.expose_secret() == password.expose_secret())
            .expect("User must be created first");

        if !valid_credentials {
            Err(auth_service::Error::InvalidCredentials)?
        }

        self.log_in_unchecked(jid)
    }

    async fn get_user_info(&self, token: AuthToken) -> Result<UserInfo, auth_service::Error> {
        let base64 = token.expose_secret();
        let json = Base64.decode(base64).map_err(|err| {
            auth_service::Error::Other(format!("Could Base64-decode test token: {err}"))
        })?;
        let json = String::from_utf8(json).map_err(|err| {
            auth_service::Error::Other(format!("Test token is not valid UTF-8: {err}"))
        })?;
        serde_json::from_str(&json).map_err(|err| {
            auth_service::Error::Other(format!("Could not parse data from test token: {err}"))
        })
    }

    async fn register_oauth2_client(&self) -> Result<(), auth_service::Error> {
        // NOTE: Nothing to do in tests
        Ok(())
    }
}
