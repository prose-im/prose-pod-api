// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::LinkedHashSet;

use crate::{server_config_reset_route, server_config_routes};

server_config_reset_route!(
    reset_server_federation_config,
    reset_server_federation_config_route
);

server_config_routes!(
            key: federation_enabled, type: bool,
      set:   set_federation_enabled_route using   set_federation_enabled,
      get:   get_federation_enabled_route,
    reset: reset_federation_enabled_route using reset_federation_enabled,
);
server_config_routes!(
            key: federation_whitelist_enabled, type: bool,
      set:   set_federation_whitelist_enabled_route using   set_federation_whitelist_enabled,
      get:   get_federation_whitelist_enabled_route,
    reset: reset_federation_whitelist_enabled_route using reset_federation_whitelist_enabled,
);
server_config_routes!(
            key: federation_friendly_servers, type: LinkedHashSet<String>,
      set:   set_federation_friendly_servers_route using   set_federation_friendly_servers,
      get:   get_federation_friendly_servers_route,
    reset: reset_federation_friendly_servers_route using reset_federation_friendly_servers,
);
