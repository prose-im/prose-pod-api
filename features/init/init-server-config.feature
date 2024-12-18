Feature: Initializing the XMPP server configuration

  Background:
    Given the Prose Pod API has started

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
