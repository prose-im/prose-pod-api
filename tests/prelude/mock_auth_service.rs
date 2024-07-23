// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use secrecy::{ExposeSecret as _, SecretString};
use service::prose_xmpp::BareJid;
use service::services::{
    auth_service::{self, AuthServiceImpl, JWT_PROSODY_TOKEN_KEY},
    jwt_service::JWT,
    jwt_service::{self, JWTService},
};

use std::sync::{Arc, RwLock};

use crate::mock_server_ctl::MockServerCtlState;

#[derive(Debug, Clone)]
pub struct MockAuthService {
    pub(crate) jwt_service: JWTService,
    pub(crate) state: Arc<RwLock<MockAuthServiceState>>,
    pub mock_server_ctl_state: Arc<RwLock<MockServerCtlState>>,
}

#[derive(Debug)]
pub struct MockAuthServiceState {
    pub online: bool,
}

impl MockAuthService {
    pub fn new(
        jwt_service: JWTService,
        state: Arc<RwLock<MockAuthServiceState>>,
        mock_server_ctl_state: Arc<RwLock<MockServerCtlState>>,
    ) -> Self {
        Self {
            jwt_service,
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

    pub fn log_in_unchecked(&self, jid: &BareJid) -> Result<SecretString, auth_service::Error> {
        let token = self.jwt_service.generate_jwt(jid, |claims| {
            claims.insert(JWT_PROSODY_TOKEN_KEY.into(), "dummy-prosody-token".into());
        })?;

        Ok(token)
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
    ) -> Result<SecretString, auth_service::Error> {
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
    fn verify(&self, jwt: &SecretString) -> Result<JWT, jwt_service::Error> {
        JWT::try_from(jwt, &self.jwt_service)
    }
}
