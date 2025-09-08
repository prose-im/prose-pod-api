// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use validator::Validate;

#[derive(Validate, serdev::Deserialize)]
#[serde(validate = "Validate::validate")]
pub struct SearchQuery {
    #[serde(alias = "search")]
    #[validate(length(max = 128), non_control_character)]
    pub q: String,
}
