Feature: Initializing the Prose Pod address

  Background:
    Given the Prose Pod API has started
      And the server config has been initialized
      And Valerian is an admin

  """
  For federation.
  """
  Rule: IPv4 or hostname is mandatory

    Scenario: The Prose Pod has a static IPv4
      Given Temp

    """
    When the Prose Pod is deployed in a Cloud environment or behind a Load Blancer,
    it doesn't have a static IP address, but can be accessed via a given hostname.
    In this situation, the XMPP server.
    """
    Scenario: The Prose Pod has a dynamic IP address
      Given Temp

  # Rule: IPv6 is optional
