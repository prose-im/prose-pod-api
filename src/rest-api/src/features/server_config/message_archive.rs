// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::models::durations::{DateLike, Duration, PossiblyInfinite};

use crate::{server_config_reset_route, server_config_routes};

server_config_reset_route!(reset_messaging_config, reset_messaging_config_route);

server_config_routes!(
            key: message_archive_enabled, type: bool,
      set:   set_message_archive_enabled_route using   set_message_archive_enabled,
      get:   get_message_archive_enabled_route,
    reset: reset_message_archive_enabled_route using reset_message_archive_enabled,
);
server_config_routes!(
            key: message_archive_retention, type: PossiblyInfinite<Duration<DateLike>>,
      set:   set_message_archive_retention_route using   set_message_archive_retention,
      get:   get_message_archive_retention_route,
    reset: reset_message_archive_retention_route using reset_message_archive_retention,
);
