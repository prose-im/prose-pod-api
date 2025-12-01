@dns-record-checks @network-checks
Feature: DNS record checks

  Background:
    Given the XMPP server has been initialized
        # NOTE: Required when `pod.address.domain` is unset
      And config "dashboard.url" is set to "https://dashboard.prose.test.org"

  Rule: SSE route sends CHECKING events first.

    Scenario: IPv4 + IPv6
      Given config "server.domain" is set to "test.org"
        And config "pod.address.ipv4" is set to "0.0.0.0"
        And config "pod.address.ipv6" is set to "::"
        And config "pod.address.domain" is unset
        And Valerian is an admin
        And federation is enabled
        And the Prose Pod API has started
        And prose.org’s DNS zone has a A record for prose.test.org.
        And prose.org’s DNS zone has a AAAA record for prose.test.org.
        And prose.org’s DNS zone has a CNAME record redirecting admin.prose.test.org. to prose.test.org.
        And prose.org’s DNS zone has a SRV record for _xmpp-client._tcp.test.org. redirecting port 5222 to prose.test.org.
        And prose.org’s DNS zone has a SRV record for _xmpp-server._tcp.test.org. redirecting port 5269 to prose.test.org.
        And prose.org’s DNS zone has a SRV record for _xmpp-server._tcp.groups.test.org. redirecting port 5269 to prose.test.org.
       When Valerian checks the DNS records configuration as "text/event-stream"
       Then the response is a SSE stream
        And one SSE with id "IPv4" is
            """
            event:dns-record-check-result
            data:{"description":"IPv4 record for prose.test.org.","status":"CHECKING"}
            """
        And one SSE with id "IPv6" is
            """
            event:dns-record-check-result
            data:{"description":"IPv6 record for prose.test.org.","status":"CHECKING"}
            """
        And one SSE with id "SRV-c2s" is
            """
            event:dns-record-check-result
            data:{"description":"SRV record for client-to-server connections","status":"CHECKING"}
            """
        And one SSE with id "SRV-s2s" is
            """
            event:dns-record-check-result
            data:{"description":"SRV record for server-to-server connections","status":"CHECKING"}
            """
        And one SSE with id "IPv4" is
            """
            event:dns-record-check-result
            data:{"description":"IPv4 record for prose.test.org.","status":"VALID"}
            """
        And one SSE with id "IPv6" is
            """
            event:dns-record-check-result
            data:{"description":"IPv6 record for prose.test.org.","status":"VALID"}
            """
        And one SSE with id "SRV-c2s" is
            """
            event:dns-record-check-result
            data:{"description":"SRV record for client-to-server connections","status":"PARTIALLY_VALID"}
            """
        And one SSE with id "SRV-s2s" is
            """
            event:dns-record-check-result
            data:{"description":"SRV record for server-to-server connections","status":"PARTIALLY_VALID"}
            """
        And one SSE with id "end" is
            """
            :End of stream
            event:end
            """

    Scenario: Hostname
      Given config "server.domain" is set to "test.org"
        And config "pod.address.domain" is set to "prose.test.org"
        And Valerian is an admin
        And the Prose Pod API has started
        And federation is enabled
        And prose.org’s DNS zone has a CNAME record redirecting admin.prose.test.org. to prose.test.org.
        And prose.org’s DNS zone has a SRV record for _xmpp-client._tcp.test.org. redirecting port 5222 to cloud-provider.com.
        And prose.org’s DNS zone has a SRV record for _xmpp-server._tcp.test.org. redirecting port 5269 to cloud-provider.com.
        And prose.org’s DNS zone has a SRV record for _xmpp-server._tcp.groups.test.org. redirecting port 5269 to cloud-provider.com.
       When Valerian checks the DNS records configuration as "text/event-stream"
       Then the response is a SSE stream
        And one SSE with id "SRV-c2s" is
            """
            event:dns-record-check-result
            data:{"description":"SRV record for client-to-server connections","status":"CHECKING"}
            """
        And one SSE with id "SRV-s2s" is
            """
            event:dns-record-check-result
            data:{"description":"SRV record for server-to-server connections","status":"CHECKING"}
            """
        And one SSE with id "SRV-c2s" is
            """
            event:dns-record-check-result
            data:{"description":"SRV record for client-to-server connections","status":"PARTIALLY_VALID"}
            """
        And one SSE with id "SRV-s2s" is
            """
            event:dns-record-check-result
            data:{"description":"SRV record for server-to-server connections","status":"PARTIALLY_VALID"}
            """
        And one SSE with id "end" is
            """
            :End of stream
            event:end
            """

  Rule: Server-to-server checks are ran only if federation is enabled

    Scenario: Hostname
      Given config "server.domain" is set to "test.org"
        And config "pod.address.domain" is set to "prose.test.org"
        And Valerian is an admin
        And the Prose Pod API has started
        And federation is disabled
        And prose.org’s DNS zone has a CNAME record redirecting admin.prose.test.org. to prose.test.org.
        And prose.org’s DNS zone has a SRV record for _xmpp-client._tcp.test.org. redirecting port 5222 to cloud-provider.com.
       When Valerian checks the DNS records configuration as "text/event-stream"
       Then the response is a SSE stream
        And at least one SSE has id "SRV-c2s"
        And no SSE has id "SRV-s2s"
