// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use log::error;
use migration::DbErr;
use service::server_ctl::{Error, ServerCtlImpl};
use service::{prosody_config_from_db, ProseDefault, ProsodyConfig};
use service::{sea_orm::DatabaseConnection, Query};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct DummyServerCtl {
    state: Arc<Mutex<DummyServerCtlState>>,
}

#[derive(Debug)]
pub struct DummyServerCtlState {
    pub conf_reload_count: usize,
    pub applied_config: ProsodyConfig,
}

impl DummyServerCtlState {
    pub async fn new(db: &DatabaseConnection) -> Result<Self, DbErr> {
        let server_config = Query::server_config(db)
            .await?
            .expect("Workspace not initialized");
        let prosody_config = prosody_config_from_db(server_config);
        Ok(Self {
            conf_reload_count: 0,
            applied_config: prosody_config,
        })
    }
}

impl Default for DummyServerCtlState {
    fn default() -> Self {
        Self {
            conf_reload_count: 0,
            applied_config: ProsodyConfig::prose_default(),
        }
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

    fn add_user(&self) -> Result<(), Error> {
        error!("DummyServerCtl `add_user` not implemented");
        todo!("DummyServerCtl `add_user`")
    }

    fn set_user_password(&self) -> Result<(), Error> {
        error!("DummyServerCtl `set_user_password` not implemented");
        todo!("DummyServerCtl `set_user_password`")
    }

    fn remove_user(&self) -> Result<(), Error> {
        error!("DummyServerCtl `remove_user` not implemented");
        todo!("DummyServerCtl `remove_user`")
    }
}
