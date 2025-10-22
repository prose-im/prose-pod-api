// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    prose_pod_server_service::*,
    prosody::{prosody_config_from_db, ProsodyConfig},
    ServerConfig,
};

use super::prelude::*;

#[derive(Debug)]
pub struct MockServerService {
    pub state: Arc<RwLock<MockServerServiceState>>,
    pub db: DatabaseRwConnectionPools,
    pub mock_auth_service_state: Arc<RwLock<MockAuthServiceState>>,
    pub mock_user_repository: Arc<MockUserRepository>,
}

#[derive(Debug)]
pub struct MockServerServiceState {
    pub online: bool,
    pub conf_reload_count: usize,
    pub applied_config: Option<Arc<ProsodyConfig>>,
}

impl Default for MockServerServiceState {
    fn default() -> Self {
        Self {
            online: true,
            conf_reload_count: 0,
            applied_config: None,
        }
    }
}

impl MockServerService {
    pub fn check_online(&self) -> Result<(), anyhow::Error> {
        check_online(&self.state)
    }
}

pub fn check_online(
    mock_server_state: &Arc<RwLock<MockServerServiceState>>,
) -> Result<(), anyhow::Error> {
    let state = mock_server_state.read().expect("Server state poisoned");
    if state.online {
        Ok(())
    } else {
        Err(anyhow::Error::msg("Prose Pod Server offline"))
    }
}

#[async_trait]
impl ProsePodServerServiceImpl for MockServerService {
    async fn wait_until_ready(&self) -> Result<(), Either<InvalidConfiguration, anyhow::Error>> {
        self.check_online().map_err(Either::E2)
    }

    async fn save_config(
        &self,
        server_config: &ServerConfig,
        app_config: &AppConfig,
        auth: Option<&AuthToken>,
    ) -> Result<(), anyhow::Error> {
        self.check_online()?;
        if let Some(auth) = auth {
            check_admin(&self.mock_auth_service_state, auth)?;
        }

        let admins: Vec<BareJid> = self
            .mock_user_repository
            .state()
            .users
            .values()
            .filter_map(|account| {
                if account.role.as_str() == "prosody:admin" {
                    Some(account.jid.clone())
                } else {
                    None
                }
            })
            .collect();

        let new_config = prosody_config_from_db(
            &self.db.read,
            app_config,
            Some(server_config.to_owned()),
            admins,
        )
        .await?;

        let mut state = self.state.write().unwrap();
        state.applied_config = Some(Arc::new(new_config));
        Ok(())
    }

    async fn reload(&self, auth: Option<&AuthToken>) -> Result<(), anyhow::Error> {
        self.check_online()?;
        if let Some(auth) = auth {
            check_admin(&self.mock_auth_service_state, auth)?;
        }

        let mut state = self.state.write().unwrap();
        state.conf_reload_count += 1;
        Ok(())
    }

    async fn delete_all_data(&self, auth: &AuthToken) -> Result<(), anyhow::Error> {
        self.check_online()?;
        check_admin(&self.mock_auth_service_state, auth)?;

        // We don't care in tests for now
        Ok(())
    }
}
