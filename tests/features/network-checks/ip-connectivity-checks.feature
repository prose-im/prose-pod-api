@ip-connectivity-checks @network-checks
@testing
Feature: IP connectivity checks

  Background:
    Given the server config has been initialized
      And Valerian is an admin
      And the Prose Pod API has started

  Rule: SSE route sends CHECKING events first.

    Scenario: IPv4 + IPv6
      Given the Prose Pod is publicly accessible via an IPv4
        And the Prose Pod is publicly accessible via an IPv6
        And the XMPP server domain is test.prose.org
        And prose.org's DNS zone has a A record for test.prose.org
        And prose.org's DNS zone has a AAAA record for test.prose.org
       When Valerian checks the IP connectivity
       Then the response is a SSE stream
        And one SSE event is "id:IPv4-c2s\nevent:ip-connectivity-check-result\ndata:{\"description\":\"Client-to-server connectivity over IPv4\",\"status\":\"CHECKING\"}"
        And one SSE event is "id:IPv6-c2s\nevent:ip-connectivity-check-result\ndata:{\"description\":\"Client-to-server connectivity over IPv6\",\"status\":\"CHECKING\"}"
        And one SSE event is "id:IPv4-s2s\nevent:ip-connectivity-check-result\ndata:{\"description\":\"Server-to-server connectivity over IPv4\",\"status\":\"CHECKING\"}"
        And one SSE event is "id:IPv6-s2s\nevent:ip-connectivity-check-result\ndata:{\"description\":\"Server-to-server connectivity over IPv6\",\"status\":\"CHECKING\"}"
        And one SSE event is "id:IPv4-c2s\nevent:ip-connectivity-check-result\ndata:{\"description\":\"Client-to-server connectivity over IPv4\",\"status\":\"SUCCESS\"}"
        And one SSE event is "id:IPv6-c2s\nevent:ip-connectivity-check-result\ndata:{\"description\":\"Client-to-server connectivity over IPv6\",\"status\":\"SUCCESS\"}"
        And one SSE event is "id:IPv4-s2s\nevent:ip-connectivity-check-result\ndata:{\"description\":\"Server-to-server connectivity over IPv4\",\"status\":\"SUCCESS\"}"
        And one SSE event is "id:IPv6-s2s\nevent:ip-connectivity-check-result\ndata:{\"description\":\"Server-to-server connectivity over IPv6\",\"status\":\"SUCCESS\"}"
        And one SSE event is ":End of stream\nid:end\nevent:end\ndata:"

    Scenario: Hostname
      Given the Prose Pod is publicly accessible via a hostname
        And the XMPP server domain is test.prose.org
        And prose.org's DNS zone has a A record for test.prose.org
        And prose.org's DNS zone has a AAAA record for test.prose.org
       When Valerian checks the IP connectivity
       Then the response is a SSE stream
        And one SSE event is "id:IPv4-c2s\nevent:ip-connectivity-check-result\ndata:{\"description\":\"Client-to-server connectivity over IPv4\",\"status\":\"CHECKING\"}"
        And one SSE event is "id:IPv6-c2s\nevent:ip-connectivity-check-result\ndata:{\"description\":\"Client-to-server connectivity over IPv6\",\"status\":\"CHECKING\"}"
        And one SSE event is "id:IPv4-s2s\nevent:ip-connectivity-check-result\ndata:{\"description\":\"Server-to-server connectivity over IPv4\",\"status\":\"CHECKING\"}"
        And one SSE event is "id:IPv6-s2s\nevent:ip-connectivity-check-result\ndata:{\"description\":\"Server-to-server connectivity over IPv6\",\"status\":\"CHECKING\"}"
        And one SSE event is "id:IPv4-c2s\nevent:ip-connectivity-check-result\ndata:{\"description\":\"Client-to-server connectivity over IPv4\",\"status\":\"SUCCESS\"}"
        And one SSE event is "id:IPv6-c2s\nevent:ip-connectivity-check-result\ndata:{\"description\":\"Client-to-server connectivity over IPv6\",\"status\":\"SUCCESS\"}"
        And one SSE event is "id:IPv4-s2s\nevent:ip-connectivity-check-result\ndata:{\"description\":\"Server-to-server connectivity over IPv4\",\"status\":\"SUCCESS\"}"
        And one SSE event is "id:IPv6-s2s\nevent:ip-connectivity-check-result\ndata:{\"description\":\"Server-to-server connectivity over IPv6\",\"status\":\"SUCCESS\"}"
        And one SSE event is ":End of stream\nid:end\nevent:end\ndata:"

  Rule: Standard hosts are checked too

    Scenario: Standard XMPP hostnames
      Given the Prose Pod is publicly accessible via a hostname
        And the XMPP server domain is test.prose.org
        And prose.org's DNS zone has no A record for test.prose.org
        And prose.org's DNS zone has no AAAA record for test.prose.org
        And prose.org's DNS zone has a A record for _xmpp-client._tcp.test.prose.org.
        And prose.org's DNS zone has a AAAA record for _xmpp-client._tcp.test.prose.org.
        And prose.org's DNS zone has a A record for _xmpp-server._tcp.test.prose.org.
        And prose.org's DNS zone has a AAAA record for _xmpp-server._tcp.test.prose.org.
       When Valerian checks the IP connectivity
       Then the response is a SSE stream
        And one SSE event is "id:IPv4-c2s\nevent:ip-connectivity-check-result\ndata:{\"description\":\"Client-to-server connectivity over IPv4\",\"status\":\"SUCCESS\"}"
        And one SSE event is "id:IPv6-c2s\nevent:ip-connectivity-check-result\ndata:{\"description\":\"Client-to-server connectivity over IPv6\",\"status\":\"SUCCESS\"}"
        And one SSE event is "id:IPv4-s2s\nevent:ip-connectivity-check-result\ndata:{\"description\":\"Server-to-server connectivity over IPv4\",\"status\":\"SUCCESS\"}"
        And one SSE event is "id:IPv6-s2s\nevent:ip-connectivity-check-result\ndata:{\"description\":\"Server-to-server connectivity over IPv6\",\"status\":\"SUCCESS\"}"
        And one SSE event is ":End of stream\nid:end\nevent:end\ndata:"

  # MISSING status = internal field to say if check should be ran + check.run(network_checker) instead of logic in routes.rs
