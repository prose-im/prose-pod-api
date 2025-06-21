@prosody
Feature: Prosody config

  Background:
    Given the Prose Pod has been initialized
      And the Prose Pod API has started

  Rule: Corrupted server config doesn’t break all fields

    Scenario: One stored Prosody config overrides then we made the schema more strict
      Given the server config is
            """
            {
              "domain": "example.org",
              "prosody_overrides": { "allow_registration": 1 }
            }
            """
        And Valerian is an admin
       When Valerian queries the server configuration
       Then the call should succeed
        And the response JSON should contain key "domain"
        And the response JSON should not contain key "prosody_overrides"

    Scenario: Overrides are valid (ensures the above test really tests something)
      Given the server config is
            """
            {
              "domain": "example.org",
              "prosody_overrides": { "allow_registration": true }
            }
            """
        And Valerian is an admin
        When Valerian queries the server configuration
        Then the call should succeed
        And the response JSON should contain key "domain"
        And the response JSON should contain key "prosody_overrides"
