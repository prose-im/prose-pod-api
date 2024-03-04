// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{path::PathBuf, process::Command};

use crate::server_ctl::*;

/// Rust interface to [`prosodyctl`](https://prosody.im/doc/prosodyctl).
#[derive(Debug, Default)]
pub struct ProsodyCtl {
    config_file_path: Option<PathBuf>,
}

impl ProsodyCtl {
    pub fn new(config_file_path: Option<PathBuf>) -> Self {
        Self { config_file_path }
    }

    fn config_args(&self) -> Vec<String> {
        if let Some(config_file) = &self.config_file_path {
            vec![
                "--config".to_string(),
                config_file.display().to_string(),
            ]
        } else {
            vec![]
        }
    }

    fn run_prosodyctl<S: ToString>(&self, args: Vec<S>) -> Result<(), Error> {
        let mut args: Vec<String> = args.iter().map(ToString::to_string).collect();
        args.extend(self.config_args().into_iter());
        let output = Command::new("prosodyctl")
            .args(args)
            .output()
            .map_err(Error::IO)?;
        if output.status.success() {
            Ok(())
        } else {
            Err(Error::CommandFailed(output))
        }
    }
}

impl ServerCtlImpl for ProsodyCtl {
    fn start(&self) -> Result<(), Error> {
        self.run_prosodyctl(vec!["start".to_string()])
    }

    fn stop(&self) -> Result<(), Error> {
        self.run_prosodyctl(vec!["stop".to_string()])
    }

    fn restart(&self) -> Result<(), Error> {
        self.run_prosodyctl(vec!["restart".to_string()])
    }

    fn reload(&self) -> Result<(), Error> {
        self.run_prosodyctl(vec!["reload".to_string()])
    }

    fn status(&self) -> Result<(), Error> {
        error!("`prosodyctl status` not implemented");
        todo!("`prosodyctl status`")
    }

    fn add_user(&self) -> Result<(), Error> {
        error!("`prosodyctl adduser` not implemented");
        todo!("`prosodyctl adduser`")
    }

    fn set_user_password(&self) -> Result<(), Error> {
        error!("`prosodyctl passwd` not implemented");
        todo!("`prosodyctl passwd`")
    }

    fn remove_user(&self) -> Result<(), Error> {
        error!("`prosodyctl deluser` not implemented");
        todo!("`prosodyctl deluser`")
    }
}
