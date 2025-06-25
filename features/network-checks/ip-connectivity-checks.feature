@ip-connectivity-checks @network-checks
Feature: IP connectivity checks

  Background:
    Given the XMPP server has been initialized
      And Valerian is an admin
      And the Prose Pod API has started

  Rule: SSE route sends CHECKING events first.

    Scenario: IPv4 + IPv6
      Given the Prose Pod is publicly accessible via an IPv4
        And the Prose Pod is publicly accessible via an IPv6
        And the Prose Pod isn’t publicly accessible via a domain
        And federation is enabled
        And the XMPP server domain is test.prose.org
        And prose.org’s DNS zone has a A record for test.prose.org.
        And prose.org’s DNS zone has a AAAA record for test.prose.org.
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
      Given the Prose Pod is publicly accessible via a domain
        And federation is enabled
        And the XMPP server domain is test.prose.org
        And prose.org’s DNS zone has a A record for test.prose.org.
        And prose.org’s DNS zone has a AAAA record for test.prose.org.
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
      Given the Prose Pod is publicly accessible via a domain
        And federation is enabled
        And the XMPP server domain is test.prose.org
        And prose.org’s DNS zone has no A record for test.prose.org.
        And prose.org’s DNS zone has no AAAA record for test.prose.org.
        And prose.org’s DNS zone has a A record for _xmpp-client._tcp.test.prose.org.
        And prose.org’s DNS zone has a AAAA record for _xmpp-client._tcp.test.prose.org.
        And prose.org’s DNS zone has a A record for _xmpp-server._tcp.test.prose.org.
        And prose.org’s DNS zone has a AAAA record for _xmpp-server._tcp.test.prose.org.
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
      Given the Prose Pod is publicly accessible via a domain
        And federation is disabled
        And the XMPP server domain is test.prose.org
        And prose.org’s DNS zone has a A record for test.prose.org.
        And prose.org’s DNS zone has a AAAA record for test.prose.org.
       When Valerian checks the IP connectivity
       Then the response is a SSE stream
        And at least one SSE has id "IPv4-c2s"
        And at least one SSE has id "IPv6-c2s"
        And no SSE has id "IPv4-s2s"
        And no SSE has id "IPv6-s2s"
