@prosody
Feature: Prosody configuration file

  Scenario: Default configuration
    Given nothing has changed since the initialization of the workspace
     When generating a new Prosody configuration file from the database
     Then the file should match the snapshot named "default_config"

  Scenario: Everything off
    Given every optional feature has been disabled
     When generating a new Prosody configuration file from the database
     Then the file should match the snapshot named "minimal_config"

  Scenario: Default contact in app configuration
    Given the following app configuration is set:
      | key | value |
      | public_contacts.default | ["mailto:example@example.org"] |
      | public_contacts.support | ["mailto:support@example.org"] |
     When generating a new Prosody configuration file from the database
     Then the file should match the snapshot named "default_config__contact_default"

  Scenario: Default admin contact from database
    Given the following app configuration is set:
      | key | value |
      | public_contacts.support | ["mailto:support@example.org"] |
      And the first admin account is "admin"
     When generating a new Prosody configuration file from the database
     Then the file should match the snapshot named "default_config__default_contact_admin"

  #Rule: A brand new Prose Pod should generate a valid Prosody configuration file
