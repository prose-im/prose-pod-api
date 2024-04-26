// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::{
    model::{MemberRole, JID},
    server_config,
};
use std::{
    fs::File,
    io::Write as _,
    path::PathBuf,
    process::{self, Command},
    str,
};
use tempfile::NamedTempFile;
use vcard_parser::vcard::Vcard;

use crate::{config::Config, prosody_config_from_db, server_ctl::*};

/// Rust interface to [`prosodyctl`](https://prosody.im/doc/prosodyctl).
#[derive(Debug, Default)]
pub struct ProsodyCtl {
    config_file_path: PathBuf,
}

impl ProsodyCtl {
    pub fn new(config_file_path: PathBuf) -> Self {
        Self { config_file_path }
    }

    fn config_args(&self) -> Vec<String> {
        vec![
            "--config".to_string(),
            self.config_file_path.display().to_string(),
        ]
    }

    fn run_prosodyctl<S: ToString>(&self, args: Vec<S>) -> Result<process::Output, Error> {
        let mut args: Vec<String> = args.iter().map(ToString::to_string).collect();
        args.extend(self.config_args().into_iter());
        let output = Command::new("prosodyctl").args(args).output()?;
        if output.status.success() {
            Ok(output)
        } else {
            Err(Error::CommandFailed(output))
        }
    }
}

impl ServerCtlImpl for ProsodyCtl {
    fn save_config(
        &self,
        server_config: &server_config::Model,
        app_config: &Config,
    ) -> Result<(), Error> {
        let mut file = File::create(&self.config_file_path)?;
        file.write_all(
            prosody_config_from_db(server_config.to_owned(), app_config)
                .to_string()
                .as_bytes(),
        )?;
        Ok(())
    }
    fn reload(&self) -> Result<(), Error> {
        self.run_prosodyctl(vec!["reload"]).map(|_| ())
    }

    fn add_user(&self, jid: &JID, password: &str) -> Result<(), Error> {
        self.run_prosodyctl(vec![
            "register",
            jid.node.as_str(),
            jid.domain.as_str(),
            password,
        ])
        .map(|_| ())
    }
    fn remove_user(&self, jid: &JID) -> Result<(), Error> {
        self.run_prosodyctl(vec![
            "deluser",
            jid.to_string().as_str(),
        ])
        .map(|_| ())
    }
    fn set_user_role(&self, _jid: &JID, _role: &MemberRole) -> Result<(), Error> {
        unimplemented!()
    }

    fn test_user_password(&self, _jid: &JID, _password: &str) -> Result<bool, Error> {
        unimplemented!()
    }

    fn get_vcard(&self, jid: &JID) -> Result<Option<Vcard>, Error> {
        self.run_prosodyctl(vec![
            "mod_vcard_command",
            "get",
            jid.to_string().as_str(),
        ])
        .and_then(|output| {
            let vcard_str = str::from_utf8(&output.stdout)?;
            let vcards = vcard_parser::parse_vcards(vcard_str)?;
            Ok(vcards.into_iter().next())
        })
    }
    fn set_vcard(&self, jid: &JID, vcard: &Vcard) -> Result<(), Error> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(vcard.export().as_bytes())?;
        self.run_prosodyctl(vec![
            "mod_vcard_command",
            "set",
            jid.to_string().as_str(),
            temp_file.path().to_str().unwrap(),
        ])
        .map(|_| ())
    }
}
