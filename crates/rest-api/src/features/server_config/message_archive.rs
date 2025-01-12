// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::models::durations::{DateLike, Duration, PossiblyInfinite};

use crate::{server_config_reset_route, server_config_set_route};

server_config_reset_route!(reset_messaging_config, reset_messaging_config_route);

server_config_set_route!(
    SetMessageArchiveEnabledRequest,
    bool,
    message_archive_enabled,
    set_message_archive_enabled,
    set_message_archive_enabled_route
);

server_config_set_route!(
    SetMessageArchiveRetentionRequest,
    PossiblyInfinite<Duration<DateLike>>,
    message_archive_retention,
    set_message_archive_retention,
    set_message_archive_retention_route
);
server_config_reset_route!(
    reset_message_archive_retention,
    reset_message_archive_retention_route
);
