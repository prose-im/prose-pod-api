Feature: Workspace name

  Scenario: API should warn if the workspace has not been initialized when getting workspace name
    Given the workspace has not been initialized
     When a user gets the workspace name
     Then the user should receive 'Workspace not initialized'

  Scenario: Get workspace name after initializing
    Given the workspace has been initialized
     When a user gets the workspace name
     Then the call should succeed
      And the response content type should be JSON
      And the returned workspace name should be "Prose"

  Scenario: Change workspace name
    Given the workspace has been initialized
      And the workspace is named "Prose"
     When a user sets the workspace name to "Prose IM"
     Then the call should succeed
      And the response content type should be JSON
      And the returned workspace name should be "Prose IM"
      And the workspace should be named "Prose IM"
