// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use linked_hash_map::LinkedHashMap;
use secrecy::SecretString;
use service::{
    features::{
        members::MemberRole,
        server_config::ServerConfig,
        xmpp::{server_ctl::Error, ServerCtlImpl},
    },
    model::BareJid,
    prosody::ProsodyConfig,
    prosody_config_from_db, AppConfig, ProsodyConfigSection,
};

use std::sync::{Arc, RwLock};

#[derive(Debug, Default, Clone)]
pub struct MockServerCtl {
    pub(crate) state: Arc<RwLock<MockServerCtlState>>,
}

#[derive(Debug, Clone)]
pub struct UserAccount {
    pub password: SecretString,
}

#[derive(Debug, Clone)]
pub struct MockServerCtlState {
    pub online: bool,
    pub conf_reload_count: usize,
    pub applied_config: Option<ProsodyConfig>,
    pub users: LinkedHashMap<BareJid, UserAccount>,
}

impl MockServerCtl {
    pub fn new(state: Arc<RwLock<MockServerCtlState>>) -> Self {
        Self { state }
    }

    fn check_online(&self) -> Result<(), Error> {
        if self.state.read().unwrap().online {
            Ok(())
        } else {
            Err(Error::Other("XMPP server offline".to_owned()))?
        }
    }
}

impl Default for MockServerCtlState {
    fn default() -> Self {
        Self {
            online: true,
            conf_reload_count: Default::default(),
            applied_config: Default::default(),
            users: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl ServerCtlImpl for MockServerCtl {
    async fn save_config(
        &self,
        server_config: &ServerConfig,
        app_config: &AppConfig,
    ) -> Result<(), Error> {
        self.check_online()?;

        let mut state = self.state.write().unwrap();
        state.applied_config = Some(prosody_config_from_db(server_config.to_owned(), app_config));
        Ok(())
    }
    async fn reload(&self) -> Result<(), Error> {
        self.check_online()?;

        let mut state = self.state.write().unwrap();
        state.conf_reload_count += 1;
        Ok(())
    }

    async fn add_user(&self, jid: &BareJid, password: &SecretString) -> Result<(), Error> {
        self.check_online()?;

        let mut state = self.state.write().unwrap();

        // Check that the domain exists in the Prosody configuration. If it's not the case,
        // Prosody won't add the user. This happens if the server config wasn't initialized
        // and Prosody wasn't reloaded with a full configuration.
        let domain_exists = jid.domain().as_str() == "admin.prose.org.local"
            || state.applied_config.as_ref().is_some_and(|config| {
                config
                    .additional_sections
                    .iter()
                    .any(|section| match section {
                        ProsodyConfigSection::VirtualHost { hostname, .. } => {
                            hostname == &jid.domain().to_string()
                        }
                        _ => false,
                    })
            });
        if !domain_exists {
            return Err(Error::Other(format!("Domain {} not declared in the Prosody configuration. You might need to initialize the server configuration.", jid.domain())));
        }

        state.users.insert(
            jid.clone(),
            UserAccount {
                password: password.to_owned(),
            },
        );
        Ok(())
    }
    async fn remove_user(&self, jid: &BareJid) -> Result<(), Error> {
        self.check_online()?;

        let mut state = self.state.write().unwrap();
        state.users.remove(jid);
        Ok(())
    }

    async fn set_user_role(&self, _jid: &BareJid, _role: &MemberRole) -> Result<(), Error> {
        self.check_online()?;

        // NOTE: The role is stored on our side in the database,
        //   our `DummyServerCtl` has nothing to save.
        Ok(())
    }
    async fn set_user_password(&self, jid: &BareJid, password: &SecretString) -> Result<(), Error> {
        self.check_online()?;

        let mut state = self.state.write().unwrap();
        state
            .users
            .get_mut(jid)
            .expect(&format!(
                "`MockServerCtl` cannot set <{jid}>'s password: User must be created first."
            ))
            .password = password.to_owned();

        Ok(())
    }

    async fn add_team_member(&self, _jid: &BareJid) -> Result<(), Error> {
        self.check_online()?;

        // We don't care in tests for now
        Ok(())
    }
    async fn remove_team_member(&self, _jid: &BareJid) -> Result<(), Error> {
        self.check_online()?;

        // We don't care in tests for now
        Ok(())
    }
}
