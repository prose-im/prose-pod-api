// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::serde::json::Json;
use service::{
    model::{DateLike, Duration, PossiblyInfinite, ServerConfig},
    services::server_manager::ServerManager,
};

use crate::{guards::LazyGuard, server_config_reset_route, server_config_set_route};

server_config_reset_route!(
    "/v1/server/config/messaging/reset",
    reset_messaging_config,
    reset_messaging_config_route
);

server_config_set_route!(
    "/v1/server/config/message-archive-enabled",
    SetMessageArchiveEnabledRequest,
    bool,
    message_archive_enabled,
    set_message_archive_enabled,
    set_message_archive_enabled_route
);

server_config_set_route!(
    "/v1/server/config/message-archive-retention",
    SetMessageArchiveRetentionRequest,
    PossiblyInfinite<Duration<DateLike>>,
    message_archive_retention,
    set_message_archive_retention,
    set_message_archive_retention_route
);
server_config_reset_route!(
    "/v1/server/config/message-archive-retention/reset",
    reset_message_archive_retention,
    reset_message_archive_retention_route
);
