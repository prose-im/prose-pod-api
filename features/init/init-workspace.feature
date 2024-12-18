Feature: Initializing the Prose Workspace

  Background:
    Given the Prose Pod API has started

  Scenario: Initializing the workspace
    Given the server config has been initialized
      And the workspace has not been initialized
     When someone initializes a workspace named "Prose"
     Then the call should succeed
      And the HTTP status code should be Created
      And the response should contain a "Location" HTTP header
      And the response content type should be JSON

  Scenario: Trying to initialize the workspace again
    Given the Prose Pod has been initialized
     When someone initializes a workspace named "Prose"
     Then the call should not succeed
      And the user should receive 'Workspace already initialized'
