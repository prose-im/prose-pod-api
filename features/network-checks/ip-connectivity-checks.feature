@ip-connectivity-checks @network-checks
Feature: IP connectivity checks

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
        And the Prose Pod API has started
        And federation is enabled
        And prose.org’s DNS zone has a A record for test.org.
        And prose.org’s DNS zone has a AAAA record for test.org.
       When Valerian checks the IP connectivity
       Then the response is a SSE stream
        And one SSE with id "IPv4-c2s" is
            """
            event:ip-connectivity-check-result
            data:{"description":"Client-to-server connectivity over IPv4","status":"CHECKING"}
            """
        And one SSE with id "IPv6-c2s" is
            """
            event:ip-connectivity-check-result
            data:{"description":"Client-to-server connectivity over IPv6","status":"CHECKING"}
            """
        And one SSE with id "IPv4-s2s" is
            """
            event:ip-connectivity-check-result
            data:{"description":"Server-to-server connectivity over IPv4","status":"CHECKING"}
            """
        And one SSE with id "IPv6-s2s" is
            """
            event:ip-connectivity-check-result
            data:{"description":"Server-to-server connectivity over IPv6","status":"CHECKING"}
            """
        And one SSE with id "IPv4-c2s" is
            """
            event:ip-connectivity-check-result
            data:{"description":"Client-to-server connectivity over IPv4","status":"SUCCESS"}
            """
        And one SSE with id "IPv6-c2s" is
            """
            event:ip-connectivity-check-result
            data:{"description":"Client-to-server connectivity over IPv6","status":"SUCCESS"}
            """
        And one SSE with id "IPv4-s2s" is
            """
            event:ip-connectivity-check-result
            data:{"description":"Server-to-server connectivity over IPv4","status":"SUCCESS"}
            """
        And one SSE with id "IPv6-s2s" is
            """
            event:ip-connectivity-check-result
            data:{"description":"Server-to-server connectivity over IPv6","status":"SUCCESS"}
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
        And prose.org’s DNS zone has a A record for test.org.
        And prose.org’s DNS zone has a AAAA record for test.org.
       When Valerian checks the IP connectivity
       Then the response is a SSE stream
        And one SSE with id "IPv4-c2s" is
            """
            event:ip-connectivity-check-result
            data:{"description":"Client-to-server connectivity over IPv4","status":"CHECKING"}
            """
        And one SSE with id "IPv6-c2s" is
            """
            event:ip-connectivity-check-result
            data:{"description":"Client-to-server connectivity over IPv6","status":"CHECKING"}
            """
        And one SSE with id "IPv4-s2s" is
            """
            event:ip-connectivity-check-result
            data:{"description":"Server-to-server connectivity over IPv4","status":"CHECKING"}
            """
        And one SSE with id "IPv6-s2s" is
            """
            event:ip-connectivity-check-result
            data:{"description":"Server-to-server connectivity over IPv6","status":"CHECKING"}
            """
        And one SSE with id "IPv4-c2s" is
            """
            event:ip-connectivity-check-result
            data:{"description":"Client-to-server connectivity over IPv4","status":"SUCCESS"}
            """
        And one SSE with id "IPv6-c2s" is
            """
            event:ip-connectivity-check-result
            data:{"description":"Client-to-server connectivity over IPv6","status":"SUCCESS"}
            """
        And one SSE with id "IPv4-s2s" is
            """
            event:ip-connectivity-check-result
            data:{"description":"Server-to-server connectivity over IPv4","status":"SUCCESS"}
            """
        And one SSE with id "IPv6-s2s" is
            """
            event:ip-connectivity-check-result
            data:{"description":"Server-to-server connectivity over IPv6","status":"SUCCESS"}
            """
        And one SSE with id "end" is
            """
            :End of stream
            event:end
            """

  Rule: Standard hosts are checked too

    Scenario: Standard XMPP hostnames
      Given config "server.domain" is set to "test.org"
        And config "pod.address.domain" is set to "prose.test.org"
        And Valerian is an admin
        And the Prose Pod API has started
        And federation is enabled
        And prose.org’s DNS zone has no A record for test.org.
        And prose.org’s DNS zone has no AAAA record for test.org.
        And prose.org’s DNS zone has a A record for _xmpp-client._tcp.test.org.
        And prose.org’s DNS zone has a AAAA record for _xmpp-client._tcp.test.org.
        And prose.org’s DNS zone has a A record for _xmpp-server._tcp.test.org.
        And prose.org’s DNS zone has a AAAA record for _xmpp-server._tcp.test.org.
        And prose.org’s DNS zone has a A record for _xmpp-server._tcp.groups.test.org.
        And prose.org’s DNS zone has a AAAA record for _xmpp-server._tcp.groups.test.org.
       When Valerian checks the IP connectivity
       Then the response is a SSE stream
        And one SSE with id "IPv4-c2s" is
            """
            event:ip-connectivity-check-result
            data:{"description":"Client-to-server connectivity over IPv4","status":"SUCCESS"}
            """
        And one SSE with id "IPv6-c2s" is
            """
            event:ip-connectivity-check-result
            data:{"description":"Client-to-server connectivity over IPv6","status":"SUCCESS"}
            """
        And one SSE with id "IPv4-s2s" is
            """
            event:ip-connectivity-check-result
            data:{"description":"Server-to-server connectivity over IPv4","status":"SUCCESS"}
            """
        And one SSE with id "IPv6-s2s" is
            """
            event:ip-connectivity-check-result
            data:{"description":"Server-to-server connectivity over IPv6","status":"SUCCESS"}
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
        And prose.org’s DNS zone has a A record for test.org.
        And prose.org’s DNS zone has a AAAA record for test.org.
       When Valerian checks the IP connectivity
       Then the response is a SSE stream
        And at least one SSE has id "IPv4-c2s"
        And at least one SSE has id "IPv6-c2s"
        And no SSE has id "IPv4-s2s"
        And no SSE has id "IPv6-s2s"
