@init-first-account @init
Feature: Initializing the first admin account

  Background:
    Given the Prose Pod API has started

  Scenario: Creating the first account
    Given the server config has been initialized
      And the workspace has been initialized
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
    Given the server config has not been initialized
     When someone creates the first account "Rémi" with node "remi"
     Then the call should not succeed
      And the user should receive 'Server config not initialized'
