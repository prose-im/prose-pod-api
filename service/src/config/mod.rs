// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod defaults;

use std::path::PathBuf;

use entity::model::{JIDNode, JID};
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::Deserialize;
use url_serde::SerdeUrl;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub api: ConfigApi,
    pub server: ConfigServer,
    pub assets: ConfigAssets,
    pub branding: ConfigBranding,
    #[serde(default)]
    pub notify: ConfigNotify,
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
        // NOTE: `admin.prose.org.local` is hard-coded here because it's internal
        //   to the Prose Pod and cannot be changed via configuration.
        JID {
            node: self.api.admin_node.to_owned(),
            domain: "admin.prose.org.local".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigApi {
    #[serde(default = "defaults::api_log_level")]
    pub log_level: String,
    #[serde(default = "defaults::api_admin_node")]
    pub admin_node: JIDNode,
    pub admin_password: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigServer {
    pub domain: String,
    #[serde(default = "defaults::server_local_hostname")]
    pub local_hostname: String,
    #[serde(default = "defaults::server_admin_rest_api_port")]
    pub admin_rest_api_port: u16,
    #[serde(default = "defaults::server_prosody_config_file_path")]
    pub prosody_config_file_path: PathBuf,
}

impl ConfigServer {
    pub fn admin_rest_api_url(&self) -> String {
        format!(
            "http://{}:{}",
            self.local_hostname, self.admin_rest_api_port
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigAssets {
    #[serde(default = "defaults::assets_path")]
    pub path: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
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
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceInvitationChannel {
    Email,
}

impl Default for WorkspaceInvitationChannel {
    fn default() -> Self {
        defaults::notify_workspace_invitation_channel()
    }
}

impl From<entity::model::InvitationChannel> for WorkspaceInvitationChannel {
    fn from(value: entity::model::InvitationChannel) -> Self {
        match value {
            entity::model::InvitationChannel::Email => Self::Email,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ConfigNotify {
    #[serde(default = "defaults::notify_workspace_invitation_channel")]
    pub workspace_invitation_channel: WorkspaceInvitationChannel,
    #[serde(default)]
    pub email: Option<ConfigNotifyEmail>,
}

#[derive(Debug, Clone, Deserialize)]
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
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
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
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotifierDependencyMode {
    Live,
    Logging,
}

#[cfg(debug_assertions)]
impl Default for NotifierDependencyMode {
    fn default() -> Self {
        Self::Live
    }
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ConfigDependencyModes {
    #[serde(default)]
    pub uuid: UuidDependencyMode,
    #[serde(default)]
    pub notifier: NotifierDependencyMode,
}
