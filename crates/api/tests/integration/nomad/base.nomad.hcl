variable "dns_zone_file" {
  type = string
}

job "api-test" {
  type = "batch"

  group "test" {
    count = 1

    task "coredns" {
      driver = "docker"

      template {
        data = file("${var.dns_zone_file}")
        destination = "local/coredns/zones.db"
      }

      config {
        image = "coredns/coredns:latest"
        ports = ["dns"]
        args = [
          "-conf", "/etc/coredns/zones.db"
        ]
        volumes = [
          "local/coredns/zones.db:/etc/coredns/zones.db"
        ]
      }

      resources {
        cpu    = 100
        memory = 128
      }
    }

    task "prose-pod-server" {
      driver = "docker"

      config {
        image = "proseim/prose-pod-server:latest"

        ports = [
          "prose-pod-server-xmpp-client-ipv4",
          "prose-pod-server-xmpp-client-ipv6",
          "prose-pod-server-xmpp-server-ipv4",
          "prose-pod-server-xmpp-server-ipv6",
          "prose-pod-server-http"
        ]

        mount {
          type     = "bind"
          readonly = true

          source = "local/prosody/"
          target = "/etc/prosody/"
        }
      }

      template {
        data = file("./prosody.initial.cfg.lua")
        destination = "local/prosody/prosody.cfg.lua"
      }

      resources {
        cpu    = 250
        memory = 256
      }

      service {
        provider = "nomad"

        name = "prose-pod-server-xmpp-client"
        port = "prose-pod-server-xmpp-client-ipv6"
      }

      service {
        provider = "nomad"

        name = "prose-pod-server-xmpp-server"
        port = "prose-pod-server-xmpp-server-ipv6"
      }

      service {
        provider = "nomad"

        name = "prose-pod-server-http"
        port = "prose-pod-server-http"
      }
    }

    task "api" {
      driver = "docker"

      config {
        image = "proseim/prose-pod-api:latest"
        dns_servers = ["${NOMAD_IP_coredns}"]
      }

      resources {
        cpu    = 500
        memory = 512
      }
    }

    network {
      # mode = "bridge"

      port "dns" {
        static = 53
        to     = 53
      }

      port "prose-pod-server-xmpp-client-ipv4" {
        host_network = "public-v4"

        static = 5222
        to     = 5222
      }

      port "prose-pod-server-xmpp-client-ipv6" {
        host_network = "public-v6"

        static = 5222
        to     = 5222
      }

      port "prose-pod-server-xmpp-server-ipv4" {
        host_network = "public-v4"

        static = 5269
        to     = 5269
      }

      port "prose-pod-server-xmpp-server-ipv6" {
        host_network = "public-v6"

        static = 5269
        to     = 5269
      }

      port "prose-pod-server-http" {
        host_network = "private"

        to = 5280
      }
    }
  }
}
