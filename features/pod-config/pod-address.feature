@pod-address @pod-config
Feature: Setting the Prose Pod address

  Background:
    Given the Prose Pod API has started
      And the server config has been initialized
      And the Pod config has been initialized
      And Valerian is an admin

  """
  Admins need to inform the API how people can publicly access the XMPP server
  in order for it to give DNS setup instructions.
  When the Prose Pod is deployed in a Cloud environment or behind a Load Balancer,
  it doesn't have a static IP address, but can be accessed via a given hostname.
  """
  Rule: Admins can set the Prose Pod address

    Scenario Outline: Valerian (not admin) tries to set the Prose Pod address
      When Valerian sets the Prose Pod address to <address>
       Then the call should succeed
        And the response content type should be JSON

    Examples:
      | address    |
      | an IPv4    |
      | an IPv6    |
      | a hostname |

  Rule: Only admins can set the Prose Pod address

    Scenario Outline: Rémi (not admin) tries to set the Prose Pod address
      Given Rémi is not an admin
       When Rémi sets the Prose Pod address to <address>
       Then the HTTP status code should be Forbidden
        And the response content type should be JSON
        And the error code should be "forbidden"

    Examples:
      | address    |
      | an IPv4    |
      | an IPv6    |
      | a hostname |

  Rule: One can change from IP addresses to hostname and vice versa

    Scenario: User had given IP addresses, but wants to switch to a hostname
      Given the Prose Pod is publicly accessible via an IPv4
        And the Prose Pod is publicly accessible via an IPv6
       When Valerian sets the Prose Pod address to a hostname
       Then the call should succeed
        And the response content type should be JSON

    Scenario Outline: User had given a hostname, but wants to switch to IP addresses
      Given the Prose Pod is publicly accessible via a hostname
       When Valerian sets the Prose Pod address to <address>
       Then the call should succeed
        And the response content type should be JSON

    Examples:
      | address    |
      | an IPv4    |
      | an IPv6    |

  Rule: Request body can’t be empty

    """
    See [prose-pod-api#151](https://github.com/prose-im/prose-pod-api/issues/151).

    NOTE: We can’t put this doc string on the `Rule` because it conflicts with
      the above `Examples:` and Cucumber fails to parse the file.
      See [cucumber-rs/gherkin#45](https://github.com/cucumber-rs/gherkin/issues/45).
    """
    Scenario: User passed an empty JSON
       When Valerian sets the Prose Pod address to an empty value
       Then the call should not succeed
        And the HTTP status code should be Unprocessable Entity
