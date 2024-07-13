// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod routes;

pub use routes::*;

use rocket::Route;

pub(super) fn routes() -> Vec<Route> {
    routes![
        get_server_config,
        reset_messaging_config,
        store_message_archive,
        set_message_archive_retention,
        reset_message_archive_retention,
        store_files,
        file_storage_encryption_scheme,
        file_retention,
    ]
}
