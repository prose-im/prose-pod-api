// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{server_config::ServerConfigCreateForm, xmpp::JidDomain};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InitServerConfigRequest {
    pub domain: JidDomain,
}

// MARK: BOILERPLATE

impl Into<ServerConfigCreateForm> for InitServerConfigRequest {
    fn into(self) -> ServerConfigCreateForm {
        ServerConfigCreateForm {
            domain: self.domain.to_owned(),
        }
    }
}
