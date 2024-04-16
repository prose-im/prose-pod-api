// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::path::PathBuf;

pub fn api_log_level() -> String {
    "error".to_string()
}

pub fn api_admin_node() -> String {
    "prose-pod-api".to_string()
}

pub fn server_local_hostname() -> String {
    "prose-pod-server".to_string()
}

pub fn server_admin_rest_api_port() -> u16 {
    5280
}

pub fn assets_path() -> PathBuf {
    PathBuf::from("./res/assets/")
}

pub fn branding_page_title() -> String {
    "Prose Pod API".to_string()
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
