Feature: Workspace name

  Scenario: API should warn if workspace has not been initialized when getting workspace name
    Given workspace has not been initialized
     When a user gets the workspace name
     Then the user should receive 'Prose Pod not initialized'

  Scenario: Get workspace name after initializing
    Given workspace has been initialized
     When a user gets the workspace name
     Then the call should succeed
      And the response content type should be JSON
      And the returned workspace name should be "Prose"

  Scenario: Change workspace name
    Given the workspace is named "Prose"
     When a user sets the workspace name to "Prose IM"
     Then the call should succeed
      And the response content type should be JSON
      And the returned workspace name should be "Prose IM"
      And the workspace should be named "Prose IM"
