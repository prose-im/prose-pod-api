@dns-record-checks @network-checks
Feature: DNS record checks

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
        And prose.org’s DNS zone has a A record for prose.test.prose.org.
        And prose.org’s DNS zone has a AAAA record for prose.test.prose.org.
        And prose.org’s DNS zone has a CNAME record redirecting admin.prose.test.prose.org. to prose.test.prose.org.
        And prose.org’s DNS zone has a SRV record for _xmpp-client._tcp.test.prose.org. redirecting port 5222 to prose.test.prose.org.
        And prose.org’s DNS zone has a SRV record for _xmpp-server._tcp.test.prose.org. redirecting port 5269 to prose.test.prose.org.
        And prose.org’s DNS zone has a SRV record for _xmpp-server._tcp.groups.test.prose.org. redirecting port 5269 to prose.test.prose.org.
       When Valerian checks the DNS records configuration as "text/event-stream"
       Then the response is a SSE stream
        And one SSE with id "IPv4" is
            """
            event:dns-record-check-result
            data:{"description":"IPv4 record for prose.test.prose.org.","status":"CHECKING"}
            """
        And one SSE with id "IPv6" is
            """
            event:dns-record-check-result
            data:{"description":"IPv6 record for prose.test.prose.org.","status":"CHECKING"}
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
            data:{"description":"IPv4 record for prose.test.prose.org.","status":"VALID"}
            """
        And one SSE with id "IPv6" is
            """
            event:dns-record-check-result
            data:{"description":"IPv6 record for prose.test.prose.org.","status":"VALID"}
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
      Given federation is enabled
        And the XMPP server domain is test.prose.org
        And the Prose Pod is publicly accessible via prose.test.prose.org
        And prose.org’s DNS zone has a CNAME record redirecting admin.prose.test.prose.org. to prose.test.prose.org.
        And prose.org’s DNS zone has a SRV record for _xmpp-client._tcp.test.prose.org. redirecting port 5222 to cloud-provider.com.
        And prose.org’s DNS zone has a SRV record for _xmpp-server._tcp.test.prose.org. redirecting port 5269 to cloud-provider.com.
        And prose.org’s DNS zone has a SRV record for _xmpp-server._tcp.groups.test.prose.org. redirecting port 5269 to cloud-provider.com.
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
      Given federation is disabled
        And the XMPP server domain is test.prose.org
        And the Prose Pod is publicly accessible via prose.test.prose.org
        And prose.org’s DNS zone has a CNAME record redirecting admin.prose.test.prose.org. to prose.test.prose.org.
        And prose.org’s DNS zone has a SRV record for _xmpp-client._tcp.test.prose.org. redirecting port 5222 to cloud-provider.com.
       When Valerian checks the DNS records configuration as "text/event-stream"
       Then the response is a SSE stream
        And at least one SSE has id "SRV-c2s"
        And no SSE has id "SRV-s2s"
