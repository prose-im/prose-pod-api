// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use secrecy::{ExposeSecret as _, SecretString};
use service::{
    auth::{auth_service, AuthServiceImpl, AuthToken, UserInfo},
    models::BareJid,
};

use std::sync::{Arc, RwLock};

use crate::mock_server_ctl::MockServerCtlState;

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
        let token =
            SecretString::new(serde_json::to_string(&UserInfo { jid: jid.clone() }).unwrap());

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
        serde_json::from_str(&token.expose_secret()).map_err(|err| {
            auth_service::Error::Other(format!("Could not parse data from test token: {err}"))
        })
    }

    async fn register_oauth2_client(&self) -> Result<(), auth_service::Error> {
        // NOTE: Nothing to do in tests
        Ok(())
    }
}
