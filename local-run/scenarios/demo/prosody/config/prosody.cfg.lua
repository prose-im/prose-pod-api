-- Prose Pod Server
-- XMPP Server Configuration
-- /!\ This file has been automatically generated by Prose Pod API.
-- /!\ Do NOT edit this file manually or your changes will be overridden during the next reload.

-- Base server configuration
pidfile = "/var/run/prosody/prosody.pid"

authentication = "internal_hashed"
storage = "internal"

log = {
  info = "*console";
  warn = "*console";
  error = "*console";
}

-- Network interfaces/ports
interfaces = { "*" }
c2s_ports = { 5222 }
http_ports = { 5280 }
http_interfaces = { "*" }
https_ports = {}
https_interfaces = {}

-- Modules
plugin_paths = { "/usr/local/lib/prosody/modules" }
modules_enabled = {
  "auto_activate_hosts";
  "roster";
  "groups_internal";
  "saslauth";
  "tls";
  "dialback";
  "disco";
  "posix";
  "smacks";
  "private";
  "vcard_legacy";
  "vcard4";
  "version";
  "uptime";
  "time";
  "ping";
  "lastactivity";
  "pep";
  "blocklist";
  "limits";
  "carbons";
  "csi";
  "server_contact_info";
  "websocket";
  "cloud_notify";
  "mam";
}
modules_disabled = { "s2s" }

-- Path to SSL key and certificate for all server domains
ssl = {
  certificate = "/etc/prosody/certs/prose.org.local.crt";
  key = "/etc/prosody/certs/prose.org.local.key";
}

-- Disable in-band registrations (done through the Prose Pod Dashboard/API)
allow_registration = false

-- Mandate highest security levels
c2s_require_encryption = true

-- Enforce safety C2S/S2S limits
c2s_stanza_size_limit = 256 * 1024

limits = {
  c2s = {
    rate = "50kb/s";
    burst = "2s";
  };
}

-- Allow reverse-proxying to WebSocket service over insecure local HTTP
consider_websocket_secure = true

-- Specify server administrator
contact_info = {
  admin = { "mailto:hostmaster@prose.org.local" };
}

-- MAM settings
archive_expires_after = "never"
default_archive_policy = true
max_archive_query_results = 100

-- Enable vCard legacy compatibility layer
upgrade_legacy_vcards = true

-- Server hosts and components
VirtualHost "prose-demo.org.local"
  admins = { "prose-pod-api@admin.prose.org.local" }

  -- Modules
  modules_enabled = {
    "rest";
    "http_oauth2";
    "admin_rest";
  }

  -- HTTP settings
  http_host = "prose-pod-server"

  -- mod_http_oauth2
  allowed_oauth2_grant_types = {
    "authorization_code";
    "refresh_token";
    "password";
  }
  oauth2_access_token_ttl = 10800
  oauth2_refresh_token_ttl = 0
  oauth2_registration_key = "hs3W8InJLICq7Sx1NfBbB1za_56HaBPBj1yJOFOB79rqVSdnGSwCpcTe-sgW--cbCK9mIIVE1ks_gnlu7VT84faHULdPdae6ppC_XvX155n-55eGF2vZi-iB4yfeLsVA0q4sc_222XpKWeplJCHK_sahx--_bzqB2kP2l-cdK9DZShYxtbjeV-EyS0pGBCU5AQSccDTu4EeEkdET03Pd4VEX-ld6qd6VU13awDgcHPS4tTPSJJE32czF37hT6QbEMnXU_kovloh2swCykueHqZP9zX4TMXlAaDtW7moHFUbnMSqrz9-WORRCtqqYqRvwhm3dJurLMekwyOL3ffwA9g"

VirtualHost "admin.prose.org.local"
  admins = { "prose-pod-api@admin.prose.org.local" }

  -- Modules
  modules_enabled = {
    "admin_rest";
    "init_admin";
  }

  -- HTTP settings
  http_host = "prose-pod-server-admin"

  -- mod_init_admin
  init_admin_jid = "prose-pod-api@admin.prose.org.local"
  init_admin_password_env_var_name = "PROSE_BOOTSTRAP__PROSE_POD_API_XMPP_PASSWORD"

Component "groups.prose-demo.org.local" "muc"
  name = "Chatrooms"

  -- Modules
  modules_enabled = { "muc_mam" }

  -- MAM settings
  max_archive_query_results = 100

  restrict_room_creation = "local"

  -- MUC settings
  muc_log_all_rooms = true
  muc_log_by_default = true
  muc_log_expires_after = "never"

Component "upload.prose.org.local" "http_file_share"
  name = "HTTP File Upload"

  -- HTTP settings
  http_file_share_size_limit = 20 * 1024 * 1024
  http_file_share_daily_quota = 250 * 1024 * 1024
  http_file_share_expires_after = -1
  http_host = "localhost"
  http_external_url = "http://localhost:5280"
