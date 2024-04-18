// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod defaults;

use std::path::PathBuf;

use entity::model::JID;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::Deserialize;
use url_serde::SerdeUrl;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub api: ConfigApi,
    pub server: ConfigServer,
    pub assets: ConfigAssets,
    pub branding: ConfigBranding,
    pub notify: Option<ConfigNotify>,
    #[cfg(debug_assertions)]
    #[serde(default)]
    pub dependency_modes: ConfigDependencyModes,
}

impl Config {
    pub fn figment() -> Self {
        // NOTE: See what's possible at <https://docs.rs/figment/latest/figment/>.
        Figment::new()
            .merge(Toml::file("Prose.toml"))
            .merge(Env::prefixed("PROSE_").split("__"))
            .extract()
            .expect("Could not read config")
    }

    pub fn api_jid(&self) -> JID {
        JID::new(self.api.admin_node.clone(), self.server.domain.clone())
    }
}

#[derive(Clone, Deserialize)]
pub struct ConfigApi {
    #[serde(default = "defaults::api_log_level")]
    pub log_level: String,
    #[serde(default = "defaults::api_admin_node")]
    pub admin_node: String,
    pub admin_password: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct ConfigServer {
    pub domain: String,
    #[serde(default = "defaults::server_local_hostname")]
    pub local_hostname: String,
    #[serde(default = "defaults::server_admin_rest_api_port")]
    pub admin_rest_api_port: u16,
}

#[derive(Clone, Deserialize)]
pub struct ConfigAssets {
    #[serde(default = "defaults::assets_path")]
    pub path: PathBuf,
}

#[derive(Clone, Deserialize)]
pub struct ConfigBranding {
    #[serde(default = "defaults::branding_page_title")]
    pub page_title: String,

    pub page_url: SerdeUrl,
    pub company_name: String,
    pub icon_color: String,
    pub icon_url: SerdeUrl,
    pub logo_color: String,
    pub logo_url: SerdeUrl,
    pub website_url: SerdeUrl,
    pub support_url: SerdeUrl,
    pub custom_html: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct ConfigNotify {
    pub email: Option<ConfigNotifyEmail>,
}

#[derive(Clone, Deserialize)]
pub struct ConfigNotifyEmail {
    pub to: String,
    pub from: String,

    #[serde(default = "defaults::notify_email_smtp_host")]
    pub smtp_host: String,

    #[serde(default = "defaults::notify_email_smtp_port")]
    pub smtp_port: u16,

    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,

    #[serde(default = "defaults::notify_email_smtp_encrypt")]
    pub smtp_encrypt: bool,
}

#[cfg(debug_assertions)]
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UuidDependencyMode {
    Normal,
    Incrementing,
}

#[cfg(debug_assertions)]
impl Default for UuidDependencyMode {
    fn default() -> Self {
        Self::Normal
    }
}

#[cfg(debug_assertions)]
#[derive(Clone, Deserialize, Default)]
pub struct ConfigDependencyModes {
    #[serde(default)]
    pub uuid: UuidDependencyMode,
}
