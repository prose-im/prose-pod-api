@dns-setup
Feature: DNS setup instructions

  Background:
    Given the server config has been initialized
      And Valerian is an admin
      And the Prose Pod API has started

  Rule: Only admins can see DNS setup instructions

    Scenario: Rémi (not admin) tries to get DNS setup instructions
      Given Rémi is not an admin
        And the Prose Pod is publicly accessible via an IPv4
       When Rémi requests DNS setup instructions
       Then the HTTP status code should be Forbidden
        And the response content type should be JSON

  """
  `SRV` records cannot point directly to IP addresses, we need to point it to a hostname.
  """
  Rule: If the Prose Pod has a static IP address, `SRV` records point to `xmpp.<domain>` and `xmpp.<domain>` points to the Prose Pod

    Scenario Outline: IPv4
      Given the Prose Pod is publicly accessible via an IPv4
        And the XMPP server domain is <domain>
       When Valerian requests DNS setup instructions
       Then DNS setup instructions should contain 3 steps
          * step 1 should contain a single A record
          * step 2 should contain a single SRV record
          * step 3 should contain a single SRV record
        And A records hostnames should be xmpp.<domain>
        And AAAA records hostnames should be xmpp.<domain>
        And SRV records targets should be xmpp.<domain>

    Examples:
      | domain           |
      | <prose.org>      |
      | <chat.prose.org> |

    Scenario Outline: IPv4 + IPv6
      Given the Prose Pod is publicly accessible via an IPv4
        And the Prose Pod is publicly accessible via an IPv6
        And the XMPP server domain is <domain>
       When Valerian requests DNS setup instructions
       Then DNS setup instructions should contain 3 steps
          * step 1 should contain A and AAAA records
          * step 2 should contain a single SRV record
          * step 3 should contain a single SRV record
        And A records hostnames should be xmpp.<domain>
        And AAAA records hostnames should be xmpp.<domain>
        And SRV records targets should be xmpp.<domain>

    Examples:
      | domain           |
      | <prose.org>      |
      | <chat.prose.org> |

  Rule: If the Prose Pod is publicly accessible via a hostname, `SRV` records point to it

    """
    TODO: Federation works with hostname only? Valerian says he thinks not but I can't find any mention of it.
    """
    Scenario: Hostname
      Given the Prose Pod is publicly accessible via a hostname
       When Valerian requests DNS setup instructions
       Then DNS setup instructions should contain 2 steps
          * step 1 should contain a single SRV record
          * step 2 should contain a single SRV record

    """
    This scenario should not happen but it's possible because of the database schema.
    """
    Scenario: IPv4 + IPv6 + hostname
      Given the Prose Pod is publicly accessible via an IPv4
        And the Prose Pod is publicly accessible via a hostname
       When Valerian requests DNS setup instructions
       Then DNS setup instructions should contain 2 steps
          * step 1 should contain a single SRV record
          * step 2 should contain a single SRV record

  Rule: DNS setup instructions give SRV records for ports 5222 and 5269

    Scenario: Prose Pod accessible via an IP address
      Given the Prose Pod is publicly accessible via an IPv4
       When Valerian requests DNS setup instructions
       Then DNS setup instructions should contain a SRV record for port 5222
        And DNS setup instructions should contain a SRV record for port 5269

    Scenario: Prose Pod accessible via a hostname
      Given the Prose Pod is publicly accessible via a hostname
       When Valerian requests DNS setup instructions
       Then DNS setup instructions should contain a SRV record for port 5222
        And DNS setup instructions should contain a SRV record for port 5269

  Rule: DNS setup instructions use the XMPP server's domain

    Scenario Outline: If we feed a hungry animal it will no longer be hungry
      Given the XMPP server domain is <domain>
       When Valerian requests DNS setup instructions
       Then SRV records hostnames should be <domain>

    Examples:
      | domain           |
      | <prose.org>      |
      | <chat.prose.org> |

  Rule: The Prose Pod address needs to be initialized in order to get DNS setup instructions

    Scenario: Prose Pod address not initialized
      Given the Prose Pod address has not been initialized
       When Valerian requests DNS setup instructions
       Then the HTTP status code should be Bad Request
        And the error reason should be "prose_pod_address_not_initialized"
