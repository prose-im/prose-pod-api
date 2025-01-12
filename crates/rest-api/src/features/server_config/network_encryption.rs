// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::server_config::TlsProfile;

use crate::{server_config_reset_route, server_config_set_route};

server_config_reset_route!(
    reset_network_encryption_config,
    reset_network_encryption_config_route
);

server_config_set_route!(
    SetTlsProfileRequest,
    TlsProfile,
    tls_profile,
    set_tls_profile,
    set_tls_profile_route
);
server_config_reset_route!(reset_tls_profile, reset_tls_profile_route);
