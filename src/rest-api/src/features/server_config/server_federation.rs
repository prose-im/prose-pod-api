// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::LinkedHashSet;

use crate::{server_config_reset_route, server_config_set_route};

server_config_reset_route!(
    reset_server_federation_config,
    reset_server_federation_config_route
);

server_config_reset_route!(reset_federation_enabled, reset_federation_enabled_route);

server_config_set_route!(
    bool,
    federation_enabled,
    set_federation_enabled,
    set_federation_enabled_route
);

server_config_reset_route!(
    reset_federation_whitelist_enabled,
    reset_federation_whitelist_enabled_route
);

server_config_set_route!(
    bool,
    federation_whitelist_enabled,
    set_federation_whitelist_enabled,
    set_federation_whitelist_enabled_route
);

server_config_reset_route!(
    reset_federation_friendly_servers,
    reset_federation_friendly_servers_route
);

server_config_set_route!(
    LinkedHashSet<String>,
    federation_friendly_servers,
    set_federation_friendly_servers,
    set_federation_friendly_servers_route
);
