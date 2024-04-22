// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use ::entity::model::JID;
use ::migration::DbErr;
use ::service::sea_orm::DatabaseConnection;
use ::service::server_ctl::{Error, ServerCtlImpl};
use ::service::vcard_parser::vcard::Vcard;
use ::service::{prosody_config_from_db, ProsodyConfigFile, Query};
use entity::model::MemberRole;
use linked_hash_map::LinkedHashMap;

use std::sync::Mutex;

#[derive(Debug)]
pub struct DummyServerCtl {
    pub(crate) state: Mutex<DummyServerCtlState>,
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
    pub vcards: LinkedHashMap<JID, Vcard>,
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
            vcards: LinkedHashMap::default(),
        })
    }
}

impl DummyServerCtl {
    pub fn new(state: Mutex<DummyServerCtlState>) -> Self {
        Self { state }
    }
}

impl ServerCtlImpl for DummyServerCtl {
    fn reload(&self) -> Result<(), Error> {
        let mut state = self.state.lock().unwrap();
        state.conf_reload_count += 1;
        Ok(())
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
    fn set_user_role(&self, _jid: &JID, _role: &MemberRole) -> Result<(), Error> {
        // NOTE: The role is stored on our side in the database,
        //   our `DummyServerCtl` has nothing to save.
        Ok(())
    }

    fn test_user_password(&self, jid: &JID, password: &str) -> Result<bool, Error> {
        let state = self.state.lock().unwrap();
        Ok(state
            .users
            .get(jid)
            .map(|user| user.password == password)
            .expect("User must be created first"))
    }

    fn get_vcard(&self, jid: &JID) -> Result<Option<Vcard>, Error> {
        Ok(self.state.lock().unwrap().vcards.get(jid).map(Clone::clone))
    }
    fn set_vcard(&self, jid: &JID, vcard: &Vcard) -> Result<(), Error> {
        self.state
            .lock()
            .unwrap()
            .vcards
            .insert(jid.clone(), vcard.clone());
        Ok(())
    }
}
