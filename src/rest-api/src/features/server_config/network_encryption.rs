// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::server_config::TlsProfile;

use crate::{server_config_reset_route, server_config_routes};

server_config_reset_route!(
    reset_network_encryption_config,
    reset_network_encryption_config_route,
);

server_config_routes!(
            key: tls_profile, type: TlsProfile,
      set:   set_tls_profile_route using   set_tls_profile,
      get:   get_tls_profile_route,
    reset: reset_tls_profile_route using reset_tls_profile,
);
