@prosody
Feature: Prosody configuration file

  Background:
    Given the workspace has been initialized

  Rule: A brand new Prose Pod should generate a valid Prosody configuration file

    Scenario: 1
      Given nothing has changed since the initialization of the workspace
       When generating a new Prosody configuration file
       Then `modules_enabled` should contain "roster", "groups", "saslauth", "tls", "dialback", "disco", "posix", "smacks", "private", "vcard_legacy", "vcard4", "version", "uptime", "time", "ping", "lastactivity", "pep", "blocklist", "limits", "carbons", "mam", "csi", "server_contact_info", "websocket", "s2s_bidi"
        And `modules_enabled` should be empty
