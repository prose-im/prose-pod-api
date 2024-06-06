// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use ::entity::model::JID;
use service::{AuthError, AuthServiceImpl, JWTError, JWTService};

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, RwLock};

use crate::mock_server_ctl::MockServerCtlState;

#[derive(Debug)]
pub struct MockAuthService {
    pub online: bool,
    jwt_service: Arc<RwLock<JWTService>>,
    pub mock_server_ctl_state: Arc<Mutex<MockServerCtlState>>,
}

impl MockAuthService {
    pub fn new(
        jwt_service: Arc<RwLock<JWTService>>,
        mock_server_ctl_state: Arc<Mutex<MockServerCtlState>>,
    ) -> Self {
        Self {
            online: true,
            jwt_service,
            mock_server_ctl_state,
        }
    }

    fn check_online(&self) -> Result<(), AuthError> {
        if self.online {
            Ok(())
        } else {
            Err(AuthError::Other("XMPP server offline".to_owned()))?
        }
    }
}

impl AuthServiceImpl for MockAuthService {
    fn log_in(&self, jid: &JID, password: &str) -> Result<String, AuthError> {
        self.check_online()?;

        let state = self.mock_server_ctl_state.lock().unwrap();
        let valid_credentials = state
            .users
            .get(jid)
            .map(|user| user.password == password)
            .expect("User must be created first");

        if !valid_credentials {
            Err(AuthError::InvalidCredentials)?
        }

        let token = self.jwt_service.read().unwrap().generate_jwt_(jid)?;

        Ok(token)
    }
    fn verify(&self, jwt: &str) -> Result<BTreeMap<String, String>, JWTError> {
        self.jwt_service.read().unwrap().verify(jwt)
    }
}
