// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use ::migration::DbErr;
use ::service::sea_orm::DatabaseConnection;
use ::service::server_ctl::{Error, ServerCtlImpl};
use ::service::{prosody_config_from_db, ProsodyConfigFile, Query};
use entity::model::JID;
use linked_hash_map::LinkedHashMap;
use log::error;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct DummyServerCtl {
    state: Arc<Mutex<DummyServerCtlState>>,
}

#[derive(Debug)]
pub struct UserAccount {
    pub jid: JID,
    pub password: String,
}

#[derive(Debug, Default)]
pub struct DummyServerCtlState {
    pub conf_reload_count: usize,
    pub applied_config: Option<ProsodyConfigFile>,
    pub users: LinkedHashMap<JID, UserAccount>,
}

impl DummyServerCtlState {
    pub async fn new(db: &DatabaseConnection) -> Result<Self, DbErr> {
        let server_config = Query::server_config(db)
            .await?
            .expect("Workspace not initialized");
        let prosody_config = prosody_config_from_db(server_config);
        Ok(Self {
            conf_reload_count: 0,
            applied_config: Some(prosody_config),
            users: LinkedHashMap::default(),
        })
    }
}

impl DummyServerCtl {
    pub fn new(state: Arc<Mutex<DummyServerCtlState>>) -> Self {
        Self { state }
    }
}

impl ServerCtlImpl for DummyServerCtl {
    fn start(&self) -> Result<(), Error> {
        error!("DummyServerCtl `start` not implemented");
        todo!("DummyServerCtl `start`")
    }

    fn stop(&self) -> Result<(), Error> {
        error!("DummyServerCtl `stop` not implemented");
        todo!("DummyServerCtl `stop`")
    }

    fn restart(&self) -> Result<(), Error> {
        error!("DummyServerCtl `restart` not implemented");
        todo!("DummyServerCtl `restart`")
    }

    fn reload(&self) -> Result<(), Error> {
        let mut state = self.state.lock().unwrap();
        state.conf_reload_count += 1;
        Ok(())
    }

    fn status(&self) -> Result<(), Error> {
        error!("DummyServerCtl `status` not implemented");
        todo!("DummyServerCtl `status`")
    }

    fn add_user(&self, jid: &JID, password: &str) -> Result<(), Error> {
        let mut state = self.state.lock().unwrap();
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
        let mut state = self.state.lock().unwrap();
        state.users.remove(&jid);
        Ok(())
    }
}
