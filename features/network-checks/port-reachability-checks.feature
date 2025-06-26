@port-reachability-checks @network-checks
Feature: Port reachability checks

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
        And test.prose.org’s port 5222 is open
        And test.prose.org’s port 5269 is open
        And test.prose.org’s port 443 is open
       When Valerian checks the ports reachability
       Then the response is a SSE stream
        And one SSE with id "TCP-c2s" is
            """
            event:port-reachability-check-result
            data:{"description":"Client-to-server port at TCP 5222","status":"CHECKING"}
            """
        And one SSE with id "TCP-s2s" is
            """
            event:port-reachability-check-result
            data:{"description":"Server-to-server port at TCP 5269","status":"CHECKING"}
            """
        And one SSE with id "TCP-HTTPS" is
            """
            event:port-reachability-check-result
            data:{"description":"HTTP server port at TCP 443","status":"CHECKING"}
            """
        And one SSE with id "TCP-c2s" is
            """
            event:port-reachability-check-result
            data:{"description":"Client-to-server port at TCP 5222","status":"OPEN"}
            """
        And one SSE with id "TCP-s2s" is
            """
            event:port-reachability-check-result
            data:{"description":"Server-to-server port at TCP 5269","status":"OPEN"}
            """
        And one SSE with id "TCP-HTTPS" is
            """
            event:port-reachability-check-result
            data:{"description":"HTTP server port at TCP 443","status":"OPEN"}
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
        And test.prose.org’s port 5222 is open
        And test.prose.org’s port 5269 is open
        And test.prose.org’s port 443 is open
       When Valerian checks the ports reachability
       Then the response is a SSE stream
        And one SSE with id "TCP-c2s" is
            """
            event:port-reachability-check-result
            data:{"description":"Client-to-server port at TCP 5222","status":"CHECKING"}
            """
        And one SSE with id "TCP-s2s" is
            """
            event:port-reachability-check-result
            data:{"description":"Server-to-server port at TCP 5269","status":"CHECKING"}
            """
        And one SSE with id "TCP-HTTPS" is
            """
            event:port-reachability-check-result
            data:{"description":"HTTP server port at TCP 443","status":"CHECKING"}
            """
        And one SSE with id "TCP-c2s" is
            """
            event:port-reachability-check-result
            data:{"description":"Client-to-server port at TCP 5222","status":"OPEN"}
            """
        And one SSE with id "TCP-s2s" is
            """
            event:port-reachability-check-result
            data:{"description":"Server-to-server port at TCP 5269","status":"OPEN"}
            """
        And one SSE with id "TCP-HTTPS" is
            """
            event:port-reachability-check-result
            data:{"description":"HTTP server port at TCP 443","status":"OPEN"}
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
        And test.prose.org’s port 5222 is closed
        And _xmpp-client._tcp.test.prose.org’s port 5222 is open
        And test.prose.org’s port 5269 is closed
        And _xmpp-server._tcp.test.prose.org’s port 5269 is open
        And test.prose.org’s port 443 is open
       When Valerian checks the ports reachability
       Then the response is a SSE stream
        And one SSE with id "TCP-c2s" is
            """
            event:port-reachability-check-result
            data:{"description":"Client-to-server port at TCP 5222","status":"OPEN"}
            """
        And one SSE with id "TCP-s2s" is
            """
            event:port-reachability-check-result
            data:{"description":"Server-to-server port at TCP 5269","status":"OPEN"}
            """
        And one SSE with id "TCP-HTTPS" is
            """
            event:port-reachability-check-result
            data:{"description":"HTTP server port at TCP 443","status":"OPEN"}
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
        And test.prose.org’s port 5222 is open
        And test.prose.org’s port 443 is open
       When Valerian checks the ports reachability
       Then the response is a SSE stream
        And at least one SSE has id "TCP-c2s"
        And no SSE has id "TCP-s2s"
