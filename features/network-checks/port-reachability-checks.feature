@port-reachability-checks @network-checks
Feature: Port reachability checks

  Background:
    Given the XMPP server has been initialized
      And Valerian is an admin
        # NOTE: Required when `pod.address.domain` is unset
      And config "dashboard.url" is set to "https://dashboard.prose.test.org"

  Rule: SSE route sends CHECKING events first.

    Scenario: IPv4 + IPv6
      Given config "server.domain" is set to "test.org"
        And config "pod.address.ipv4" is set to "0.0.0.0"
        And config "pod.address.ipv6" is set to "::"
        And config "pod.address.domain" is unset
        And the Prose Pod API has started
        And federation is enabled
        And test.org’s port 5222 is open
        And test.org’s port 5269 is open
        And test.org’s port 443 is open
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
      Given config "server.domain" is set to "test.org"
        And config "pod.address.domain" is set to "prose.test.org"
        And the Prose Pod API has started
        And federation is enabled
        And test.org’s port 5222 is open
        And test.org’s port 5269 is open
        And test.org’s port 443 is open
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
      Given config "server.domain" is set to "test.org"
        And config "pod.address.domain" is set to "prose.test.org"
        And the Prose Pod API has started
        And federation is enabled
        And test.org’s port 5222 is closed
        And _xmpp-client._tcp.test.org’s port 5222 is open
        And test.org’s port 5269 is closed
        And _xmpp-server._tcp.test.org’s port 5269 is open
        And test.org’s port 443 is open
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
      Given config "server.domain" is set to "test.org"
        And config "pod.address.domain" is set to "prose.test.org"
        And the Prose Pod API has started
        And federation is disabled
        And test.org’s port 5222 is open
        And test.org’s port 443 is open
       When Valerian checks the ports reachability
       Then the response is a SSE stream
        And at least one SSE has id "TCP-c2s"
        And no SSE has id "TCP-s2s"
