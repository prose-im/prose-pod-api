// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::server_ctl::*;

/// Rust interface to [`prosodyctl`](https://prosody.im/doc/prosodyctl).
#[derive(Debug)]
pub struct ProsodyCtl {}

impl ProsodyCtl {
    pub fn new() -> Self {
        Self {}
    }
}

impl ServerCtlImpl for ProsodyCtl {
    fn start(&self) {
        todo!("`prosodyctl start`")
    }

    fn stop(&self) {
        todo!("`prosodyctl stop`")
    }

    fn restart(&self) {
        todo!("`prosodyctl restart`")
    }

    fn reload(&self) {
        todo!("`prosodyctl reload`")
    }

    fn status(&self) {
        todo!("`prosodyctl status`")
    }

    fn add_user(&self) {
        todo!("`prosodyctl adduser`")
    }

    fn set_user_password(&self) {
        todo!("`prosodyctl passwd`")
    }

    fn remove_user(&self) {
        todo!("`prosodyctl deluser`")
    }
}
