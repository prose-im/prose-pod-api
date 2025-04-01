// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{server_config_reset_route, server_config_routes};

server_config_reset_route!(
    reset_push_notifications_config,
    reset_push_notifications_config_route
);

server_config_routes!(
            key: push_notification_with_body, type: bool,
      set:   set_push_notification_with_body_route using   set_push_notification_with_body,
      get:   get_push_notification_with_body_route,
    reset: reset_push_notification_with_body_route using reset_push_notification_with_body,
);
server_config_routes!(
            key: push_notification_with_sender, type: bool,
      set:   set_push_notification_with_sender_route using   set_push_notification_with_sender,
      get:   get_push_notification_with_sender_route,
    reset: reset_push_notification_with_sender_route using reset_push_notification_with_sender,
);
