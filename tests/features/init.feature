Feature: Initialize a workspace

  Scenario: Initializing a workspace
    Given the workspace has not been initialized
     When a user initializes a workspace named "Prose"
     Then the call should succeed
      And the response content type should be JSON

  Scenario: Trying to initialize a workspace again
    Given the workspace has been initialized
     When a user initializes a workspace named "Prose"
     Then the user should receive 'Prose Pod already initialized'
