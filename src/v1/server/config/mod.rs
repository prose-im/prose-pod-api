// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod routes;

pub use routes::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![
        get_server_config,
        reset_messaging_config,
        set_message_archive_enabled,
        set_message_archive_retention,
        reset_message_archive_retention,
        set_file_upload_allowed,
        set_file_storage_encryption_scheme,
        set_file_storage_retention,
    ]
}
