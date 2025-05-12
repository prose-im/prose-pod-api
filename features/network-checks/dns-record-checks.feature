@dns-record-checks @network-checks
Feature: DNS record checks

  Background:
    Given the server config has been initialized
      And Valerian is an admin
      And the Prose Pod API has started

  Rule: SSE route sends CHECKING events first.

    Scenario: IPv4 + IPv6
      Given the Prose Pod is publicly accessible via an IPv4
        And the Prose Pod is publicly accessible via an IPv6
        And the XMPP server domain is test.prose.org
        And prose.org's DNS zone has a A record for xmpp.test.prose.org
        And prose.org's DNS zone has a AAAA record for xmpp.test.prose.org
        And prose.org's DNS zone has a SRV record for test.prose.org redirecting port 5222 to xmpp.test.prose.org.
        And prose.org's DNS zone has a SRV record for test.prose.org redirecting port 5269 to xmpp.test.prose.org.
       When Valerian checks the DNS records configuration as "text/event-stream"
       Then the response is a SSE stream
        And one SSE event is "id:IPv4\nevent:dns-record-check-result\ndata:{\"description\":\"IPv4 record for xmpp.test.prose.org\",\"status\":\"CHECKING\"}"
        And one SSE event is "id:IPv6\nevent:dns-record-check-result\ndata:{\"description\":\"IPv6 record for xmpp.test.prose.org\",\"status\":\"CHECKING\"}"
        And one SSE event is "id:SRV-c2s\nevent:dns-record-check-result\ndata:{\"description\":\"SRV record for client-to-server connections\",\"status\":\"CHECKING\"}"
        And one SSE event is "id:SRV-s2s\nevent:dns-record-check-result\ndata:{\"description\":\"SRV record for server-to-server connections\",\"status\":\"CHECKING\"}"
        And one SSE event is "id:IPv4\nevent:dns-record-check-result\ndata:{\"description\":\"IPv4 record for xmpp.test.prose.org\",\"status\":\"VALID\"}"
        And one SSE event is "id:IPv6\nevent:dns-record-check-result\ndata:{\"description\":\"IPv6 record for xmpp.test.prose.org\",\"status\":\"VALID\"}"
        And one SSE event is "id:SRV-c2s\nevent:dns-record-check-result\ndata:{\"description\":\"SRV record for client-to-server connections\",\"status\":\"PARTIALLY_VALID\"}"
        And one SSE event is "id:SRV-s2s\nevent:dns-record-check-result\ndata:{\"description\":\"SRV record for server-to-server connections\",\"status\":\"PARTIALLY_VALID\"}"
        And one SSE event is ":End of stream\nid:end\nevent:end"

    Scenario: Hostname
      Given the Prose Pod is publicly accessible via a hostname
        And the XMPP server domain is test.prose.org
        And prose.org's DNS zone has a SRV record for test.prose.org redirecting port 5222 to cloud-provider.com.
        And prose.org's DNS zone has a SRV record for test.prose.org redirecting port 5269 to cloud-provider.com.
       When Valerian checks the DNS records configuration as "text/event-stream"
       Then the response is a SSE stream
        And one SSE event is "id:SRV-c2s\nevent:dns-record-check-result\ndata:{\"description\":\"SRV record for client-to-server connections\",\"status\":\"CHECKING\"}"
        And one SSE event is "id:SRV-s2s\nevent:dns-record-check-result\ndata:{\"description\":\"SRV record for server-to-server connections\",\"status\":\"CHECKING\"}"
        And one SSE event is "id:SRV-c2s\nevent:dns-record-check-result\ndata:{\"description\":\"SRV record for client-to-server connections\",\"status\":\"PARTIALLY_VALID\"}"
        And one SSE event is "id:SRV-s2s\nevent:dns-record-check-result\ndata:{\"description\":\"SRV record for server-to-server connections\",\"status\":\"PARTIALLY_VALID\"}"
        And one SSE event is ":End of stream\nid:end\nevent:end"
