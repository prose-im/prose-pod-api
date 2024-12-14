// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::serde::json::Json;
use service::{server_config::ServerConfig, xmpp::ServerManager};

use crate::{guards::LazyGuard, server_config_reset_route, server_config_set_route};

server_config_reset_route!(
    "/v1/server/config/push-notifications/reset",
    reset_push_notifications_config,
    reset_push_notifications_config_route,
    reset_push_notifications_config_route_axum
);

server_config_set_route!(
    "/v1/server/config/push-notification-with-body",
    SetPushNotificationWithBodyRequest,
    bool,
    push_notification_with_body,
    set_push_notification_with_body,
    set_push_notification_with_body_route,
    set_push_notification_with_body_route_axum
);
server_config_reset_route!(
    "/v1/server/config/push-notification-with-body/reset",
    reset_push_notification_with_body,
    reset_push_notification_with_body_route,
    reset_push_notification_with_body_route_axum
);

server_config_set_route!(
    "/v1/server/config/push-notification-with-sender",
    SetPushNotificationWithSenderRequest,
    bool,
    push_notification_with_sender,
    set_push_notification_with_sender,
    set_push_notification_with_sender_route,
    set_push_notification_with_sender_route_axum
);
server_config_reset_route!(
    "/v1/server/config/push-notification-with-sender/reset",
    reset_push_notification_with_sender,
    reset_push_notification_with_sender_route,
    reset_push_notification_with_sender_route_axum
);
