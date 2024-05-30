Feature: Prose Pod initialization

  Scenario: Initializing the workspace
    Given the workspace has not been initialized
     When someone initializes a workspace named "Prose"
     Then the call should succeed
      And the HTTP status code should be Created
      And the response should contain a "Location" HTTP header
      And the response content type should be JSON

  Scenario: Trying to initialize the workspace again
    Given the workspace has been initialized
     When someone initializes a workspace named "Prose"
     Then the call should not succeed
      And the user should receive 'Workspace already initialized'

  Scenario: Initializing the server config
    Given the workspace has not been initialized
     When someone initializes the server at <prose.org>
     Then the call should succeed
      And the HTTP status code should be Created
      And the response should contain a "Location" HTTP header
      And the response content type should be JSON

  Scenario: Trying to initialize the server config again
    Given the server config has been initialized
     When someone initializes the server at <prose.org>
     Then the call should not succeed
      And the user should receive 'Server config already initialized'

  Scenario: Creating the first account
    Given the workspace has been initialized
      And the server config has been initialized
     When someone creates the first account "Rémi" with node "remi"
     Then the call should succeed
      And the HTTP status code should be Created
      And the response should contain a "Location" HTTP header

  Scenario: Trying to create the first account a second time
    Given the Prose Pod has been initialized
      And Valerian is an admin
     When someone creates the first account "Rémi" with node "remi"
     Then the user should receive 'First account already created'

  Scenario: Creating the first account before initializing the workspace
    Given the server config has been initialized
      And the workspace has not been initialized
     When someone creates the first account "Rémi" with node "remi"
     Then the call should succeed
      And the HTTP status code should be Created
      And the response should contain a "Location" HTTP header

  Scenario: Creating the first account before initializing the server config
    Given the workspace has been initialized
      And the server config has not been initialized
     When someone creates the first account "Rémi" with node "remi"
     Then the call should not succeed
      And the user should receive 'Server config not initialized'
