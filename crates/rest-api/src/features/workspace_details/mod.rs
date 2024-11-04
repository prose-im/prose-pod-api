// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod get_workspace;
mod workspace_accent_color;
mod workspace_icon;
mod workspace_name;
mod workspace_vcard;

pub use get_workspace::*;
pub use workspace_accent_color::*;
pub use workspace_icon::*;
pub use workspace_name::*;
pub use workspace_vcard::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![
        get_workspace_route,
        get_workspace_accent_color_route,
        set_workspace_accent_color_route,
        get_workspace_icon_route,
        set_workspace_icon_route,
        get_workspace_name_route,
        set_workspace_name_route,
        get_workspace_details_card_route,
        set_workspace_details_card_route,
    ]
}
