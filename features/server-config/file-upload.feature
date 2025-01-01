@file-upload @server-config
Feature: XMPP server configuration: File upload

  Background:
    Given the Prose Pod has been initialized
      And the Prose Pod API has started

  Rule: File uploading can be turned on and off

    Scenario Outline: An admin turns file uploading on/off
      Given file uploading is <initial_state>
        And Valerian is an admin
       When Valerian turns file uploading <new_state>
       Then file uploading should be <new_state>
        And the server should have been reconfigured

    Examples:
      | initial_state | new_state |
      | off           | on        |
      | on            | off       |

  Rule: The file retention can be configured

    Scenario: An admin changes the file retention
      Given the file retention is set to 2 years
        And Valerian is an admin
       When Valerian sets the file retention to 1 year
       Then the file retention should be set to 1 year
        And the server should have been reconfigured

  Rule: The Files configuration can be reset to its default value

    Scenario: An admin resets the Files configuration to its default value
      Given file uploading is off
        And the file retention is set to 1 year
        And Valerian is an admin
       When Valerian resets the Files configuration to its default value
       Then file uploading should be on
        And the file retention should be set to infinite
        And the server should have been reconfigured

  Rule: Turning on/off file uploading is idempotent

    Scenario Outline: Turning on/off twice
      Given file uploading is <initial_state>
        And Valerian is an admin
       When Valerian turns file uploading <initial_state>
       Then file uploading should be <initial_state>
        And the server should not have been reconfigured

    Examples:
      | initial_state |
      | off           |
      | on            |

  Rule: Changing the file retention is idempotent

    Scenario Outline: Changing to the same value twice
      Given the file retention is set to <initial_state>
        And Valerian is an admin
       When Valerian sets the file retention to <initial_state>
       Then the file retention should be set to <initial_state>
        And the server should not have been reconfigured

    Examples:
      | initial_state |
      | 1 year        |
      | 2 years       |

  Rule: The Files configuration can only be changed by an admin

    Scenario Outline: Unauthorized actions
      Given Rémi is not an admin
       When <action>
       Then the call should not succeed
        And the response content type should be JSON
        And the HTTP status code should be Forbidden

    Examples:
      | action |
      | Rémi turns file uploading off |
      | Rémi sets the file retention to 1 year |
