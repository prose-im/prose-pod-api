Feature: Workspace name

  Background:
    Given the Prose Pod API has started

  Rule: The API should warn if the workspace has not been initialized when getting the workspace name

    Scenario: XMPP server and workspace not initialized
      Given the server config has not been initialized
       When a user gets the workspace name
       Then the user should receive 'Server config not initialized'

    Scenario: XMPP server initialized but not the workspace
      Given the server config has been initialized
        And the workspace has not been initialized
       When a user gets the workspace name
       Then the user should receive 'Workspace not initialized'

  Rule: A user can request the workspace name

    Scenario: Get workspace name after initializing
      Given the Prose Pod has been initialized
       When a user gets the workspace name
       Then the call should succeed
        And the response content type should be JSON
        And the returned workspace name should be "Prose"

  Rule: An admin user can change the workspace name

    Scenario: Change workspace name
      Given the Prose Pod has been initialized
        And the workspace is named "Prose"
       When a user sets the workspace name to "Prose IM"
       Then the call should succeed
        And the response content type should be JSON
        And the returned workspace name should be "Prose IM"
        And the workspace should be named "Prose IM"
