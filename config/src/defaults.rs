// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{path::PathBuf, str::FromStr as _};

use crate::model::JidNode;

use super::WorkspaceInvitationChannel;

pub fn api_admin_node() -> JidNode {
    JidNode::from_str("prose-pod-api").expect("Invalid default `api_admin_node`")
}

pub fn server_local_hostname() -> String {
    "prose-pod-server".to_string()
}

pub fn server_local_hostname_admin() -> String {
    "prose-pod-server-admin".to_string()
}

pub fn server_http_port() -> u16 {
    5280
}

pub fn server_prosody_config_file_path() -> PathBuf {
    PathBuf::from("/etc/prosody/prosody.cfg.lua")
}

pub fn branding_page_title() -> String {
    "Prose Pod API".to_string()
}

pub fn notify_workspace_invitation_channel() -> WorkspaceInvitationChannel {
    WorkspaceInvitationChannel::Email
}

pub fn notify_email_smtp_host() -> String {
    "localhost".to_string()
}

pub fn notify_email_smtp_port() -> u16 {
    587
}

pub fn notify_email_smtp_encrypt() -> bool {
    true
}
