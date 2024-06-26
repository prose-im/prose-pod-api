Feature: Workspace icon

  Scenario: API should warn if the workspace has not been initialized when getting workspace icon
    Given the workspace has not been initialized
     When a user gets the workspace icon
     Then the user should receive 'Workspace not initialized'

  Scenario: Get workspace icon after initializing
    Given the workspace has been initialized
     When a user gets the workspace icon
     Then the call should succeed
      And the response content type should be JSON
      And the returned workspace icon URL should be undefined

  Scenario: Get workspace icon after setting it once
    Given the workspace has been initialized
      And the workspace icon URL is "https://avatars.githubusercontent.com/u/81181949?s=200&v=4"
     When a user gets the workspace icon
     Then the call should succeed
      And the response content type should be JSON
      And the returned workspace icon URL should be "https://avatars.githubusercontent.com/u/81181949?s=200&v=4"

  Scenario: Change workspace icon
    Given the workspace has been initialized
      And the workspace icon URL is "https://avatars.githubusercontent.com/u/81181949?s=200&v=4"
     When a user sets the workspace icon URL to "https://avatars.githubusercontent.com/u/81181949?s=200&v=5"
     Then the call should succeed
      And the response content type should be JSON
      And the returned workspace icon URL should be "https://avatars.githubusercontent.com/u/81181949?s=200&v=5"
      And the workspace icon URL should be "https://avatars.githubusercontent.com/u/81181949?s=200&v=5"
