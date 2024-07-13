Feature: XMPP server configuration: Message archive

  Background:
    Given the Prose Pod has been initialized

  Rule: Message archiving can be turned on and off

    Scenario Outline: An admin turns message archiving on/off
      Given message archiving is <initial_state>
        And Valerian is an admin
       When Valerian turns message archiving <new_state>
       Then the call should succeed
        And message archiving should be <new_state>
        And the server should have been reconfigured

    Examples:
      | initial_state | new_state |
      | off           | on        |
      | on            | off       |

  Rule: The message archive retention can be configured

    Scenario: An admin changes the message archive retention
      Given the message archive retention is set to 2 years
        And Valerian is an admin
       When Valerian sets the message archive retention to 1 year
       Then the call should succeed
        And the message archive retention should be set to 1 year
        And the server should have been reconfigured

  Rule: The Messaging configuration can be reset to its default value

    Scenario: An admin resets the Messaging configuration to its default value
      Given message archiving is off
        And the message archive retention is set to 1 year
        And Valerian is an admin
       When Valerian resets the Messaging configuration to its default value
       Then the call should succeed
        And message archiving should be on
        And the message archive retention should be set to infinite
        And the server should have been reconfigured

  Rule: The message archive retention can be reset to its default value

    Scenario: An admin resets the message archive retention to its default value
      Given the message archive retention is set to 1 year
        And Valerian is an admin
       When Valerian resets the Messaging configuration to its default value
       Then the call should succeed
        And the message archive retention should be set to infinite
        And the server should have been reconfigured

  Rule: Turning on/off message archiving is idempotent

    Scenario Outline: Turning on/off twice
      Given message archiving is <initial_state>
        And Valerian is an admin
       When Valerian turns message archiving <initial_state>
       Then the call should succeed
        And message archiving should be <initial_state>
        And the server should not have been reconfigured

    Examples:
      | initial_state |
      | off           |
      | on            |

  Rule: Changing the message archive retention is idempotent

    Scenario Outline: Changing to the same value twice
      Given the message archive retention is set to <initial_state>
        And Valerian is an admin
       When Valerian sets the message archive retention to <initial_state>
       Then the call should succeed
        And the message archive retention should be set to <initial_state>
        And the server should not have been reconfigured

    Examples:
      | initial_state |
      | 1 year        |
      | 2 years       |

  Rule: The Messaging configuration can only be changed by an admin

    Scenario Outline: Unauthorized actions
      Given Rémi is not an admin
       When <action>
       Then the call should not succeed
        And the response content type should be JSON
        And the HTTP status code should be Forbidden

    Examples:
      | action |
      | Rémi turns message archiving off |
      | Rémi sets the message archive retention to 1 year |
