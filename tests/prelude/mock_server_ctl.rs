// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use ::entity::model::JID;
use ::service::server_ctl::{Error, ServerCtlImpl};
use ::service::{prosody_config_from_db, ProsodyConfigSection};
use entity::model::MemberRole;
use entity::server_config;
use linked_hash_map::LinkedHashMap;
use service::config::Config;
use service::prosody::ProsodyConfig;

use std::collections::HashSet;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct MockServerCtl {
    pub(crate) online: bool,
    pub(crate) state: Arc<RwLock<MockServerCtlState>>,
}

#[derive(Debug)]
pub struct UserAccount {
    pub jid: JID,
    pub password: String,
}

#[derive(Debug, Default)]
pub struct MockServerCtlState {
    pub conf_reload_count: usize,
    pub applied_config: Option<ProsodyConfig>,
    pub users: LinkedHashMap<JID, UserAccount>,
    pub online: HashSet<JID>,
}

impl MockServerCtl {
    pub fn new(state: Arc<RwLock<MockServerCtlState>>) -> Self {
        Self {
            online: true,
            state,
        }
    }

    fn check_online(&self) -> Result<(), Error> {
        if self.online {
            Ok(())
        } else {
            Err(Error::Other("XMPP server offline".to_owned()))?
        }
    }
}

impl Default for MockServerCtl {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl ServerCtlImpl for MockServerCtl {
    fn save_config(
        &self,
        server_config: &server_config::Model,
        app_config: &Config,
    ) -> Result<(), Error> {
        self.check_online()?;

        let mut state = self.state.write().unwrap();
        state.applied_config = Some(prosody_config_from_db(server_config.to_owned(), app_config));
        Ok(())
    }
    fn reload(&self) -> Result<(), Error> {
        self.check_online()?;

        let mut state = self.state.write().unwrap();
        state.conf_reload_count += 1;
        Ok(())
    }

    fn add_user(&self, jid: &JID, password: &str) -> Result<(), Error> {
        self.check_online()?;

        let mut state = self.state.write().unwrap();

        // Check that the domain exists in the Prosody configuration. If it's not the case,
        // Prosody won't add the user. This happens if the server config wasn't initialized
        // and Prosody wasn't reloaded with a full configuration.
        let domain_exists = state.applied_config.as_ref().is_some_and(|config| {
            config
                .additional_sections
                .iter()
                .any(|section| match section {
                    ProsodyConfigSection::VirtualHost { hostname, .. } => hostname == &jid.domain,
                    _ => false,
                })
        });
        if !domain_exists {
            return Err(Error::Other(format!("Domain {} not declared in the Prosody configuration. You might need to initialize the server configuration.", &jid.domain)));
        }

        state.users.insert(
            jid.clone(),
            UserAccount {
                jid: jid.clone(),
                password: password.to_string(),
            },
        );
        Ok(())
    }
    fn remove_user(&self, jid: &JID) -> Result<(), Error> {
        self.check_online()?;

        let mut state = self.state.write().unwrap();
        state.users.remove(&jid);
        Ok(())
    }
    fn set_user_role(&self, _jid: &JID, _role: &MemberRole) -> Result<(), Error> {
        self.check_online()?;

        // NOTE: The role is stored on our side in the database,
        //   our `DummyServerCtl` has nothing to save.
        Ok(())
    }
}
