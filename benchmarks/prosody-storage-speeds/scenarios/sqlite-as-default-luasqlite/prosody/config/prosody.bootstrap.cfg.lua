-- Prose Pod Server bootstrap configuration
-- XMPP Server Configuration

-- Base server configuration
pidfile = "/var/run/prosody/prosody.pid"

authentication = "internal_hashed"

default_storage = "sql"
storage = { roster = "sql" }
sql = { driver = "SQLite3", database = "prosody.sqlite" }
sqlite_tune = "normal"

log = {
  debug = "*console",
}

-- Network interfaces/ports
interfaces = { "*" }
c2s_ports = { 5222 }
s2s_ports = { 5269 }
http_ports = { 5280 }
http_interfaces = { "*" }
https_ports = {}
https_interfaces = {}

-- Modules
plugin_paths = { "/usr/local/lib/prosody/modules" }
modules_enabled = {
  "auto_activate_hosts",
  "storage_sql",
}

-- Disable in-band registrations (done through the Prose Pod Dashboard/API)
allow_registration = false

-- Mandate highest security levels
c2s_require_encryption = true
s2s_require_encryption = true
s2s_secure_auth = false

-- Server hosts and components
VirtualHost "admin.prose.org.local"
  admins = { "prose-pod-api@admin.prose.org.local" }

  -- Modules
  modules_enabled = {
    "admin_rest",
    "init_admin",
  }

  -- HTTP settings
  http_host = "prose-pod-server-admin"

  -- mod_init_admin
  init_admin_jid = "prose-pod-api@admin.prose.org.local"
  init_admin_password_env_var_name = "PROSE_BOOTSTRAP__PROSE_POD_API_XMPP_PASSWORD"
