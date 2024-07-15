Feature: Workspace icon

  Rule: The API should warn if the workspace has not been initialized when getting the workspace icon

    Scenario: XMPP server and workspace not initialized
      Given the server config has not been initialized
       When a user gets the workspace icon
       Then the user should receive 'Server config not initialized'

    Scenario: XMPP server initialized but not the workspace
      Given the server config has been initialized
        And the workspace has not been initialized
       When a user gets the workspace icon
       Then the user should receive 'Workspace not initialized'

  Rule: A user can request the workspace icon

    Scenario: Get workspace icon after initializing
      Given the Prose Pod has been initialized
       When a user gets the workspace icon
       Then the call should succeed
        And the response content type should be JSON
        And the returned workspace icon URL should be undefined

    Scenario: Get workspace icon after setting it once
      Given the Prose Pod has been initialized
        And the workspace icon URL is "https://avatars.githubusercontent.com/u/81181949?s=200&v=4"
       When a user gets the workspace icon
       Then the call should succeed
        And the response content type should be JSON
        And the returned workspace icon URL should be "https://avatars.githubusercontent.com/u/81181949?s=200&v=4"

  Rule: An admin user can change the workspace icon

    Scenario: Change workspace icon
      Given the Prose Pod has been initialized
        And the workspace icon URL is "https://avatars.githubusercontent.com/u/81181949?s=200&v=4"
       When a user sets the workspace icon URL to "https://avatars.githubusercontent.com/u/81181949?s=200&v=5"
       Then the call should succeed
        And the response content type should be JSON
        And the returned workspace icon URL should be "https://avatars.githubusercontent.com/u/81181949?s=200&v=5"
        And the workspace icon URL should be "https://avatars.githubusercontent.com/u/81181949?s=200&v=5"
