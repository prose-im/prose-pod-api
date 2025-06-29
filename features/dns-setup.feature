@dns-setup
Feature: DNS setup instructions

  Background:
    Given the XMPP server has been initialized
      And Valerian is an admin
      And the Prose Pod API has started

  Rule: Only admins can see DNS setup instructions

    Scenario: Rémi (not admin) tries to get DNS setup instructions
      Given Rémi is not an admin
        And the Prose Pod is publicly accessible via an IPv4
       When Rémi requests DNS setup instructions
       Then the HTTP status code should be Forbidden
        And the response content type should be JSON
        And the error code should be "forbidden"

  """
  `SRV` records cannot point directly to IP addresses, we need to point it to a hostname.
  """
  Rule: If the Prose Pod has a static IP address, `SRV` records point to `prose.<domain>` and `prose.<domain>` points to the Prose Pod

    Scenario Outline: IPv4 only
      Given the Prose Pod is publicly accessible via an IPv4
        And the Prose Pod isn’t publicly accessible via a domain
        And the XMPP server domain is <domain>
        And federation is enabled
       When Valerian requests DNS setup instructions
       Then the call should succeed
        And DNS setup instructions should contain 4 steps
        And step 1 should contain a single A record
        And step 2 should contain a single SRV record
        And step 3 should contain a single CNAME record
        And step 4 should contain SRV and SRV records
        And A records hostnames should be prose.<domain>
        And SRV records targets should be prose.<domain>.

    Examples:
      | domain         |
      | prose.org      |
      | chat.prose.org |

    Scenario Outline: IPv6 only
      Given the Prose Pod is publicly accessible via an IPv6
        And the Prose Pod isn’t publicly accessible via a domain
        And the XMPP server domain is <domain>
        And federation is enabled
       When Valerian requests DNS setup instructions
       Then the call should succeed
        And DNS setup instructions should contain 4 steps
        And step 1 should contain a single AAAA record
        And step 2 should contain a single SRV record
        And step 3 should contain a single CNAME record
        And step 4 should contain SRV and SRV records
        And AAAA records hostnames should be prose.<domain>
        And SRV records targets should be prose.<domain>.

    Examples:
      | domain         |
      | prose.org      |
      | chat.prose.org |

    Scenario Outline: IPv4 + IPv6
      Given the Prose Pod is publicly accessible via an IPv4
        And the Prose Pod is publicly accessible via an IPv6
        And the Prose Pod isn’t publicly accessible via a domain
        And the XMPP server domain is <domain>
        And federation is enabled
       When Valerian requests DNS setup instructions
       Then the call should succeed
        And DNS setup instructions should contain 4 steps
        And step 1 should contain A and AAAA records
        And step 2 should contain a single SRV record
        And step 3 should contain a single CNAME record
        And step 4 should contain SRV and SRV records
        And A records hostnames should be prose.<domain>
        And AAAA records hostnames should be prose.<domain>
        And SRV records targets should be prose.<domain>.

    Examples:
      | domain         |
      | prose.org      |
      | chat.prose.org |

  Rule: If the Prose Pod is publicly accessible via a domain, `SRV` records point to it

    Scenario: Hostname
      Given the Prose Pod is publicly accessible via a domain
        And federation is enabled
       When Valerian requests DNS setup instructions
       Then the call should succeed
        And DNS setup instructions should contain 3 steps
        And step 1 should contain a single SRV record
        And step 2 should contain a single CNAME record
        And step 3 should contain SRV and SRV records

    """
    This scenario should not happen but it's possible because of the database schema.
    """
    Scenario: IPv4 + IPv6 + hostname
      Given the Prose Pod is publicly accessible via an IPv4
        And the Prose Pod is publicly accessible via an IPv6
        And the Prose Pod is publicly accessible via a domain
        And federation is enabled
       When Valerian requests DNS setup instructions
       Then the call should succeed
        And DNS setup instructions should contain 3 steps
        And step 1 should contain a single SRV record
        And step 2 should contain a single CNAME record
        And step 3 should contain SRV and SRV records

  Rule: DNS setup instructions give SRV records for ports 5222 and 5269

    Scenario: Prose Pod accessible via an IP address
      Given the Prose Pod is publicly accessible via an IPv4
        And federation is enabled
       When Valerian requests DNS setup instructions
       Then the call should succeed
        And DNS setup instructions should contain a SRV record for port 5222
        And DNS setup instructions should contain a SRV record for port 5269

    Scenario: Prose Pod accessible via a domain
      Given the Prose Pod is publicly accessible via a domain
        And federation is enabled
       When Valerian requests DNS setup instructions
       Then the call should succeed
        And DNS setup instructions should contain a SRV record for port 5222
        And DNS setup instructions should contain a SRV record for port 5269

  Rule: DNS setup instructions use the XMPP server's domain

    Scenario Outline: Prose Pod accessible via an IP address
      Given the XMPP server domain is <domain>
        And the Prose Pod is publicly accessible via an IPv4
       When Valerian requests DNS setup instructions
       Then the call should succeed
        And SRV record hostname should be _xmpp-client._tcp.<domain> for port 5222
        And SRV record hostname should be _xmpp-server._tcp.<domain> for port 5269
        And SRV record hostname should be _xmpp-server._tcp.groups.<domain> for port 5269

    Examples:
      | domain         |
      | prose.org      |
      | chat.prose.org |

    Scenario Outline: Prose Pod accessible via a domain
      Given the XMPP server domain is <domain>
        And the Prose Pod is publicly accessible via a domain
       When Valerian requests DNS setup instructions
       Then the call should succeed
        And SRV record hostname should be _xmpp-client._tcp.<domain> for port 5222
        And SRV record hostname should be _xmpp-server._tcp.<domain> for port 5269
        And SRV record hostname should be _xmpp-server._tcp.groups.<domain> for port 5269

    Examples:
      | domain         |
      | prose.org      |
      | chat.prose.org |

  Rule: No SRV record is given for port 5269 if federation is disabled

    Scenario: Prose Pod accessible via a domain
      Given the Prose Pod is publicly accessible via a domain
        And federation is disabled
       When Valerian requests DNS setup instructions
       Then the call should succeed
        And DNS setup instructions should contain a SRV record for port 5222
        And DNS setup instructions should not contain a SRV record for port 5269
