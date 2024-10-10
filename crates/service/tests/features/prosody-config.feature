@prosody
Feature: Prosody configuration file

  Scenario: Brand new Prose Pod configuration
    Given nothing has changed since the initialization of the workspace
     When generating a new Prosody configuration file from the database
     Then the file should match the snapshot named "default_config"

  Scenario: Everything off
    Given every optional feature has been disabled
     When generating a new Prosody configuration file from the database
     Then the file should match the snapshot named "minimal_config"

  Rule: A brand new Prose Pod should generate a valid Prosody configuration file
