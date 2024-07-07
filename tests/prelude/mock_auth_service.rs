// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::prose_xmpp::BareJid;
use service::services::{
    auth_service::{self, AuthServiceImpl, JWT_PROSODY_TOKEN_KEY},
    jwt_service::{self, JWTService},
};

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use crate::mock_server_ctl::MockServerCtlState;

#[derive(Debug, Clone)]
pub struct MockAuthService {
    jwt_service: JWTService,
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

    pub fn log_in_unchecked(&self, jid: &BareJid) -> Result<String, auth_service::Error> {
        let token = self.jwt_service.generate_jwt(jid, |claims| {
            claims.insert(JWT_PROSODY_TOKEN_KEY, "dummy-prosody-token".to_owned());
        })?;

        Ok(token)
    }
}

impl Default for MockAuthServiceState {
    fn default() -> Self {
        Self { online: true }
    }
}

impl AuthServiceImpl for MockAuthService {
    fn log_in(&self, jid: &BareJid, password: &str) -> Result<String, auth_service::Error> {
        self.check_online()?;

        let state = self.mock_server_ctl_state.read().unwrap();
        let valid_credentials = state
            .users
            .get(jid)
            .map(|user| user.password == password)
            .expect("User must be created first");

        if !valid_credentials {
            Err(auth_service::Error::InvalidCredentials)?
        }

        self.log_in_unchecked(jid)
    }
    fn verify(&self, jwt: &str) -> Result<BTreeMap<String, String>, jwt_service::Error> {
        self.jwt_service.verify(jwt)
    }
}
