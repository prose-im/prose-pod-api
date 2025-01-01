@port-reachability-checks @network-checks
Feature: Port reachability checks

  Background:
    Given the server config has been initialized
      And Valerian is an admin
      And the Prose Pod API has started

  Rule: SSE route sends CHECKING events first.

    Scenario: IPv4 + IPv6
      Given the Prose Pod is publicly accessible via an IPv4
        And the Prose Pod is publicly accessible via an IPv6
        And the XMPP server domain is test.prose.org
        And test.prose.org's port 5222 is open
        And test.prose.org's port 5269 is open
        And test.prose.org's port 443 is open
       When Valerian checks the ports reachability
       Then the response is a SSE stream
        And one SSE event is "id:TCP-c2s\nevent:port-reachability-check-result\ndata:{\"description\":\"Client-to-server port at TCP 5222\",\"status\":\"CHECKING\"}"
        And one SSE event is "id:TCP-s2s\nevent:port-reachability-check-result\ndata:{\"description\":\"Server-to-server port at TCP 5269\",\"status\":\"CHECKING\"}"
        And one SSE event is "id:TCP-HTTPS\nevent:port-reachability-check-result\ndata:{\"description\":\"HTTP server port at TCP 443\",\"status\":\"CHECKING\"}"
        And one SSE event is "id:TCP-c2s\nevent:port-reachability-check-result\ndata:{\"description\":\"Client-to-server port at TCP 5222\",\"status\":\"OPEN\"}"
        And one SSE event is "id:TCP-s2s\nevent:port-reachability-check-result\ndata:{\"description\":\"Server-to-server port at TCP 5269\",\"status\":\"OPEN\"}"
        And one SSE event is "id:TCP-HTTPS\nevent:port-reachability-check-result\ndata:{\"description\":\"HTTP server port at TCP 443\",\"status\":\"OPEN\"}"
        And one SSE event is ":End of stream\nid:end\nevent:end\ndata:"

    Scenario: Hostname
      Given the Prose Pod is publicly accessible via a hostname
        And the XMPP server domain is test.prose.org
        And test.prose.org's port 5222 is open
        And test.prose.org's port 5269 is open
        And test.prose.org's port 443 is open
       When Valerian checks the ports reachability
       Then the response is a SSE stream
        And one SSE event is "id:TCP-c2s\nevent:port-reachability-check-result\ndata:{\"description\":\"Client-to-server port at TCP 5222\",\"status\":\"CHECKING\"}"
        And one SSE event is "id:TCP-s2s\nevent:port-reachability-check-result\ndata:{\"description\":\"Server-to-server port at TCP 5269\",\"status\":\"CHECKING\"}"
        And one SSE event is "id:TCP-HTTPS\nevent:port-reachability-check-result\ndata:{\"description\":\"HTTP server port at TCP 443\",\"status\":\"CHECKING\"}"
        And one SSE event is "id:TCP-c2s\nevent:port-reachability-check-result\ndata:{\"description\":\"Client-to-server port at TCP 5222\",\"status\":\"OPEN\"}"
        And one SSE event is "id:TCP-s2s\nevent:port-reachability-check-result\ndata:{\"description\":\"Server-to-server port at TCP 5269\",\"status\":\"OPEN\"}"
        And one SSE event is "id:TCP-HTTPS\nevent:port-reachability-check-result\ndata:{\"description\":\"HTTP server port at TCP 443\",\"status\":\"OPEN\"}"
        And one SSE event is ":End of stream\nid:end\nevent:end\ndata:"

  Rule: Standard hosts are checked too

    Scenario: Standard XMPP hostnames
      Given the Prose Pod is publicly accessible via a hostname
        And the XMPP server domain is test.prose.org
        And test.prose.org's port 5222 is closed
        And _xmpp-client._tcp.test.prose.org's port 5222 is open
        And test.prose.org's port 5269 is closed
        And _xmpp-server._tcp.test.prose.org's port 5269 is open
        And test.prose.org's port 443 is open
       When Valerian checks the ports reachability
       Then the response is a SSE stream
        And one SSE event is "id:TCP-c2s\nevent:port-reachability-check-result\ndata:{\"description\":\"Client-to-server port at TCP 5222\",\"status\":\"OPEN\"}"
        And one SSE event is "id:TCP-s2s\nevent:port-reachability-check-result\ndata:{\"description\":\"Server-to-server port at TCP 5269\",\"status\":\"OPEN\"}"
        And one SSE event is "id:TCP-HTTPS\nevent:port-reachability-check-result\ndata:{\"description\":\"HTTP server port at TCP 443\",\"status\":\"OPEN\"}"
        And one SSE event is ":End of stream\nid:end\nevent:end\ndata:"
