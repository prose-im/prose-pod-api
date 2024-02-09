Feature: Initialize a workspace

  Scenario: Initializing a workspace
    Given workspace has not been initialized
     When a user ititializes a workspace named "Prose"
     Then the call should succeed
      And the response content type should be JSON

  Scenario: Trying to initializing a workspace again
    Given workspace has been initialized
     When a user ititializes a workspace named "Prose"
     Then the user should receive 'Prose Pod already initialized'
