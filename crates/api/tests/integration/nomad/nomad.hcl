data_dir = "/opt/nomad/data"

disable_update_check = true
enable_syslog        = true

log_level = "WARN"

region     = "global"
datacenter = "paris"

addresses {
  # Also listen on Docker network so that the admin.prose.org reverse proxy \
  #   can reach Nomad
  http = "::1 172.17.0.1"

  rpc  = "::1"
  serf = "::1"
}

advertise {
  http = "::1"
  rpc  = "::1"
  serf = "::1"
}

# ports {
#   http = 4646
#   rpc  = 4647
#   serf = 4648
# }

server {
  enabled          = true
  bootstrap_expect = 1
}

client {
  enabled = true
  servers = ["::1"]

  host_network "public-v4" {
    # During integration tests, we don't want the API to be accessible to the world
    interface = "lo"
    cidr = "127.0.0.1/32"
    reserved_ports = "22"
  }

  host_network "public-v6" {
    interface = "lo"
    cidr      = "::1/128"
    reserved_ports = "22"
  }

  host_network "private" {
    interface = "lo"
    cidr      = "::1/128"
  }

  options = {
    "driver.allowlist" = "docker"
  }
}

acl {
  enabled = true
}

ui {
  enabled = true
}
