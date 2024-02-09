// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod routes;

pub use routes::*;

use rocket::Route;

pub(super) fn routes() -> Vec<Route> {
    routes![
        get_features_config,
        store_message_archive,
        message_archive_retention,
        expunge_message_archive,
        file_upload,
        file_storage_encryption_scheme,
        file_retention,
        expunge_file_storage,
    ]
}
