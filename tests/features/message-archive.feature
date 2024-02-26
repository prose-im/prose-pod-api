Feature: Message archive

  Background:
    Given workspace has been initialized

  Rule: Message archiving can be turned on and off

    Scenario Outline: An admin turns message archiving on/off
      Given message archiving is <initial_state>
        And Valerian is an admin
       When Valerian turns message archiving <new_state>
       Then message archiving is <new_state>
        And the server is reconfigured

    Examples:
      | initial_state | new_state |
      | off           | on        |
      | on            | off       |

  Rule: Message archive retention time can be configured

    Scenario: An admin changes message archive retention time
      Given message archive retention time is set to 2 years
        And Valerian is an admin
       When Valerian sets the message archive retention time to 1 year
       Then message archive retention time is set to 1 year
        And the server is reconfigured

  Rule: The Messaging configuration can be reset to its default value

    Scenario: An admin resets the Messaging configuration to its default value
      Given message archiving is off
        And message archive retention time is set to 1 year
        And Valerian is an admin
       When Valerian resets the Messaging configuration to its default value
       Then message archiving is on
        And message archive retention time is set to 2 years
        And the server is reconfigured

  Rule: Turning on/off message archiving is idempotent

    Scenario Outline: Turning on/off twice
      Given message archiving is <initial_state>
        And Valerian is an admin
       When Valerian turns message archiving <initial_state>
       Then message archiving is <initial_state>
        And the server is not reconfigured

    Examples:
      | initial_state |
      | off           |
      | on            |

  Rule: Changing message archive retention is idempotent

    Scenario Outline: Changing to the same value twice
      Given message archive retention time is set to <initial_state>
        And Valerian is an admin
       When Valerian turns message archiving <initial_state>
       Then message archive retention time is set to <initial_state>
        And the server is not reconfigured

    Examples:
      | initial_state |
      | 1 year        |
      | 2 years       |

  Rule: The Messaging configuration can only be changed by an admin

    Scenario Outline: Unauthorized actions
      Given Valerian is not an admin
       When <action>
       Then the call should not succeed
        And the response content type should be JSON
        And the HTTP status code should be Unauthorized
        And the response should contain a "WWW-Authenticate" HTTP header

    Examples:
      | action |
      | Valerian sets the message archive retention time to 1 year |
