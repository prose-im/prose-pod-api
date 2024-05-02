Feature: File upload

  Background:
    Given the workspace has been initialized

  Rule: File uploading can be turned on and off

    Scenario Outline: An admin turns file uploading on/off
      Given file uploading is <initial_state>
        And Valerian is an admin
       When Valerian turns file uploading <new_state>
       Then file uploading is <new_state>
        And the server is reconfigured

    Examples:
      | initial_state | new_state |
      | off           | on        |
      | on            | off       |

  Rule: The file retention can be configured

    Scenario: An admin changes the file retention
      Given the file retention is set to 2 years
        And Valerian is an admin
       When Valerian sets the file retention to 1 year
       Then the file retention is set to 1 year
        And the server is reconfigured

  Rule: The Files configuration can be reset to its default value

    Scenario: An admin resets the Files configuration to its default value
      Given file uploading is off
        And the file retention is set to 1 year
        And Valerian is an admin
       When Valerian resets the Files configuration to its default value
       Then file uploading is on
        And the file retention is set to 2 years
        And the server is reconfigured

  Rule: Turning on/off file uploading is idempotent

    Scenario Outline: Turning on/off twice
      Given file uploading is <initial_state>
        And Valerian is an admin
       When Valerian turns file uploading <initial_state>
       Then file uploading is <initial_state>
        And the server is not reconfigured

    Examples:
      | initial_state |
      | off           |
      | on            |

  Rule: Changing the file retention is idempotent

    Scenario Outline: Changing to the same value twice
      Given the file retention is set to <initial_state>
        And Valerian is an admin
       When Valerian sets the file retention to <initial_state>
       Then the file retention is set to <initial_state>
        And the server is not reconfigured

    Examples:
      | initial_state |
      | 1 year        |
      | 2 years       |

  Rule: The Files configuration can only be changed by an admin

    Scenario Outline: Unauthorized actions
      Given Valerian is not an admin
       When <action>
       Then the call should not succeed
        And the response content type should be JSON
        And the HTTP status code should be Unauthorized
        And the response should contain a "WWW-Authenticate" HTTP header

    Examples:
      | action |
      | Valerian turns file uploading off |
      | Valerian sets the file retention to 1 year |
